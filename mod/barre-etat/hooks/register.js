import { parseConfig, renderLines, cssColor, modelName, systemSample } from './render.js'

const USAGE_URL = 'https://api.anthropic.com/api/oauth/usage'
// L'API des quotas limite les appels : une lecture partagee par compte, gardee deux minutes
const QUOTA_TTL_MS = 120000
const QUOTA_RETRY_MS = 300000
const TICK_MS = 3000

let cfg = parseConfig(null)
let cfgText = null
let cfgPath = null
let home = ''
let themeFallback = 'light'
let usage = null
let apiQuota = null
let lastQuotaTry = 0
let model = null
let effort = null
let acWindow = null
let sys = null
let sample = null
let helper = null
let lastHelperTry = 0
let busy = false
let lines = null
let signature = ''

function num(v) {
  const n = typeof v === 'string' ? Number(v.trim()) : v
  return typeof n === 'number' && Number.isFinite(n) && n > 0 ? n : null
}

function epoch(iso) {
  if (typeof iso !== 'string' || iso === '') return null
  const t = Date.parse(iso)
  return Number.isFinite(t) ? Math.floor(t / 1000) : null
}

function quotaOf(kind) {
  const live = usage && Array.isArray(usage.rateLimits) ? usage.rateLimits.find((r) => r.kind === kind) : null
  if (live && typeof live.percentUsed === 'number') return { pct: live.percentUsed, at: epoch(live.resetsAt) }
  const saved = apiQuota ? apiQuota[kind] : null
  if (saved) return saved
  return null
}

function hasLiveQuota() {
  return !!(usage && Array.isArray(usage.rateLimits) && usage.rateLimits.some((r) => r.kind === 'five_hour' || r.kind === 'seven_day'))
}

function snapshot() {
  const c = usage ? usage.context : null
  const q5 = quotaOf('five_hour')
  const q7 = quotaOf('seven_day')
  const none = q5 === null && q7 === null
  return {
    ctx: c ? { size: num(c.window), pct: typeof c.percent === 'number' ? c.percent : null, tokens: typeof c.tokens === 'number' ? c.tokens : null } : null,
    acWindow,
    q5: none ? undefined : q5,
    q7: none ? undefined : q7,
    model,
    effort,
    sys,
  }
}

function refresh(nowMs) {
  lines = renderLines(cfg, snapshot(), Math.floor(nowMs / 1000))
  const sig = JSON.stringify(lines)
  const changed = sig !== signature
  signature = sig
  return changed
}

async function loadConfig($) {
  let text = null
  try {
    text = await $.fs.read(cfgPath)
  } catch {
    text = null
  }
  if (text !== cfgText || cfg.theme === undefined) {
    cfgText = text
    cfg = parseConfig(text, themeFallback)
  }
}

async function loadTheme($) {
  const forced = await $.env.get('STATUSLINE_BG')
  if (forced) {
    themeFallback = /^(light|clair|day)$/i.test(forced) ? 'light' : 'dark'
    return
  }
  try {
    const row = (await $.config.list()).find((r) => r.key === 'theme')
    if (row && typeof row.value === 'string') themeFallback = /light/i.test(row.value) ? 'light' : 'dark'
  } catch {
    themeFallback = 'light'
  }
}

async function loadSettings($) {
  let s = {}
  try {
    s = await $.settings.read()
  } catch {
    s = {}
  }
  const env = s && typeof s.env === 'object' && s.env !== null ? s.env : {}
  acWindow = num(await $.env.get('CLAUDE_CODE_AUTO_COMPACT_WINDOW')) ?? num(env.CLAUDE_CODE_AUTO_COMPACT_WINDOW) ?? num(s.autoCompactWindow)
  if (effort === null && typeof s.effortLevel === 'string') effort = s.effortLevel
}

// Le jeton de la session d'abord. CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC le
// retient : on prend alors celui du compte, comme le fait le menu clm
async function quotaRequest($) {
  const headers = { 'anthropic-beta': 'oauth-2025-04-20', accept: 'application/json' }
  try {
    const auth = await $.session.authorize()
    if (auth && auth.kind !== 'bearer') return null
    if (auth) return await $.http.fetch(USAGE_URL, { auth: auth.handle, headers })
  } catch {
    // jeton retenu, on passe au fichier du compte
  }
  const dir = (await $.env.get('CLAUDE_CONFIG_DIR')) || home + '\\.claude'
  const cred = JSON.parse(await $.fs.read(dir + '\\.credentials.json'))
  const token = cred && cred.claudeAiOauth ? cred.claudeAiOauth.accessToken : null
  if (typeof token !== 'string' || token === '') return null
  return await $.http.fetch(USAGE_URL, { headers: { ...headers, authorization: 'Bearer ' + token } })
}

async function fetchQuota($, nowMs) {
  lastQuotaTry = nowMs
  const cached = await $.store.get('quota')
  if (cached && typeof cached === 'object' && nowMs - cached.at < QUOTA_TTL_MS) {
    apiQuota = cached
    return
  }
  try {
    const r = await quotaRequest($)
    if (!r || !r.ok) {
      if (cached && typeof cached === 'object') apiQuota = cached
      return
    }
    const j = JSON.parse(r.text)
    const pick = (w) => (w && typeof w.utilization === 'number' ? { pct: w.utilization, at: epoch(w.resets_at) } : null)
    apiQuota = { at: nowMs, five_hour: pick(j.five_hour), seven_day: pick(j.seven_day) }
    await $.store.set('quota', apiQuota)
  } catch {
    if (cached && typeof cached === 'object') apiQuota = cached
  }
}

// Le module des hooks n'a pas acces au systeme : un petit programme C# compile
// une fois sur le poste avec le csc.exe livre avec Windows lit les compteurs
async function ensureHelper($, nowMs) {
  if (helper) return helper
  if (nowMs - lastHelperTry < 60000) return null
  lastHelperTry = nowMs
  const local = await $.env.get('LOCALAPPDATA')
  const root = await $.env.get('SystemRoot') || 'C:\\Windows'
  if (!local) return null
  const dir = local + '\\claude-statusline'
  const exe = dir + '\\barre-etat-sys.exe'
  const src = dir + '\\barre-etat-sys.cs'
  try {
    const code = await $.fs.read($.plugin.root + '/helper/sysinfo.cs')
    let current = null
    try {
      current = await $.fs.read(src)
    } catch {
      current = null
    }
    if (current === code && (await $.fs.exists(exe))) {
      helper = exe
      return helper
    }
    await $.fs.write(src, code)
    for (const fw of ['Framework64', 'Framework']) {
      const csc = root + '\\Microsoft.NET\\' + fw + '\\v4.0.30319\\csc.exe'
      if (!(await $.fs.exists(csc))) continue
      const r = await $.process.run([csc, '-nologo', '-optimize+', '-target:exe', '-out:' + exe, src], { timeoutMs: 60000 })
      if (r.exitCode === 0) {
        helper = exe
        return helper
      }
    }
  } catch {
    helper = null
  }
  return null
}

async function readSystem($, nowMs) {
  if (!cfg.system.enabled) {
    sys = null
    return
  }
  const exe = await ensureHelper($, nowMs)
  if (!exe) return
  try {
    const r = await $.process.run([exe, cfg.system.drive], { timeoutMs: 5000 })
    if (r.exitCode !== 0) return
    const next = systemSample(r.stdout, sample)
    sample = next.sample
    if (next.sys) sys = next.sys
  } catch {
    helper = null
  }
}

async function tick($) {
  if (busy) return
  busy = true
  try {
    const nowMs = await $.clock.now()
    await loadConfig($)
    await readSystem($, nowMs)
    usage = await $.session.usage()
    model = modelName(await $.session.model()) ?? model
    if (!hasLiveQuota() && nowMs - lastQuotaTry > (apiQuota ? QUOTA_RETRY_MS : QUOTA_TTL_MS)) await fetchQuota($, nowMs)
    if (refresh(nowMs)) $.ui.invalidate('ui.render')
  } catch {
    // un releve manque, le suivant repartira
  } finally {
    busy = false
  }
}

async function start($) {
  home = (await $.env.get('USERPROFILE')) || (await $.env.get('HOME')) || ''
  cfgPath = await $.env.get('STATUSLINE_CONFIG') || home + '/.claude/statusline.json'
  await loadTheme($)
  cfgText = undefined
  await loadConfig($)
  await loadSettings($)
  const nowMs = await $.clock.now()
  usage = await $.session.usage()
  model = modelName(await $.session.model())
  if (!hasLiveQuota()) await fetchQuota($, nowMs)
  refresh(nowMs)
  $.ui.invalidate('ui.render')
  await readSystem($, nowMs)
  if (refresh(nowMs)) $.ui.invalidate('ui.render')
}

function rows(Text, list) {
  return list.map((runs) =>
    Text({
      wrap: 'truncate-end',
      children: runs.map((r) =>
        r.color
          ? Text({ color: cssColor(r.color), bold: r.bold, underline: r.underline, dimColor: r.dim, children: [r.text] })
          : r.text,
      ),
    }),
  )
}

export function register(on) {
  on('session.start', async ($, e, next) => {
    await start($)
    $.clock.every(TICK_MS, () => tick($))
    return next(e)
  })

  on('session.measure', async ($, e, next) => {
    usage = { context: e.context, rateLimits: e.rateLimits }
    if (refresh(await $.clock.now())) $.ui.invalidate('ui.render')
    return next(e)
  })

  on('turn.step', async function* ($, e, next) {
    if (!e.agentId && e.effort !== undefined && e.effort !== null) effort = String(e.effort)
    return yield* next(e)
  })

  on('ui.render', { component: 'AbovePrompt' }, async ($, e, next) => {
    if (e.props.hasSurvey) return next(e)
    if (!lines) refresh(await $.clock.now())
    const { Box, Text } = $.ui.resolve(e)
    const max = Math.max(1, e.props.maxRows || lines.length)
    const theirs = await next(e)
    const children = rows(Text, lines.slice(0, max))
    if (theirs) children.push(theirs)
    return Box({ flexDirection: 'column', children })
  })
}
