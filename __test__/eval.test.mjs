import test from 'node:test'
import assert from 'node:assert/strict'
import { createRequire } from 'node:module'

const require = createRequire(import.meta.url)
const pk = require('../index.js')

// THE HAND, Negreanu vs Hansen. pkcore's own `from__seven` test pins every
// value asserted below at src/analysis/eval.rs:383-392.
const THE_HAND_SEVEN = '6♠ 6♥ 9♣ 6♦ 5♥ 5♠ 8♠'
const THE_HAND_BEST_FIVE = '6♠ 6♥ 6♦ 5♠ 5♥'
const THE_HAND_RANK_VALUE = 271

test('eval_matches_kernel_fixture: THE HAND scores exactly what pkcore pins', () => {
  const evaluation = pk.Eval.fromSeven(THE_HAND_SEVEN)
  assert.equal(evaluation.handRank.value, THE_HAND_RANK_VALUE)
  assert.equal(evaluation.handRank.name, 'FullHouse')
  assert.equal(evaluation.handRank.class, 'SixesOverFives')
  assert.equal(evaluation.bestFive.toString(), THE_HAND_BEST_FIVE)
})

test('Eval.fromCards agrees with Eval.fromSeven', () => {
  const fromText = pk.Eval.fromSeven(THE_HAND_SEVEN)
  const fromCards = pk.Eval.fromCards(pk.Cards.parse(THE_HAND_SEVEN))
  assert.ok(fromCards.handRank.equals(fromText.handRank))
  assert.equal(fromCards.bestFive.toString(), THE_HAND_BEST_FIVE)
})

test('lower rank value is a stronger hand', () => {
  const royal = pk.Eval.fromSeven('A♠ K♠ Q♠ J♠ T♠ 2♦ 3♣')
  assert.equal(royal.handRank.value, 1)
  assert.ok(royal.handRank.value < pk.Eval.fromSeven(THE_HAND_SEVEN).handRank.value)
})

test('a seven-card string with the wrong count throws with a pkcore code', () => {
  assert.throws(
    () => pk.Eval.fromCards(pk.Cards.parse('A♠ K♠')),
    (err) => {
      assert.equal(typeof err.code, 'string')
      assert.notEqual(err.code, 'InvalidArg')
      return true
    },
  )
})
