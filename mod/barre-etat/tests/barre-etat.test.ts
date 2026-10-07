import { expect, mock, test } from 'claude-code/testing'
import { parseConfig, renderLines, tokens, resetText, modelName, systemSample } from '../hooks/render.js'

const CONFIG = JSON.stringify({
  terminal_background: 'light',
  gradient: { '5h': 0.55, '7d': 0.3 },
  system: { enabled: true, metrics: ['ram', 'io', 'cpu'], gradient: 0.45 },
})

const BAND = {
  plugin: 'barre-etat',
  component: 'AbovePrompt',
  viewport: { columns: 120, rows: 30 },
  props: {
    hasSurvey: false,
    isWorking: false,
    maxRows: 10,
    bodyColumns: 120,
    scroll: { offset: 0, bodyRows: 10 },
    view: {},
  },
} as const

// Compteurs bruts de sysinfo.exe : 16 Gio dont 4 libres, puis 3 s plus tard
const SAMPLES = [
  '17179869184 4294967296 1000 2000 1000 500 1000\n',
  '17179869184 4294967296 1600 3000 1500 800 2000\n',
]

function text(el: any): string {
  if (typeof el === 'string') return el
  if (!el || !Array.isArray(el.children)) return ''
  return el.children.map(text).join('')
}

function colors(el: any, out: string[] = []): string[] {
  if (el && typeof el === 'object') {
    if (el.props && el.props.color) out.push(el.props.color)
    for (const c of el.children ?? []) colors(c, out)
  }
  return out
}

test('les nombres et les durees suivent l ancienne barre', () => {
  expect(tokens(999)).toBe('999')
  expect(tokens(1500)).toBe('2k')
  expect(tokens(2500)).toBe('2k')
  expect(tokens(999600)).toBe('1000k')
  expect(tokens(2500000)).toBe('2M')
  expect(resetText(59, false)).toBe('0m')
  expect(resetText(7300, false)).toBe('2h01m')
  expect(resetText(90000, false)).toBe('25h00m')
  expect(resetText(90000, true)).toBe('1d01h')
  expect(modelName('Opus 5.5 (1M context)')).toBe('Opus 5.5')
  expect(modelName('Unknown')).toBe(null)
})

test('le degrade du quota 5 h reprend les teintes de statusline.exe', () => {
  const cfg = parseConfig(CONFIG)
  const [line] = renderLines(cfg, { ctx: null, q5: { pct: 36, at: 7300 }, q7: null, model: 'Opus 5.5', effort: 'high', sys: null }, 0)
  const chars = line.flatMap((r: any) => Array.from(r.text as string).map((ch) => ({ ch, color: r.color })))
  const cells = chars.filter((c: any) => c.ch === '\u25A0' || c.ch === '\u25AA').slice(8, 11)
  expect(cells.map((r: any) => r.color)).toEqual([
    [170, 90, 175],
    [154, 81, 158],
    [138, 73, 141],
  ])
  expect(line.map((r: any) => r.text).join('')).toContain('36% 2h01m')
})

test('la cpu et le disque se calculent entre deux releves', () => {
  const a = systemSample(SAMPLES[0], null)
  expect(a.sys.cpu).toBe(null)
  expect(a.sys.ram).toEqual({ total: 17179869184, avail: 4294967296 })
  const b = systemSample(SAMPLES[1], a.sample)
  expect(b.sys.cpu).toBe(60)
  expect(b.sys.io).toBe(70)
})

test('la bande au-dessus du prompt affiche les deux lignes', async ($, on) => {
  const clock = mock.clock(on, { now: 1_800_000_000_000 })
  mock.env(on, { USERPROFILE: 'C:\\Users\\test', LOCALAPPDATA: 'C:\\Users\\test\\AppData\\Local', SystemRoot: 'C:\\Windows' })
  const saved = new Map<string, unknown>()
  on('store.get', ($, e) => ({ value: saved.get(e.key) }))
  on('store.set', ($, e) => {
    saved.set(e.key, e.value)
    return { value: undefined }
  })
  on('fs.read', ($, e) => {
    if (e.path.endsWith('statusline.json')) return { value: CONFIG }
    if (e.path.endsWith('sysinfo.cs')) return { value: 'class X {}' }
    return { deny: 'absent' }
  })
  on('fs.write', () => ({ value: undefined }))
  on('fs.exists', ($, e) => ({ value: e.path.endsWith('csc.exe') }))
  let sampleIndex = 0
  const runs: string[][] = []
  on('process.run', ($, e) => {
    runs.push([...e.argv])
    if (String(e.argv[0]).endsWith('csc.exe')) return { value: { exitCode: 0, stdout: '', stderr: '' } }
    const out = SAMPLES[Math.min(sampleIndex++, SAMPLES.length - 1)]
    return { value: { exitCode: 0, stdout: out, stderr: '' } }
  })
  on('config.list', () => ({ value: [] }))
  on('settings.read', () => ({ value: { effortLevel: 'high' } }))
  on('session.usage', () => ({
    value: { context: { window: 1000000, tokens: 230000, percent: 23 }, rateLimits: [] },
  }))
  on('session.model', () => ({ value: 'Opus 5.5 (1M context)' }))
  on('session.authorize', () => ({ value: { handle: 'h', kind: 'bearer' } }))
  const fetched: string[] = []
  on('http.fetch', ($, e) => {
    fetched.push(e.url)
    return {
      value: {
        status: 200,
        ok: true,
        headers: {},
        text: JSON.stringify({
          five_hour: { utilization: 36, resets_at: '2027-01-15T10:01:40Z' },
          seven_day: { utilization: 72.5, resets_at: '2027-01-17T08:00:00Z' },
        }),
      },
    }
  })
  on('session.start', () => ({ cwd: 'C:\\work' }))
  on('ui.render', () => ({ type: 'Text', props: {}, children: ['autre mod'] }))

  await $.session.start({ surface: 'terminal', isInteractive: true, cwd: 'C:\\work' })
  expect(fetched.length).toBe(1)
  expect((saved.get('quota') as any).five_hour.pct).toBe(36)
  expect(runs.some((a) => String(a[0]).endsWith('csc.exe'))).toBe(true)

  let ui = await $.ui.mount({ ...BAND, surface: 'terminal' })
  let rows = (await ui.find({ type: 'Box' })) as any
  let all = rows.children.map(text)
  expect(all[0]).toBe('ctx \u25A0\u25AA\u25A0\u25A0\u25A0\u25A0\u25A0\u25A0 230k/1M | 5h \u25A0\u25A0\u25AA\u25A0\u25A0\u25A0\u25A0\u25A0 36% ' + resetText(Date.parse('2027-01-15T10:01:40Z') / 1000 - 1_800_000_000, false) + ' | 7d \u25A0\u25A0\u25A0\u25A0\u25A0\u25AA\u25A0\u25A0 73% ' + resetText(Date.parse('2027-01-17T08:00:00Z') / 1000 - 1_800_000_000, true) + ' | Opus 5.5:high')
  expect(all[1]).toBe('ram \u25A0\u25A0\u25A0\u25A0\u25A0\u25AA\u25A0\u25A0 12.0/16.0G | C: \u25A0\u25A0\u25A0\u25A0\u25A0\u25A0\u25A0\u25A0 --% | cpu \u25A0\u25A0\u25A0\u25A0\u25A0\u25A0\u25A0\u25A0 --%')
  expect(all[2]).toBe('autre mod')
  expect(colors(rows.children[0])).toContain('#7db9ff')
  await ui.unmount()

  await clock.advance(3000)
  ui = await $.ui.mount({ ...BAND, surface: 'terminal' })
  rows = (await ui.find({ type: 'Box' })) as any
  all = rows.children.map(text)
  expect(all[1]).toContain('C: \u25A0\u25A0\u25A0\u25A0\u25A0\u25AA\u25A0\u25A0 70%')
  expect(all[1]).toContain('cpu \u25A0\u25A0\u25A0\u25A0\u25AA\u25A0\u25A0\u25A0 60%')
  // Le cache de deux minutes evite un second appel a l'API
  expect(fetched.length).toBe(1)
  await ui.unmount()
})

test('sans jeton de session, les quotas viennent du compte puis de la session', async ($, on) => {
  mock.clock(on, { now: 1_800_000_000_000 })
  mock.env(on, { USERPROFILE: 'C:\\Users\\test', CLAUDE_CONFIG_DIR: 'C:\\Users\\test\\.claude-compte2' })
  mock.store(on, {})
  on('fs.read', ($, e) => {
    if (e.path.endsWith('statusline.json')) return { value: JSON.stringify({ terminal_background: 'light', system: { enabled: false } }) }
    if (e.path === 'C:\\Users\\test\\.claude-compte2\\.credentials.json') return { value: JSON.stringify({ claudeAiOauth: { accessToken: 'jeton-test' } }) }
    return { deny: 'absent' }
  })
  on('config.list', () => ({ value: [] }))
  on('settings.read', () => ({ value: {} }))
  on('session.usage', () => ({ value: { context: { window: 200000 }, rateLimits: [] } }))
  on('session.model', () => ({ value: 'claude-sonnet-5-5' }))
  on('session.authorize', () => ({ deny: 'nonessential network traffic is disabled for this session' }))
  const headers: Record<string, string>[] = []
  on('http.fetch', ($, e) => {
    headers.push(e.init.headers)
    return { value: { status: 200, ok: true, headers: {}, text: JSON.stringify({ five_hour: { utilization: 12, resets_at: null }, seven_day: null }) } }
  })
  on('session.start', () => ({ cwd: 'C:\\work' }))
  on('session.measure', () => ({ changed: ['rateLimits'] }))
  on('ui.render', () => ({ type: 'Text', props: {}, children: [''] }))

  await $.session.start({ surface: 'terminal', isInteractive: true, cwd: 'C:\\work' })
  expect(headers[0].authorization).toBe('Bearer jeton-test')
  let ui = await $.ui.mount({ ...BAND, surface: 'terminal' })
  let row = text(((await ui.find({ type: 'Box' })) as any).children[0])
  expect(row).toContain('5h \u25AA\u25A0\u25A0\u25A0\u25A0\u25A0\u25A0\u25A0 12% -- | 7d \u25A0\u25A0\u25A0\u25A0\u25A0\u25A0\u25A0\u25A0 --%')
  expect(row).toContain('Sonnet 5.5')
  await ui.unmount()

  await $.session.measure({
    context: { window: 200000, tokens: 50000, percent: 25 },
    rateLimits: [{ kind: 'five_hour', percentUsed: 40, resetsAt: '2027-01-15T10:00:00Z' }],
    changed: ['rateLimits'],
  })
  ui = await $.ui.mount({ ...BAND, surface: 'terminal' })
  row = text(((await ui.find({ type: 'Box' })) as any).children[0])
  expect(row).toContain('ctx \u25A0\u25AA\u25A0\u25A0\u25A0\u25A0\u25A0\u25A0 50k/200k')
  expect(row).toContain('5h \u25A0\u25A0\u25AA\u25A0\u25A0\u25A0\u25A0\u25A0 40%')
  await ui.unmount()
})

test('la bande cede la place a un sondage', async ($, on) => {
  on('ui.render', () => ({ type: 'Text', props: {}, children: ['sondage'] }))
  const ui = await $.ui.mount({ ...BAND, surface: 'terminal', props: { ...BAND.props, hasSurvey: true } })
  expect(await ui.find({ type: 'Text', text: 'sondage' })).toBeDefined()
  await ui.unmount()
})
