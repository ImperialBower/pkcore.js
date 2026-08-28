import test from 'node:test'
import assert from 'node:assert/strict'
import { createRequire } from 'node:module'

const require = createRequire(import.meta.url)
const pk = require('../index.js')

const STACK = 10_000
const BLINDS = () => new pk.ForcedBets(50, 100)

function headsUpTable() {
  return pk.Table.nlhFromSeats(pk.Seats.fromNames(['Alice', 'Bob'], STACK), BLINDS())
}

test('heads_up_table_seat_count: two seats in, two seats out', () => {
  const table = headsUpTable()
  assert.equal(table.seatCount(), 2)
  assert.equal(table.seats.size, 2)
  assert.equal(table.countOccupiedSeats(), 2)
})

test('a fresh table holds every chip and nothing in the pot', () => {
  const table = headsUpTable()
  assert.equal(table.tableChipCount(), 2 * STACK)
  assert.equal(table.pot, 0)
  assert.equal(table.bet, 0)
  assert.equal(table.button, 0)
  assert.equal(table.board.toString(), '')
  assert.equal(table.phase, 'NewHand')
  assert.equal(table.isPreflop(), true)
  assert.equal(table.isGameOver(), false)
  assert.equal(table.minRaise(), 100, 'min raise is the big blind before any action')
  assert.equal(table.name, "No Limit Hold'em Table")
})

test('ForcedBets carries blinds through to the table', () => {
  const table = headsUpTable()
  assert.equal(table.forced.smallBlind, 50)
  assert.equal(table.forced.bigBlind, 100)
  assert.equal(table.forced.ante, 0)
  assert.equal(pk.ForcedBets.withAnte(50, 100, 25).ante, 25)
})

test('Seats can also be built one chair at a time', () => {
  const seats = new pk.Seats()
  seats.push(new pk.Seat(new pk.Player('Alice', STACK)))
  seats.push(new pk.Seat(new pk.Player('Bob', STACK)))
  const table = pk.Table.nlhFromSeats(seats, BLINDS())
  assert.equal(table.seatCount(), 2)
  assert.equal(seats.totalChipCount(), 2 * STACK)
})

test('Player exposes its stack as an exact JS number, not a truncated u32', () => {
  // EPIC-85 Scope: usize -> i64 -> number. An `as u32` cast would wrap here.
  const big = 8_000_000_000 // > u32::MAX (4,294,967,295)
  const player = new pk.Player('Whale', big)
  assert.equal(player.chips, big)
  assert.equal(player.totalChipCount(), big)
  assert.ok(Number.isSafeInteger(player.chips))
})

test('a negative stack floors at zero instead of wrapping', () => {
  assert.equal(new pk.Player('Broke', -5).chips, 0)
})

test('Player state and flags read back', () => {
  const player = new pk.Player('Alice', STACK)
  assert.equal(player.handle, 'Alice')
  assert.equal(player.bet, 0)
  assert.equal(player.chipsInPlay, 0)
  assert.equal(typeof player.state, 'string')
  assert.equal(player.hasBet(), false)
})

test('reload adds to the stack and to withdrawn', () => {
  const player = new pk.Player('Alice', 1_000)
  const after = player.reload(500)
  assert.equal(after, player.chips)
  assert.equal(player.chips, 1_500)
  assert.equal(player.withdrawn, 1_500)
})

test('getSeat returns the chair, and out-of-range returns null', () => {
  const table = headsUpTable()
  assert.equal(table.seats.getSeat(0).player.handle, 'Alice')
  assert.equal(table.seats.getSeat(1).player.handle, 'Bob')
  assert.equal(table.seats.getSeat(9), null)
})

test('Winnings and PotWin classes are exported and shaped as documented', () => {
  // A populated Winnings comes from Dealer.endHand in Phase 3; here we only
  // pin the surface so the Phase 3 test has something to assert against.
  assert.equal(typeof pk.Winnings, 'function')
  assert.equal(typeof pk.PotWin, 'function')
  assert.equal(typeof pk.SeatEquity, 'function')
})
