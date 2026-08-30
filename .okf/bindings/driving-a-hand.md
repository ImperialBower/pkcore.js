---
type: Playbook
title: Driving a hand — loop terminators
description: Dealer and PokerSession stop on different signals; using the wrong one is the most common mistake.
tags: [playbook, dealer, session]
timestamp: 2026-08-30T00:00:00Z
---

# Overview

`Dealer` and `PokerSession` stop on different signals when driving a hand to
completion. Picking the wrong loop terminator is the most common mistake made
against this API.

# The right loop

`PokerSession.nextActor()` returns `null` when betting is done, and deals
each street for you automatically. **This is the loop you want.**

# The wrong loop

`Dealer.isHandInProgress()` stays `true` until `endHand()` runs — it is the
**wrong** loop terminator. Use `dealer.table.isGameOver()` instead, which
flips as soon as river betting closes.

# Why this exists

`PokerSession::run_hand` (the Rust-side convenience loop) is deliberately not
bound — see [missing features](missing-features.md) — so JS must assemble the
same loop by hand from `startHand` / `nextActor` / `applyAction` / `endHand`.

# Citations

[1] [CLAUDE.md — Driving a hand](../../CLAUDE.md)
