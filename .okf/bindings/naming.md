---
type: Convention
title: Naming across pkcore.js
description: Five different names for related things, matching the sibling pkcore.py.
tags: [naming, napi-rs]
timestamp: 2026-08-30T00:00:00Z
---

# Overview

Several different names apply to this project on purpose, mirroring the
naming scheme in [`pkcore.py`](https://github.com/ImperialBower/pkcore.py).
A newcomer expecting one name everywhere will get confused; the split is
intentional, not drift.

# Schema

| Thing | Name |
|-------|------|
| Repository | `pkcore.js` |
| Rust crate | `pkcore-js` |
| npm package | `@imperialbower/pkcore` (`require('@imperialbower/pkcore')`) |
| npm platform packages | `@imperialbower/pkcore-<triple>` |
| Built addon | `pkcore.<platform>.node` |

See [platform packages](/release/platform-packages.md) for why the platform
packages are scoped while the root npm package name is not (they are —
`package.json`'s `name` field is unscoped `pkcore`, but `napi.packageName`
is `@imperialbower/pkcore`; see that concept for the mechanics).

# Citations

[1] [CLAUDE.md — Naming](../../CLAUDE.md)
