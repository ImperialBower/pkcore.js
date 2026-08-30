---
type: Convention
title: Generated files are committed
description: index.js and index.d.ts are produced by napi build and committed; CI fails the build if they drift from a fresh rebuild.
resource: https://github.com/ImperialBower/pkcore.js/blob/main/index.js
tags: [napi-rs, ci, convention]
timestamp: 2026-08-30T00:00:00Z
---

# Overview

`index.js` and `index.d.ts` are produced by `napi build` and **are
committed**. Re-run `npm run build` after any `#[napi]` signature change, or
the checked-in types drift from the addon.

`*.node` binaries are **not** committed — they are gitignored, and
`napi artifacts` (run during a release, or by hand) copies `.node` files into
the repo root as a side effect; delete stray ones or a stub can shadow a real
build.

# How it's enforced

`ci.yml`'s last step runs `git diff --exit-code -- index.js index.d.ts`
after a fresh build — this is what catches a signature change nobody
rebuilt, surfacing it as a dirty tree rather than as wrong types shipped to
npm. See [CI/CD](ci-cd.md). The local [Makefile](/tooling/makefile.md) has
the same step as `check-bindings`.

# Citations

[1] [CLAUDE.md — Generated files](../../CLAUDE.md)
