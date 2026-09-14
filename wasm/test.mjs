// Exercises the built package (pkg/, from `wasm-pack build --release --target web`)
// under Node. The generation logic itself is tested in ../src/password.rs; this
// checks the JavaScript boundary: option parsing, defaults, errors, results.
import { test, before } from 'node:test'
import assert from 'node:assert/strict'
import { readFileSync } from 'node:fs'

const pkg = new URL('./pkg/', import.meta.url)
let generatePassword, validate, entropyBits

before(async () => {
  const mod = await import(new URL('password_generator_wasm.js', pkg))
  mod.initSync({ module: readFileSync(new URL('password_generator_wasm_bg.wasm', pkg)) })
  ;({ generatePassword, validate, entropyBits } = mod)
})

test('defaults: 16 characters in four blocks', () => {
  const pw = generatePassword()
  assert.equal(pw.length, 19)
  assert.equal(pw.split('.').length, 4)
  assert.match(pw, /^[a-zA-Z0-9.]+$/)
})

test('options object with partial fields', () => {
  assert.match(generatePassword({ length: 6, separator: '', base: { digits: true } }), /^[0-9]{6}$/)
  const pw = generatePassword({ length: 8, separator: '--', base: { lower: true }, one: { upper: true } })
  assert.equal(pw.length, 10)
  assert.equal([...pw].filter((c) => /[A-Z]/.test(c)).length, 1)
})

test('strict adds exactly one special character', () => {
  for (let i = 0; i < 50; i++) {
    const pw = generatePassword({ strict: true, separator: '' })
    assert.equal([...pw].filter((c) => '!#$%&*+=?@_'.includes(c)).length, 1, pw)
  }
})

test('invalid options throw with the CLI wording', () => {
  assert.throws(() => generatePassword({ length: 3 }), /invalid length 3: allowed range is 4 to 128/)
  assert.throws(() => validate({ separator: '---------' }), /separator too long \(9\)/)
  assert.throws(() => validate({ base: { lower: true }, one: { lower: true } }), /one-lower is redundant/)
  assert.throws(() => generatePassword({ length: 'sixteen' }), /invalid options/)
  assert.doesNotThrow(() => validate({}))
  assert.doesNotThrow(() => validate(null))
})

test('entropy estimate', () => {
  assert.equal(entropyBits(), 92)
  assert.equal(entropyBits({ length: 20 }), 115)
  assert.equal(entropyBits({ base: { lower: true } }), 71)
})
