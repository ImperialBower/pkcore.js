---
type: Design Decision
title: Two things deliberately missing
description: PokerSession::run_hand and Winnings::total() are intentionally unbound; do not "fix" either.
resource: https://github.com/ImperialBower/pkcore.js/blob/main/src/lib.rs
tags: [design-decision, napi-rs, safety]
timestamp: 2026-08-30T00:00:00Z
---

# Overview

Two omissions look like bugs but are not. Both come up repeatedly enough that
a future contributor (or agent) may try to "fix" them — don't.

# `PokerSession::run_hand` is not bound

It takes a Rust closure. Calling a JS callback from inside a `&mut self`
method would let that callback re-enter the same session object and alias the
mutable borrow — undefined behaviour, and napi-rs does not guard against it.

JS drives the loop with `startHand` / `nextActor` / `applyAction` / `endHand`
instead. See [driving a hand](driving-a-hand.md) for the actual loop shape.

# `Winnings` has no `total()`

Summing the pots is one line of JS, and all arithmetic belongs in `pkcore` —
see [binding rules](binding-rules.md): no poker logic in this crate,
calculations belong upstream.

# Citations

[1] [CLAUDE.md — Two things that are deliberately missing](../../CLAUDE.md)
