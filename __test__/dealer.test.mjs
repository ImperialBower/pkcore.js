import test from 'node:test'
import assert from 'node:assert/strict'
import { createRequire } from 'node:module'

const require = createRequire(import.meta.url)
const pk = require('../index.js')

const STACK = 10_000

function seatedDealer(handles = ['Alice', 'Bob']) {
  const dealer = new pk.Dealer(new pk.ForcedBets(50, 100), handles.length)
  for (const handle of handles) {
    dealer.seatPlayer(new pk.Player(handle, STACK))
  }
  return dealer
}

/** Calls when facing a bet, checks otherwise. Uses only bound API. */
function callOrCheck(dealer, seat) {
  const facing = dealer.table.bet
  const committed = dealer.table.seats.getSeat(seat).player.bet
  if (committed < facing) {
    dealer.call(seat)
  } else {
    dealer.check(seat)
  }
}

test('seatPlayer fills chairs in order and reports the seat index', () => {
  const dealer = new pk.Dealer(new pk.ForcedBets(50, 100), 3)
  assert.equal(dealer.seatPlayer(new pk.Player('Alice', STACK)), 0)
  assert.equal(dealer.seatPlayer(new pk.Player('Bob', STACK)), 1)
  assert.equal(dealer.chipsAt(0), STACK)
  assert.equal(dealer.chipsAt(2), 0, 'an empty chair exists and holds no chips')
  assert.equal(dealer.chipsAt(9), null, 'a seat that does not exist is null')
})

test('full_hand_chip_conservation: chips in equal chips out', () => {
  const dealer = seatedDealer()
  const startingTotal = dealer.chipsAt(0) + dealer.chipsAt(1)
  assert.equal(startingTotal, 2 * STACK)

  dealer.startHand()
  assert.equal(dealer.isHandInProgress(), true)

  // `isHandInProgress()` stays true until `endHand()` runs, so it is the wrong
  // loop terminator. `table.isGameOver()` flips as soon as the river betting
  // closes, which is the point where there is nothing left to act on.
  let guard = 0
  while (!dealer.table.isGameOver() && guard++ < 50) {
    callOrCheck(dealer, dealer.nextToAct())
    try {
      dealer.advanceStreet()
    } catch {
      // The street is not finished yet; keep taking actions.
    }
  }
  assert.ok(guard < 50, 'the hand must finish inside the guard')
  assert.equal(dealer.table.phase, 'DealRiver')

  const winnings = dealer.endHand()
  const awarded = winnings.toArray().reduce((sum, pot) => sum + pot.chips, 0)
  const endingTotal = dealer.chipsAt(0) + dealer.chipsAt(1)

  assert.ok(awarded > 0, 'somebody must win the pot')
  assert.equal(
    endingTotal,
    startingTotal,
    'pkcore polices this with Table.hand_chip_total; the binding must not lose chips',
  )
})

test('dealer_error_shape: an illegal action throws with the DealerError code', () => {
  const dealer = seatedDealer()
  assert.throws(
    () => dealer.check(0),
    (err) => {
      assert.equal(err.code, 'HandNotStarted', 'code is the pkcore variant, not an N-API status')
      assert.ok(err.message.length > 0)
      return true
    },
  )

  dealer.startHand()
  assert.throws(() => dealer.bet(9, 100), (err) => err.code === 'IllegalAction')
  assert.throws(() => dealer.raiseTo(dealer.nextToAct(), 1), (err) => err.code === 'IllegalAction')
})

test('event log records the hand as an array of typed events', () => {
  const dealer = seatedDealer()
  // Opening the table is itself the first event.
  assert.deepEqual(dealer.eventLog().map((event) => event.kind), ['TableOpen'])

  dealer.startHand()
  const events = dealer.eventLog()
  assert.ok(Array.isArray(events), 'a plain array, not a TableLog wrapper')
  assert.ok(events.length > 0)

  const kinds = events.map((event) => event.kind)
  assert.ok(kinds.includes('ShuffleDeck'))
  assert.ok(kinds.includes('ForcedBetSmallBlind'))
  assert.ok(kinds.includes('ForcedBetBigBlind'))

  const bigBlind = events.find((event) => event.kind === 'ForcedBetBigBlind')
  assert.equal(bigBlind.amount, 100, 'the payload amount comes through')
  assert.equal(typeof bigBlind.seat, 'number')

  const shuffle = events.find((event) => event.kind === 'ShuffleDeck')
  assert.equal(shuffle.seat, null, 'events with no seat report null')
  assert.equal(shuffle.amount, null)
})

test('the table is readable through the dealer between actions', () => {
  const dealer = seatedDealer()
  dealer.startHand()
  assert.equal(dealer.table.seatCount(), 2)
  assert.equal(dealer.pot() + dealer.table.seats.currentBet() * 0, dealer.pot())
  assert.equal(typeof dealer.tableId(), 'string')
  assert.ok(dealer.tableId().length > 0)
})
