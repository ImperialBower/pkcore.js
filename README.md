# pkcore.js

[![npm](https://img.shields.io/npm/v/@imperialbower/pkcore.svg)](https://www.npmjs.com/package/@imperialbower/pkcore)
[![CI](https://github.com/ImperialBower/pkcore.js/actions/workflows/ci.yml/badge.svg)](https://github.com/ImperialBower/pkcore.js/actions/workflows/ci.yml)
[![node](https://img.shields.io/node/v/@imperialbower/pkcore.svg)](https://nodejs.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE-MIT)
[![License: Apache 2.0](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](LICENSE-APACHE)
[![AI BOM](https://img.shields.io/badge/AI--BOM-tracked-blueviolet)](./AI-BOM.md)

Node.js bindings for [`pkcore`](https://github.com/ImperialBower/pkcore), a
high-performance poker analysis library written in Rust. Built with
[napi-rs](https://napi.rs), so it is a **native addon**, not WebAssembly: full
`std`, real threads, and the embedded lookup tables all work.

The npm package is `@imperialbower/pkcore`. The unscoped name `pkcore` is
blocked by npm as too similar to an existing `pk-core`. The sibling Python
binding is [`pkcore.py`](https://github.com/ImperialBower/pkcore.py).

## Install

```bash
npm install @imperialbower/pkcore
```

## Use

```js
const { Cards, Card, Eval } = require('@imperialbower/pkcore')

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
const { Table, Seats, ForcedBets } = require('@imperialbower/pkcore')

const seats = Seats.fromNames(['Alice', 'Bob'], 10_000)
const table = Table.nlhFromSeats(seats, new ForcedBets(50, 100))

console.log(table.seatCount())       // 2
console.log(table.tableChipCount())  // 20000
console.log(table.forced.bigBlind)   // 100
```

Chip counts are plain JS numbers and stay exact: `pkcore` stores them as
`usize`, and this binding maps them through `i64`, so a stack above
4,294,967,295 does not wrap.

Play a hand:

```js
const { Table, Seats, ForcedBets, PokerSession, PlayerAction } = require('@imperialbower/pkcore')

const table = Table.nlhFromSeats(
  Seats.fromNames(['Alice', 'Bob', 'Cara'], 10_000),
  new ForcedBets(50, 100),
)
const session = new PokerSession(table)

session.startHand()

let seat
while ((seat = session.nextActor()) !== null) {
  session.applyAction(seat, PlayerAction.call())   // everybody calls
}

const winnings = session.endHand()
console.log(winnings.first().chips)  // pot size
console.log(winnings.first().seats)  // [1]  -- winning seat indices
```

`session.nextActor()` deals each street for you and returns `null` when the
betting is finished. `Dealer` is the lower-level alternative if you want to
drive the streets yourself.

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
| `Dealer` + `TableAction` event log | Done |
| `PokerSession`, `PlayerAction`, `SessionStep` | Done |
| `Board`, `HoleCards`, `Two` | Done |
| Prebuilt binaries for all five platforms | CI wired; not published yet |

## Try it

```bash
npm run demo
```

`demo.mjs` walks the whole surface: ranks and suits, parsing cards, evaluating
THE HAND (Negreanu vs Hansen), building a table, playing a hand to showdown, and
dumping the event log.

## Build from source

Needs a Rust toolchain and Node 20 or newer.

```bash
npm install
npm run build      # release build, writes pkcore.<platform>.node
npm test           # node --test
npm run typecheck  # tsc --noEmit against index.d.ts
npm run demo       # the showcase script
```

## Releasing

Push a `v*` tag. `.github/workflows/publish.yml` cross-compiles all five
targets, assembles the per-platform npm packages, and publishes them plus the
root `pkcore` package.

The five platform binaries publish as `@imperialbower/pkcore-<triple>`; the
package you install stays plain `pkcore`.

It needs three things:

- a **public** repository — `npm publish --provenance` will not run from a
  private one;
- an `NPM_TOKEN` repository secret;
- a repository environment named `npm`.

`napi artifacts` fails if any target is missing, so a broken matrix leg stops
the release instead of shipping a partial set.

**Token auth has a deadline.** npm removes direct publishing from access tokens
around January 2027. After the first release creates the six packages, switch
each to trusted publishing (OIDC) and delete the token. See `CLAUDE.md`.

## Install-script free

`npm install @imperialbower/pkcore` runs no lifecycle scripts and needs no `--allow-scripts`
flag under npm v12's install-time defaults. The platform binary arrives through
`optionalDependencies`, not a postinstall build.

## Licence

MIT OR Apache-2.0, matching `pkcore`.
