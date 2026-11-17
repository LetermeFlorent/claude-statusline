import { expect, test } from 'claude-code/testing'
import { body, shortModel } from './register'

test('short model names', () => {
  expect(shortModel('claude-opus-5-5')).toBe('Opus 5.5')
  expect(shortModel('claude-haiku-4-5-20251001')).toBe('Haiku 4.5')
  expect(shortModel('gpt-x')).toBe('gpt-x')
})

test('falls back on the last request', () => {
  const text = body({ type: 'Explore', label: 'search', lastRequest: { model: 'claude-sonnet-5-5', effort: 'high' } })
  expect(text).toContain('Sonnet 5.5')
  expect(text).toContain('high')
})

test('hides what is missing', () => {
  expect(body({ type: 'Explore', label: '' })).toBe('Explore')
  expect(body({ type: 'x', label: '', model: 'claude-opus-5-5' })).toBe('x  Opus 5.5')
})

test('shows sent and received tokens on the right', () => {
  const usage = [{ input_tokens: 500, cache_read_input_tokens: 20_000, output_tokens: 1200 }]
  const text = body({ type: 'Explore', label: 'a long task label', model: 'claude-sonnet-5-5', usage, columns: 60 })
  const plain = text.replace(/\x1b\[[0-9;]*m/g, '')
  expect(plain.endsWith('\u2191 21k  \u2193 1.2k')).toBe(true)
  expect([...plain].length).toBe(60)
  const narrow = body({ type: 'Explore', label: 'a long task label', model: 'claude-sonnet-5-5', usage, columns: 40 })
  expect([...narrow.replace(/\x1b\[[0-9;]*m/g, '')].length).toBeLessThanOrEqual(40)
})
