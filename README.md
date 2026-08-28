# pkcore.js

Node.js bindings for [`pkcore`](https://github.com/ImperialBower/pkcore), a
high-performance poker analysis library written in Rust. Built with
[napi-rs](https://napi.rs), so it is a **native addon**, not WebAssembly: full
`std`, real threads, and the embedded lookup tables all work.

The npm package is `pkcore`. The sibling Python binding is
[`pkcore.py`](https://github.com/ImperialBower/pkcore.py).

## Install

```bash
npm install pkcore
```

## Use

```js
const { Cards, Card, Eval } = require('pkcore')

// THE HAND: Negreanu's 6s6h against Hansen's 5d5c.
const evaluation = Eval.fromSeven('6♠ 6♥ 9♣ 6♦ 5♥ 5♠ 8♠')

console.log(evaluation.handRank.value)  // 271  (lower is stronger)
console.log(evaluation.handRank.name)   // 'FullHouse'
console.log(evaluation.handRank.class)  // 'SixesOverFives'
console.log(evaluation.bestFive.toString()) // '6♠ 6♥ 6♦ 5♠ 5♥'

const deck = Cards.deck()
console.log(deck.length)                 // 52
console.log(deck.contains(Card.parse('A♠'))) // true
```

Build a table:

```js
const { Table, Seats, ForcedBets } = require('pkcore')

const seats = Seats.fromNames(['Alice', 'Bob'], 10_000)
const table = Table.nlhFromSeats(seats, new ForcedBets(50, 100))

console.log(table.seatCount())       // 2
console.log(table.tableChipCount())  // 20000
console.log(table.forced.bigBlind)   // 100
```

Chip counts are plain JS numbers and stay exact: `pkcore` stores them as
`usize`, and this binding maps them through `i64`, so a stack above
4,294,967,295 does not wrap.

TypeScript definitions ship with the package; no `@types` install is needed.

## Errors

A failure throws a normal JS `Error` whose `code` is the `pkcore` error
variant, so you can branch on it:

```js
try {
  Card.parse('ZZ')
} catch (err) {
  console.log(err.code)  // 'InvalidCardIndex'
}
```

## Status

This is an early build. See **EPIC-85** in the `pkcore` repo for the full plan.

| Area | Status |
| --- | --- |
| `Card`, `Cards`, `Rank`, `Suit` | Done |
| `Eval`, `HandRank` | Done |
| `Table`, `Player`, `Seat`, `Seats`, `ForcedBets` | Done |
| `Winnings`, `PotWin`, `SeatEquity` | Done |
| `Board`, `HoleCards` | Planned |
| `Dealer` + event log | Planned |
| `PokerSession` | Planned |
| Prebuilt binaries for all five platforms | Planned |

## Build from source

Needs a Rust toolchain and Node 20 or newer.

```bash
npm install
npm run build      # release build, writes pkcore.<platform>.node
npm test           # node --test
npm run typecheck  # tsc --noEmit against index.d.ts
```

## Licence

MIT OR Apache-2.0, matching `pkcore`.
