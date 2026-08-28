import test from 'node:test'
import assert from 'node:assert/strict'
import { createRequire } from 'node:module'

const require = createRequire(import.meta.url)
const pk = require('../index.js')

// THE HAND again: Negreanu 6s6h, Hansen 5d5c, board 9c 6d 5h 5s 8s.
const THE_HAND_BOARD = '9♣ 6♦ 5♥ 5♠ 8♠'
const THE_HAND_HOLES = '6♠ 6♥ 5♦ 5♣'

test('Board splits the five community cards into streets', () => {
  const board = pk.Board.parse(THE_HAND_BOARD)
  assert.equal(board.flop.toString(), '9♣ 6♦ 5♥')
  assert.equal(board.turn.toString(), '5♠')
  assert.equal(board.river.toString(), '8♠')
  assert.equal(board.turnCards().toString(), '9♣ 6♦ 5♥ 5♠', 'flop plus turn')
  assert.equal(board.flop.length, 3)
})

test('HoleCards holds one Two per player, in order', () => {
  const holes = pk.HoleCards.parse(THE_HAND_HOLES)
  assert.equal(holes.length, 2)
  assert.equal(holes.isEmpty(), false)
  assert.equal(holes.get(0).toString(), '6♠ 6♥')
  assert.equal(holes.get(1).toString(), '5♦ 5♣')
  assert.equal(holes.get(9), null, 'past the end is null')
  assert.equal(holes.toArray().length, 2)
})

test('HoleCards can be built one hand at a time', () => {
  const holes = new pk.HoleCards()
  assert.equal(holes.isEmpty(), true)
  holes.push(pk.Two.parse('6♠ 6♥'))
  holes.push(pk.Two.parse('5♦ 5♣'))
  assert.equal(holes.length, 2)
  assert.equal(holes.get(0).first.toString(), '6♠')
  assert.equal(holes.get(0).second.toString(), '6♥')
})

test('Two composes from cards and reports what it holds', () => {
  const two = pk.Two.fromCards(pk.Card.parse('6♠'), pk.Card.parse('6♥'))
  assert.equal(two.toString(), '6♠ 6♥')
  assert.equal(two.containsCard(pk.Card.parse('6♠')), true)
  assert.equal(two.containsCard(pk.Card.parse('A♠')), false)
})

test('board plus hole cards reproduce the pinned eval for THE HAND', () => {
  // Ties item 2b back to the kernel fixture: the same seven cards, assembled
  // from a Board and a Two instead of one flat string.
  const board = pk.Board.parse(THE_HAND_BOARD)
  const hero = pk.HoleCards.parse(THE_HAND_HOLES).get(0)
  const seven = `${hero} ${board.flop} ${board.turn} ${board.river}`
  const evaluation = pk.Eval.fromSeven(seven)
  assert.equal(evaluation.handRank.value, 271)
  assert.equal(evaluation.handRank.class, 'SixesOverFives')
})

test('a malformed board throws with a pkcore error code', () => {
  assert.throws(
    () => pk.Board.parse('9♣ 6♦'),
    (err) => {
      assert.equal(typeof err.code, 'string')
      assert.notEqual(err.code, 'InvalidArg')
      return true
    },
  )
})
