---
type: Policy
title: Version lockstep rule
description: pkcore.js's version in Cargo.toml and package.json must always match the pkcore dependency version.
resource: https://github.com/ImperialBower/pkcore.js/blob/main/Cargo.toml
tags: [versioning, policy]
timestamp: 2026-08-30T00:00:00Z
---

# Overview

`pkcore.js`'s own version in `Cargo.toml` **and** `package.json` must always
match the `pkcore` dependency version pinned in `Cargo.toml`. When `pkcore`
is bumped, bump both in the same change.

# How it's enforced

`pkcoreVersion()` reads `CARGO_PKG_VERSION`, and a Node test
(`pkcoreVersion matches the package version`) asserts it equals
`package.json`'s version — a drift fails the suite. The
[Makefile](/tooling/makefile.md)'s `version-check` target checks all three
numbers (`Cargo.toml` package version, `package.json` version, and the
`pkcore` dependency pin) agree without needing a build first, and
`bump-pkcore VERSION=x.y.z` updates all three plus rebuilds in one step.

# Citations

[1] [CLAUDE.md — Version rule](../../CLAUDE.md)
