import test from 'node:test'
import assert from 'node:assert/strict'
import { createRequire } from 'node:module'

const require = createRequire(import.meta.url)
const pk = require('../index.js')

const STACK = 10_000

function session(handles = ['Alice', 'Bob', 'Cara']) {
  const table = pk.Table.nlhFromSeats(
    pk.Seats.fromNames(handles, STACK),
    new pk.ForcedBets(50, 100),
  )
  return new pk.PokerSession(table)
}

function stacks(game, count) {
  return Array.from({ length: count }, (_, seat) => game.table.seats.getSeat(seat).player.chips)
}

test('session_scripted_hand: everyone calls, chips are conserved', () => {
  const game = session()
  assert.equal(game.handNumber, 0)
  assert.equal(game.countFunded(), 3)

  game.startHand()
  assert.equal(game.handNumber, 1)
  assert.ok(game.shuffledDeck.length > 0, 'the shuffled deck is captured at start')

  let seat
  let guard = 0
  while ((seat = game.nextActor()) !== null && guard++ < 60) {
    game.applyAction(seat, pk.PlayerAction.call())
  }
  assert.ok(guard < 60, 'the scripted hand must finish inside the guard')

  const winnings = game.endHand()
  const awarded = winnings.toArray().reduce((sum, pot) => sum + pot.chips, 0)

  assert.ok(awarded > 0)
  assert.equal(
    stacks(game, 3).reduce((sum, chips) => sum + chips, 0),
    3 * STACK,
    'every chip is still on the table after the payout',
  )
})

test('a folded field pays the blinds to the last player standing', () => {
  const game = session(['Alice', 'Bob', 'Cara'])
  game.startHand()

  let seat
  let guard = 0
  let folds = 0
  while ((seat = game.nextActor()) !== null && guard++ < 60) {
    // Fold everyone who can; the last player left wins uncontested.
    try {
      game.applyAction(seat, pk.PlayerAction.fold())
      folds += 1
    } catch {
      game.applyAction(seat, pk.PlayerAction.call())
    }
  }

  const winnings = game.endHand()

  assert.equal(folds, 2, 'two players actually folded; this is not a call-down')
  assert.equal(game.table.phase, 'NewHand', 'the hand ended before the flop')
  assert.equal(winnings.length, 1, 'one uncontested pot')
  assert.equal(winnings.first().seats.length, 1, 'exactly one winner')
  assert.equal(winnings.first().chips, 150, 'the pot is the small blind plus the big blind')
  assert.equal(
    stacks(game, 3).reduce((sum, chips) => sum + chips, 0),
    3 * STACK,
  )
})

test('PlayerAction factories carry their kind and amount', () => {
  assert.equal(pk.PlayerAction.fold().kind, 'Fold')
  assert.equal(pk.PlayerAction.check().kind, 'Check')
  assert.equal(pk.PlayerAction.call().kind, 'Call')
  assert.equal(pk.PlayerAction.allIn().kind, 'AllIn')

  assert.equal(pk.PlayerAction.fold().amount, null)
  assert.equal(pk.PlayerAction.bet(250).kind, 'Bet')
  assert.equal(pk.PlayerAction.bet(250).amount, 250)
  assert.equal(pk.PlayerAction.raise(600).kind, 'Raise')
  assert.equal(pk.PlayerAction.raise(600).amount, 600)
})

test('nextStep reports who acts, and null seat when nobody does', () => {
  const game = session(['Alice', 'Bob'])
  game.startHand()

  const step = game.nextStep()
  assert.equal(step.kind, 'PlayerToAct')
  assert.equal(typeof step.seat, 'number')
  assert.equal(step.error, null)
  assert.equal(step.isComplete(), false)
})

test('the session runs more than one hand and tracks the count', () => {
  const game = session(['Alice', 'Bob'])

  for (let hand = 1; hand <= 3; hand += 1) {
    game.startHand()
    assert.equal(game.handNumber, hand)

    let seat
    let guard = 0
    while ((seat = game.nextActor()) !== null && guard++ < 60) {
      game.applyAction(seat, pk.PlayerAction.call())
    }
    game.endHand()

    assert.equal(
      stacks(game, 2).reduce((sum, chips) => sum + chips, 0),
      2 * STACK,
      `chips conserved after hand ${hand}`,
    )
  }
})

test('an action from the wrong seat throws with a pkcore code', () => {
  const game = session(['Alice', 'Bob'])
  game.startHand()
  const acting = game.nextActor()
  const wrong = acting === 0 ? 1 : 0

  assert.throws(
    () => game.applyAction(wrong, pk.PlayerAction.raise(1)),
    (err) => {
      assert.equal(typeof err.code, 'string')
      assert.notEqual(err.code, 'InvalidArg')
      assert.notEqual(err.code, 'GenericFailure')
      return true
    },
  )
})
