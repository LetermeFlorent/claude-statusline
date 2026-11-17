import type { Register } from 'claude-code'

const TONES: Record<string, [number, number, number]> = {
  low: [102, 187, 106],
  auto: [102, 187, 106],
  medium: [230, 200, 90],
  high: [245, 155, 70],
  xhigh: [240, 95, 90],
  max: [215, 100, 200],
}
const BEYOND: [number, number, number] = [80, 200, 220]
const DIM = '\x1b[2m'
const RESET = '\x1b[0m'

type Usage = {
  input_tokens?: number
  output_tokens?: number
  cache_read_input_tokens?: number
  cache_creation_input_tokens?: number
}

export function shortModel(id: string): string {
  const m = /^claude-(opus|sonnet|haiku|fable)-(\d{1,2}(?:-\d{1,2})*)(?:-\d{3,}.*)?(?:\[.*\])?$/.exec(id)
  const family = m?.[1]
  const version = m?.[2]
  if (!family || !version) return id
  return `${family.charAt(0).toUpperCase()}${family.slice(1)} ${version.replace(/-/g, '.')}`
}

function paint(text: string, rgb: [number, number, number]): string {
  return `\x1b[1;38;2;${rgb[0]};${rgb[1]};${rgb[2]}m${text}${RESET}`
}

function width(text: string): number {
  return [...text.replace(/\x1b\[[0-9;]*m/g, '')].length
}

export function amount(n: number): string {
  if (n < 1000) return String(n)
  if (n < 1_000_000) return `${(n / 1000).toFixed(n < 10_000 ? 1 : 0)}k`
  return `${(n / 1_000_000).toFixed(1)}M`
}

export function traffic(usage: readonly Usage[] | undefined): string {
  if (!usage?.length) return ''
  let sent = 0
  let received = 0
  for (const u of usage) {
    sent += (u.input_tokens ?? 0) + (u.cache_read_input_tokens ?? 0) + (u.cache_creation_input_tokens ?? 0)
    received += u.output_tokens ?? 0
  }
  return `↑ ${amount(sent)}  ↓ ${amount(received)}`
}

export function body(row: {
  name?: string
  type: string
  label: string
  model?: string
  effort?: string | number
  lastRequest?: { model: string; effort?: string | number }
  usage?: readonly Usage[]
  columns?: number
}): string {
  const model = row.model ?? row.lastRequest?.model
  const effort = row.effort ?? row.lastRequest?.effort
  const level = effort === undefined ? '' : String(effort)
  const parts = [
    model ? shortModel(model) : '',
    level ? paint(level, TONES[level.toLowerCase()] ?? BEYOND) : '',
  ].filter(Boolean)
  const left = [row.name || row.type, parts.join(':')].filter(Boolean).join('  ')
  const right = traffic(row.usage)
  let label = row.label.split('\n')[0] ?? ''
  if (row.columns && right) {
    const room = row.columns - width(left) - width(right) - 4
    label = room > 1 ? [...label].slice(0, room).join('') : ''
    const used = width(left) + (label ? 2 + width(label) : 0)
    const gap = Math.max(2, row.columns - used - width(right))
    return `${left}${label ? `  ${DIM}${label}${RESET}` : ''}${' '.repeat(gap)}${DIM}${right}${RESET}`
  }
  return [left, label && `${DIM}${label}${RESET}`, right && `${DIM}${right}${RESET}`].filter(Boolean).join('  ')
}

export const register: Register = on => {
  on('ui.render', { component: 'AgentRow' }, ($, e, next) => {
    return next({ ...e, props: { ...e.props, body: body(e.props) } })
  })
}
