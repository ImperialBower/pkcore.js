---
type: Convention
title: Binding rules you would not guess
description: napi-rs constraints specific to this crate — error types, chip-count width, struct shape, and case conversion.
resource: https://github.com/ImperialBower/pkcore.js/blob/main/src/lib.rs
tags: [napi-rs, rust, conventions]
timestamp: 2026-08-30T00:00:00Z
---

# Overview

Every `#[napi]` method here is a one-line delegation into a `pkcore` type — if
a binding needs a calculation, the calculation belongs in `pkcore`, not here.
Beyond that, five specific rules are easy to get wrong because napi-rs's own
docs don't call them out:

# Rules

**Fallible methods must spell out `Result<Self, napi::Error<String>>`
literally.** The `#[napi]` macro pattern-matches the literal `Result<...>`
token to detect a fallible method. A type alias like `PkResult<T>` compiles,
but as a *return class* — and fails at runtime with a confusing
`ObjectFinalize` error, not a compile error.

**`napi::Error<String>` is the only way to set a JS `.code`.** The default
`napi::Error` is `Error<Status>`, which can only report fixed N-API status
names (`InvalidArg`, `GenericFailure`). napi-rs feeds `status.as_ref()` to
`napi_create_error` as the JS error code, so a `String` status is what lets a
thrown error carry the actual `pkcore` error-variant name. EPIC-85 Scope
requires this.

**Chip counts are `i64`, never `u32`.** `pkcore` chip fields are `usize`.
napi-rs maps `i64` to a plain JS `number` (exact below 2^53); an `as u32` cast
wraps silently at 4,294,967,295. Small counts (seat indices, seat counts) stay
`u32`.

**Tuple structs work as `#[napi]` classes.** `pub struct Card(PkCard);` is the
house shape, mirroring `pkcore.py`'s `#[pyclass] pub struct Card(PkCard)`.

**napi-rs converts `snake_case` to `camelCase` automatically.** Do not
hand-write `#[napi(js_name = ...)]` except where JS demands a reserved shape,
such as `toString`.

# Citations

[1] [CLAUDE.md — Binding rules you would not guess](../../CLAUDE.md)
