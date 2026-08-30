---
type: Repository
title: pkcore.js
description: Node.js bindings for the pkcore Rust poker engine, built with napi-rs.
resource: https://github.com/ImperialBower/pkcore.js
tags: [napi-rs, node, rust, bindings, poker]
timestamp: 2026-08-30T00:00:00Z
---

# Overview

pkcore.js wraps [`pkcore`](https://github.com/ImperialBower/pkcore) — a
high-performance poker analysis library written in Rust — for consumption from
Node.js, using [napi-rs](https://napi.rs). It is one of two sibling bindings;
the other is [`pkcore.py`](https://github.com/ImperialBower/pkcore.py) for
Python.

The design contract is **EPIC-85** in the `pkcore` repository
(`docs/epics/EPIC-85_Node_Bindings.md`).

# Naming

Several different names apply on purpose — see [naming](bindings/naming.md).

# What lives here vs. in `pkcore`

This crate contains **no poker logic**. Every `#[napi]` method is a one-line
delegation into a `pkcore` type — see
[binding rules](bindings/binding-rules.md). Anything that needs a calculation
belongs upstream, in `pkcore`.

# Citations

[1] [CLAUDE.md](../CLAUDE.md)
[2] [pkcore](https://github.com/ImperialBower/pkcore)
[3] [pkcore.py](https://github.com/ImperialBower/pkcore.py)
