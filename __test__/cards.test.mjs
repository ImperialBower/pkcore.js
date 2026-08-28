import test from 'node:test'
import assert from 'node:assert/strict'
import { createRequire } from 'node:module'

const require = createRequire(import.meta.url)
const pk = require('../index.js')

// THE HAND: Negreanu 6s6h vs Hansen 5d5c, board 9c 6d 5h 5s 8s.
// Mirrors pkcore's own `TestData::the_hand` fixture.
const THE_HAND_SEVEN = '6♠ 6♥ 9♣ 6♦ 5♥ 5♠ 8♠'

test('card_round_trip: a Cards string survives parse -> toString', () => {
  const cards = pk.Cards.parse(THE_HAND_SEVEN)
  assert.equal(cards.toString(), THE_HAND_SEVEN)
  assert.equal(cards.length, 7)
  assert.ok(cards.areUnique())
})

test('single card round-trips and exposes rank and suit', () => {
  const card = pk.Card.parse('A♠')
  assert.equal(card.toString(), 'A♠')
  assert.equal(card.rank.toString(), 'A')
  assert.equal(card.suit.symbol(), '♠')
  assert.equal(card.suit.letter(), 'S')
  assert.ok(card.isDealt())
})

test('Card.fromRankSuit composes the same card as parse', () => {
  const built = pk.Card.fromRankSuit(pk.Rank.ace(), pk.Suit.spades())
  assert.ok(built.equals(pk.Card.parse('A♠')))
  assert.equal(built.asU32(), pk.Card.parse('As').asU32())
})

test('Rank exposes the Cactus Kev numbers pkcore computes', () => {
  const ace = pk.Rank.ace()
  assert.equal(ace.number(), 12)
  assert.equal(pk.Rank.deuce().number(), 0)
  assert.ok(ace.prime() > 0)
  assert.ok(ace.bits() > 0)
})

test('Cards.deck is a full 52-card deck of unique cards', () => {
  const deck = pk.Cards.deck()
  assert.equal(deck.length, 52)
  assert.ok(deck.areUnique())
  assert.ok(deck.contains(pk.Card.parse('A♠')))
  assert.equal(deck.toArray().length, 52)
})

test('parse_error_shape: a bad card throws with a pkcore error code', () => {
  // EPIC-85 Scope requires a structured `.code`, not a debug string. The
  // Dealer version of this lands in Phase 3 item 5.
  assert.throws(
    () => pk.Card.parse('ZZ'),
    (err) => {
      assert.ok(err instanceof Error)
      assert.equal(typeof err.code, 'string')
      assert.notEqual(err.code, 'InvalidArg', 'code must be the pkcore variant, not the N-API status')
      assert.ok(err.code.length > 0)
      return true
    },
  )
})

test('pkcoreVersion matches the package version (lockstep rule)', () => {
  const pkg = require('../package.json')
  assert.equal(pk.pkcoreVersion(), pkg.version)
})
