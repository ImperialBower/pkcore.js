// demo.mjs — pkcore.js feature showcase
//
// Run with:
//     node demo.mjs
//
// Mirrors pkcore.py's demo.py, section for section, using this binding's API.

import { createRequire } from 'node:module'

const require = createRequire(import.meta.url)
const pk = require('./index.js')

const SEP = '-'.repeat(60)
const section = (title) => console.log(`\n${SEP}\n${title}\n${SEP}`)

// ============================================================
// Rank and Suit
// ============================================================

section('Rank and Suit')

const ranks = [
  pk.Rank.ace(), pk.Rank.king(), pk.Rank.queen(), pk.Rank.jack(),
  pk.Rank.ten(), pk.Rank.deuce(),
]
console.log('Ranks (PyO3 class attributes become static factories here):')
for (const rank of ranks) {
  console.log(
    `  ${String(rank).padEnd(3)}  value=${String(rank.value).padStart(3)}` +
    `  index=${String(rank.number()).padStart(2)}  prime=${rank.prime()}`,
  )
}

console.log('\nSuits:')
for (const suit of [pk.Suit.spades(), pk.Suit.hearts(), pk.Suit.diamonds(), pk.Suit.clubs()]) {
  console.log(`  ${suit.symbol()}  letter=${suit.letter()}  value=${suit.value}`)
}

// ============================================================
// Card and Cards
// ============================================================

section('Card and Cards')

const ace = pk.Card.parse('A♠')
console.log(`Parsed "A♠"      -> ${ace}  rank=${ace.rank}  suit=${ace.suit.symbol()}`)
console.log(`Built from parts -> ${pk.Card.fromRankSuit(pk.Rank.ace(), pk.Suit.spades())}`)
console.log(`Cactus Kev value -> ${ace.asU32()}`)

const deck = pk.Cards.deck()
console.log(`\nA full deck has ${deck.length} cards, all unique: ${deck.areUnique()}`)
console.log(`Contains A♠: ${deck.contains(ace)}`)

// Errors carry a pkcore error code, not an opaque string.
try {
  pk.Card.parse('ZZ')
} catch (err) {
  console.log(`\nBad parse throws  -> code=${err.code}`)
}

// ============================================================
// Evaluating a hand
// ============================================================

section('Evaluating THE HAND')

// Negreanu's 6♠6♥ against Hansen's 5♦5♣, board 9♣ 6♦ 5♥ 5♠ 8♠.
const board = pk.Board.parse('9♣ 6♦ 5♥ 5♠ 8♠')
const holes = pk.HoleCards.parse('6♠ 6♥ 5♦ 5♣')

console.log(`Board: flop ${board.flop}  turn ${board.turn}  river ${board.river}`)

for (let player = 0; player < holes.length; player += 1) {
  const hole = holes.get(player)
  const seven = `${hole} ${board.flop} ${board.turn} ${board.river}`
  const result = pk.Eval.fromSeven(seven)
  console.log(
    `  ${String(hole).padEnd(6)} -> ${String(result.handRank.class).padEnd(18)}` +
    ` rank=${String(result.handRank.value).padStart(4)}  best five: ${result.bestFive}`,
  )
}
console.log('\nLower rank value wins: Hansen\'s quad fives (124) beat')
console.log('Negreanu\'s sixes full (271). One of the most famous hands ever played.')

// ============================================================
// Building a table
// ============================================================

section('Building a table')

const seats = pk.Seats.fromNames(['Alice', 'Bob', 'Cara'], 10_000)
const table = pk.Table.nlhFromSeats(seats, new pk.ForcedBets(50, 100))

console.log(`${table.name}`)
console.log(`  seats:  ${table.seatCount()}`)
console.log(`  blinds: ${table.forced.smallBlind} / ${table.forced.bigBlind}`)
console.log(`  chips:  ${table.tableChipCount()}`)

// ============================================================
// Playing a hand
// ============================================================

section('Playing a hand')

const session = new pk.PokerSession(table)
session.startHand()
console.log(`Hand #${session.handNumber} dealt. Everybody calls it down.\n`)

let seat
let actions = 0
while ((seat = session.nextActor()) !== null) {
  const handle = session.table.seats.getSeat(seat).player.handle
  session.applyAction(seat, pk.PlayerAction.call())
  console.log(`  seat ${seat} (${handle}) calls`)
  actions += 1
}

const winnings = session.endHand()

console.log(`\n${actions} actions taken. Pots awarded: ${winnings.length}`)
for (const pot of winnings.toArray()) {
  const names = pot.seats.map((index) => seats.getSeat(index).player.handle)
  console.log(`  ${pot.chips} chips to ${names.join(' and ')} (${pot.handRank.class})`)
}

const finalStacks = [0, 1, 2].map((index) => session.table.seats.getSeat(index).player)
console.log('\nFinal stacks:')
for (const player of finalStacks) {
  console.log(`  ${player.handle.padEnd(6)} ${player.chips}`)
}

const total = finalStacks.reduce((sum, player) => sum + player.chips, 0)
console.log(`\nChips in: 30000   chips out: ${total}   conserved: ${total === 30_000}`)

// ============================================================
// The event log
// ============================================================

section('The event log')

const dealer = new pk.Dealer(new pk.ForcedBets(50, 100), 2)
dealer.seatPlayer(new pk.Player('Alice', 10_000))
dealer.seatPlayer(new pk.Player('Bob', 10_000))
dealer.startHand()

console.log('Every event pkcore recorded while starting the hand:\n')
for (const event of dealer.eventLog()) {
  const seatText = event.seat === null ? '   ' : `s${event.seat} `
  const amountText = event.amount === null ? '' : ` ${event.amount}`
  console.log(`  ${seatText} ${event.kind}${amountText}`)
}

console.log(`\nVersion: pkcore ${pk.pkcoreVersion()}`)
