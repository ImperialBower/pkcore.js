---
type: Tooling
title: Makefile
description: make (ayce) runs a full clean build/test/lint sweep before push; make ci mirrors ci.yml exactly; make bump-pkcore automates the version-lockstep upgrade.
resource: https://github.com/ImperialBower/pkcore.js/blob/main/Makefile
tags: [make, ci, tooling]
timestamp: 2026-08-30T00:00:00Z
---

# Overview

Scoped to what this bindings repo actually needs — no WASM checks, purity
gate, or perf harness, since those belong to the `pkcore` kernel repo, not
here.

# Key targets

`make` (default, `ayce`) — clean, format, build, test, typecheck, clippy, and
every repo-specific check, in that order. Slower than `ci` on purpose:
`clean` forces a from-scratch compile instead of reusing the incremental
cache.

`make ci` — mirrors [`ci.yml`](/release/ci-cd.md) exactly, in the same order,
so a local pass predicts a CI pass: `fmt-check clippy build test typecheck
check-bindings`.

`make check-bindings` — the [generated-files](/release/generated-files.md)
drift check.

`make check-scripts` — the [install-scripts policy](/release/install-scripts-policy.md)
check.

`make version-check` — the [version lockstep](/release/version-lockstep.md)
check.

`make bump-pkcore VERSION=x.y.z` — edits `Cargo.toml`'s crate version and the
`pkcore` dependency pin, `package.json`'s version, runs `cargo update -p
pkcore --precise VERSION`, and rebuilds — the whole
[version lockstep](/release/version-lockstep.md) upgrade in one command.

# Citations

[1] [Makefile](../../Makefile)
