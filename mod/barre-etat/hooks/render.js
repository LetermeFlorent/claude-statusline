// Dessin de la barre, sans aucun appel a la mod : tout ce qui est ici se teste
// hors de Claude Code, contre la sortie de l'ancien statusline.exe.

const THEMES = {
  light: {
    ink: [0, 0, 0],
    empty: [190, 180, 165],
    ctxFrom: [125, 185, 255],
    ctxTo: [40, 90, 180],
  },
  dark: {
    ink: [232, 234, 237],
    empty: [226, 229, 234],
    ctxFrom: [150, 205, 255],
    ctxTo: [70, 130, 220],
  },
}

const PALETTES = {
  '5h': { ok: [120, 110, 200], warn: [170, 90, 175], hot: [200, 60, 95] },
  '7d': { ok: [83, 137, 119], warn: [184, 129, 67], hot: [185, 85, 85] },
  ram: { ok: [30, 120, 125], warn: [190, 130, 60], hot: [200, 60, 60] },
  dsk: { ok: [105, 120, 85], warn: [190, 130, 60], hot: [200, 60, 60] },
  cpu: { ok: [55, 90, 140], warn: [190, 130, 60], hot: [200, 60, 60] },
}

// [debut, fin] du degrade des lettres de l'effort
const EFFORT = {
  light: {
    low: [[70, 100, 97], [72, 92, 114]],
    medium: [[71, 90, 115], [77, 85, 117]],
    high: [[75, 84, 118], [90, 81, 117]],
    xhigh: [[88, 80, 118], [102, 78, 116]],
    max: [[102, 78, 116], [114, 80, 114]],
    plain: [[116, 80, 117], [128, 93, 121]],
  },
  dark: {
    low: [[120, 172, 168], [124, 158, 196]],
    medium: [[122, 156, 198], [132, 146, 202]],
    high: [[130, 144, 204], [156, 140, 202]],
    xhigh: [[152, 138, 204], [176, 134, 200]],
    max: [[176, 134, 200], [196, 138, 196]],
    plain: [[200, 138, 202], [220, 160, 208]],
  },
}

function isObj(v) {
  return v !== null && typeof v === 'object' && !Array.isArray(v)
}

function pick(obj, path) {
  let v = obj
  for (const k of path.split('.')) {
    if (!isObj(v) || !(k in v)) return undefined
    v = v[k]
  }
  return v
}

function hex(v) {
  if (typeof v !== 'string') return null
  const m = /^#?([0-9a-fA-F]{6})$/.exec(v.trim())
  if (!m) return null
  const n = parseInt(m[1], 16)
  return [(n >> 16) & 255, (n >> 8) & 255, n & 255]
}

function num(v, def) {
  return typeof v === 'number' && Number.isFinite(v) ? v : def
}

function clamp01(v) {
  return Math.min(1, Math.max(0, v))
}

// Les arrondis de l'ancien binaire : f64::round, moitie loin de zero
function roundAway(x) {
  return x < 0 ? -Math.floor(-x + 0.5) : Math.floor(x + 0.5)
}

// format!("{:.0}") de Rust : la moitie exacte va au pair
function roundEven(x) {
  const f = Math.floor(x)
  const d = x - f
  if (d > 0.5) return f + 1
  if (d < 0.5) return f
  return f % 2 === 0 ? f : f + 1
}

function fixed1(x) {
  const t = x * 10
  return (roundEven(t) / 10).toFixed(1)
}

export function parseConfig(text, fallbackTheme) {
  let raw = {}
  if (typeof text === 'string' && text.trim() !== '') {
    try {
      const j = JSON.parse(text.replace(/^\uFEFF/, ''))
      if (isObj(j)) raw = j
    } catch {
      raw = {}
    }
  }
  let theme = raw.terminal_background
  if (typeof theme === 'string') theme = theme.toLowerCase()
  if (theme === 'clair' || theme === 'day') theme = 'light'
  if (theme === 'sombre') theme = 'dark'
  if (theme !== 'light' && theme !== 'dark') theme = fallbackTheme === 'dark' ? 'dark' : 'light'
  const t = THEMES[theme]
  const color = (key, def) => hex(pick(raw, key)) ?? def
  const palette = (name, base) => {
    const p = PALETTES[name]
    return {
      ok: color(base + '.ok', p.ok),
      warn: color(base + '.warn', p.warn),
      hot: color(base + '.hot', p.hot),
    }
  }
  const glyph = (k, def) => {
    const v = pick(raw, 'glyphs.' + k)
    return typeof v === 'string' && v !== '' ? v : def
  }
  const seg = (k) => pick(raw, 'segments.' + k) !== false
  const sys = isObj(raw.system) ? raw.system : {}
  const metrics = Array.isArray(sys.metrics) ? sys.metrics.filter((m) => typeof m === 'string') : ['ram', 'io', 'cpu']
  const diskPath = typeof sys.disk_path === 'string' && sys.disk_path !== '' ? sys.disk_path : 'C:\\'
  return {
    theme,
    width: Math.max(1, Math.floor(num(raw.bar_width, 8))),
    bold: raw.bold !== false,
    filled: glyph('filled', '\u25A0'),
    empty: glyph('empty', '\u25A0'),
    half: glyph('half', '\u25AA'),
    marker: typeof raw.auto_compact_marker === 'string' ? raw.auto_compact_marker : '\u00AA',
    effortFlat: raw.effort_style === 'flat',
    segments: { ctx: seg('ctx'), '5h': seg('5h'), '7d': seg('7d'), model: seg('model') },
    gradient: {
      '5h': clamp01(num(pick(raw, 'gradient.5h'), 0.55)),
      '7d': clamp01(num(pick(raw, 'gradient.7d'), 0.3)),
      system: clamp01(num(sys.gradient, 0.45)),
    },
    thresholds: {
      quotaWarn: num(pick(raw, 'thresholds.quota_warn'), 30),
      quotaHot: num(pick(raw, 'thresholds.quota_hot'), 70),
      systemWarn: num(pick(raw, 'thresholds.system_warn'), 70),
      systemHot: num(pick(raw, 'thresholds.system_hot'), 85),
    },
    ink: color('colors.ink_' + theme, t.ink),
    emptyColor: color('colors.empty_' + theme, t.empty),
    ctxFrom: color('colors.ctx_from_' + theme, t.ctxFrom),
    ctxTo: color('colors.ctx_to_' + theme, t.ctxTo),
    palettes: {
      '5h': palette('5h', 'palettes.5h'),
      '7d': palette('7d', 'palettes.7d'),
      ram: palette('ram', 'system.colors.ram'),
      dsk: palette('dsk', 'system.colors.dsk'),
      cpu: palette('cpu', 'system.colors.cpu'),
    },
    system: {
      enabled: sys.enabled !== false,
      metrics,
      diskPath,
      drive: diskPath.split('\\')[0].split('/')[0] || 'C:',
    },
  }
}

function lerp(a, b, t) {
  return [0, 1, 2].map((k) => a[k] + (b[k] - a[k]) * t)
}

function rgb(c) {
  return c.map(roundAway)
}

// Teinte de fin d'une barre : assombrie sur fond clair, eclaircie sur fond sombre
function gradientEnd(cfg, start, g) {
  const target = start.map((c) => roundAway(cfg.theme === 'light' ? c * 0.65 : c + (255 - c) * 0.3))
  return lerp(start, target, g)
}

function cellCount(cfg, p) {
  if (p === null || !(p > 0)) return 0
  const units = roundAway((p * 2 * cfg.width) / 100)
  return Math.min(cfg.width, Math.max(1, Math.ceil(units / 2)))
}

function run(text, color, style = {}) {
  return { text, color, bold: !!style.bold, underline: !!style.underline, dim: !!style.dim }
}

function bar(cfg, p, colorAt) {
  if (p === null) return { runs: [run(cfg.empty.repeat(cfg.width), cfg.emptyColor)], last: null }
  const n = cellCount(cfg, p)
  const runs = []
  let last = null
  for (let i = 0; i < cfg.width; i++) {
    if (i < n) {
      last = colorAt(i, n)
      runs.push(run(i === n - 1 ? cfg.half : cfg.filled, last))
    } else {
      runs.push(run(cfg.empty, cfg.emptyColor))
    }
  }
  return { runs, last }
}

function gradientBar(cfg, p, start, g) {
  const end = gradientEnd(cfg, start, g)
  return bar(cfg, p, (i, n) => rgb(lerp(start, end, n > 1 ? i / (n - 1) : 0)))
}

function quotaStart(cfg, name, p) {
  const pal = cfg.palettes[name]
  if (p >= cfg.thresholds.quotaHot) return pal.hot
  if (p >= cfg.thresholds.quotaWarn) return pal.warn
  return pal.ok
}

function systemStart(cfg, name, p) {
  const pal = cfg.palettes[name]
  if (p > cfg.thresholds.systemHot) return pal.hot
  if (p > cfg.thresholds.systemWarn) return pal.warn
  return pal.ok
}

function strong(cfg, text, color) {
  return run(text, color ?? cfg.ink, { bold: cfg.bold })
}

function valueRun(cfg, text, p, last) {
  return strong(cfg, text, p !== null && roundAway(p) >= 100 && last ? last : cfg.ink)
}

function pctText(p) {
  return p === null ? '--%' : roundAway(p) + '%'
}

export function tokens(n) {
  if (n < 1000) return String(Math.trunc(n))
  if (n < 1e6) return roundEven(n / 1000) + 'k'
  return roundEven(n / 1e6) + 'M'
}

export function resetText(secs, long) {
  const s = Math.floor(secs)
  if (s <= 0) return '0m'
  if (s < 3600) return Math.floor(s / 60) + 'm'
  if (long && s >= 86400) {
    const d = Math.floor(s / 86400)
    const h = Math.floor((s % 86400) / 3600)
    return d + 'd' + String(h).padStart(2, '0') + 'h'
  }
  const h = Math.floor(s / 3600)
  const m = Math.floor((s % 3600) / 60)
  return h + 'h' + String(m).padStart(2, '0') + 'm'
}

function ctxSegment(cfg, ctx, acWindow) {
  const size = ctx && typeof ctx.size === 'number' && ctx.size > 0 ? ctx.size : null
  let pct = ctx && typeof ctx.pct === 'number' ? ctx.pct : null
  let used = ctx && typeof ctx.tokens === 'number' ? ctx.tokens : null
  if (used === null && pct !== null && size !== null) used = (pct * size) / 100
  let denom = size
  let marked = false
  if (size !== null && acWindow !== null && acWindow > 0 && acWindow < size) {
    denom = acWindow
    pct = used === null ? null : (used / acWindow) * 100
    marked = true
  }
  const out = []
  if (marked) out.push(run(cfg.marker, cfg.ink, { dim: true }))
  out.push(strong(cfg, 'ctx'), ' ')
  const span = cfg.width > 1 ? cfg.width - 1 : 1
  const b = bar(cfg, pct, (i) => rgb(lerp(cfg.ctxFrom, cfg.ctxTo, i / span)))
  out.push(...b.runs, ' ')
  let text
  if (denom !== null && used !== null) text = tokens(used) + '/' + tokens(denom)
  else text = pctText(pct)
  out.push(valueRun(cfg, text, pct, b.last))
  return out
}

function quotaSegment(cfg, name, q, now) {
  const p = q && typeof q.pct === 'number' ? q.pct : null
  const out = [strong(cfg, name), ' ']
  const b = p === null ? bar(cfg, null) : gradientBar(cfg, p, quotaStart(cfg, name, p), cfg.gradient[name])
  out.push(...b.runs, ' ', valueRun(cfg, pctText(p), p, b.last))
  const at = q && typeof q.at === 'number' ? q.at : null
  if (at !== null) out.push(' ', strong(cfg, resetText(at - now, name === '7d')))
  else if (name === '5h') out.push(' ', strong(cfg, '--'))
  return out
}

function effortRuns(cfg, level) {
  if (level === null || level === undefined || level === '') return []
  let word = String(level)
  let key = word.toLowerCase()
  if (key === 'auto') key = 'low'
  if (key === 'ultracode') word = 'ultracode(+workflows)'
  const pair = EFFORT[cfg.theme][key] ?? EFFORT[cfg.theme].plain
  const [start, end] = pair
  const chars = Array.from(word)
  const out = [strong(cfg, ':')]
  chars.forEach((ch, i) => {
    const t = chars.length > 1 ? i / (chars.length - 1) : 0
    out.push(run(ch, cfg.effortFlat ? end : rgb(lerp(start, end, t)), { bold: cfg.bold }))
  })
  return out
}

function systemSegment(cfg, metric, sys) {
  if (metric === 'ram') {
    const r = sys && sys.ram
    const p = r && r.total > 0 ? ((r.total - r.avail) / r.total) * 100 : null
    const b = p === null ? bar(cfg, null) : gradientBar(cfg, p, systemStart(cfg, 'ram', p), cfg.gradient.system)
    const gib = 1073741824
    const text = p === null ? '--%' : fixed1((r.total - r.avail) / gib) + '/' + fixed1(r.total / gib) + 'G'
    return [strong(cfg, 'ram'), ' ', ...b.runs, ' ', valueRun(cfg, text, p, b.last)]
  }
  const name = metric === 'io' ? 'dsk' : 'cpu'
  const label = metric === 'io' ? cfg.system.drive : 'cpu'
  const v = sys ? sys[metric] : null
  const p = typeof v === 'number' ? v : null
  const b = p === null ? bar(cfg, null) : gradientBar(cfg, p, systemStart(cfg, name, p), cfg.gradient.system)
  return [strong(cfg, label), ' ', ...b.runs, ' ', valueRun(cfg, pctText(p), p, b.last)]
}

function join(cfg, segments) {
  const out = []
  segments.forEach((s, i) => {
    if (i > 0) out.push(' ', strong(cfg, '|'), ' ')
    out.push(...s)
  })
  return out
}

// data : { ctx: {size, pct, tokens}, acWindow, q5: {pct, at}, q7, model, effort,
// sys: {ram: {total, avail}, io, cpu} }. now en secondes.
export function renderLines(cfg, data, now) {
  const first = []
  if (cfg.segments.ctx) first.push(ctxSegment(cfg, data.ctx, data.acWindow ?? null))
  // Sans aucune donnee de quota, l'ancienne barre masquait les deux segments
  const quotas = data.q5 !== undefined || data.q7 !== undefined
  if (quotas && cfg.segments['5h']) first.push(quotaSegment(cfg, '5h', data.q5, now))
  if (quotas && cfg.segments['7d']) first.push(quotaSegment(cfg, '7d', data.q7, now))
  if (cfg.segments.model) {
    const name = data.model || 'claude'
    first.push([run(name, cfg.ink, { underline: true }), ...effortRuns(cfg, data.effort)])
  }
  const lines = [join(cfg, first)]
  if (cfg.system.enabled) {
    const metrics = cfg.system.metrics.filter((m) => m === 'ram' || m === 'io' || m === 'cpu')
    if (metrics.length > 0) lines.push(join(cfg, metrics.map((m) => systemSegment(cfg, m, data.sys))))
  }
  return lines.map(merge)
}

// Fusionne les morceaux voisins de meme style, pour garder l'arbre court
function merge(parts) {
  const out = []
  for (const p of parts) {
    const r = typeof p === 'string' ? run(p, null) : p
    const prev = out[out.length - 1]
    if (prev && same(prev, r)) prev.text += r.text
    else out.push({ ...r })
  }
  return out
}

function same(a, b) {
  const ca = a.color ? a.color.join(',') : ''
  const cb = b.color ? b.color.join(',') : ''
  return ca === cb && a.bold === b.bold && a.underline === b.underline && a.dim === b.dim
}

export function cssColor(c) {
  return '#' + c.map((v) => v.toString(16).padStart(2, '0')).join('')
}

// Nom du modele tel que l'ancienne barre l'affichait : sans le suffixe entre parentheses
export function modelName(s) {
  if (typeof s !== 'string') return null
  const name = s.replace(/\s*\(.*$/, '').replace(/\[[^\]]*\]$/, '').trim()
  if (name === '' || name === 'Unknown') return null
  // La mod recoit l'identifiant (claude-opus-5-5), la barre affichait le nom (Opus 5.5)
  const cap = (w) => w.charAt(0).toUpperCase() + w.slice(1)
  let m = /^claude-([a-z]+)-(\d+)(?:-(\d{1,2}))?(?:-\d{8})?$/.exec(name)
  if (m) return cap(m[1]) + ' ' + m[2] + (m[3] ? '.' + m[3] : '')
  m = /^claude-(\d+)(?:-(\d{1,2}))?-([a-z]+)(?:-\d{8})?$/.exec(name)
  if (m) return cap(m[3]) + ' ' + m[1] + (m[2] ? '.' + m[2] : '')
  return name
}

// Pourcentages systeme par difference entre deux releves de sysinfo.exe
export function systemSample(line, prev) {
  const f = String(line).trim().split(/\s+/)
  if (f.length < 5) return { sample: prev, sys: null }
  const n = (i) => (f[i] === '-' || f[i] === undefined ? null : Number(f[i]))
  const s = { total: n(0), avail: n(1), idle: n(2), kernel: n(3), user: n(4), dIdle: n(5), dQuery: n(6) }
  const sys = { ram: s.total > 0 ? { total: s.total, avail: s.avail } : null, cpu: null, io: null }
  if (prev) {
    const busy = s.kernel - prev.kernel + (s.user - prev.user)
    const idle = s.idle - prev.idle
    if (busy > 0) sys.cpu = Math.min(100, Math.max(0, (1 - idle / busy) * 100))
    if (s.dQuery !== null && prev.dQuery !== null && s.dIdle !== null && prev.dIdle !== null) {
      const q = s.dQuery - prev.dQuery
      if (q > 0) sys.io = Math.min(100, Math.max(0, (1 - (s.dIdle - prev.dIdle) / q) * 100))
    }
  }
  return { sample: s, sys }
}
