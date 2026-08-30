---
type: Pipeline
title: CI and release pipelines
description: ci.yml runs fmt/clippy/build/test/typecheck plus a generated-bindings drift check; publish.yml builds five platform targets on a v* tag.
resource: https://github.com/ImperialBower/pkcore.js/blob/main/.github/workflows/ci.yml
tags: [ci, github-actions, release]
timestamp: 2026-08-30T00:00:00Z
---

# Overview

Two workflows: `ci.yml` runs on every push/PR to `main`; `publish.yml` runs
only on a `v*` tag.

# `ci.yml`

In order: `cargo fmt --check`, `cargo clippy -D warnings`, `npm run build`,
`npm test`, `npm run typecheck`, then a `git diff --exit-code` on `index.js`
and `index.d.ts`. That last step is the one that catches a signature change
nobody rebuilt — see [generated files](generated-files.md).

The local [Makefile](/tooling/makefile.md)'s `ci` target mirrors this exact
order.

# `publish.yml`

Five build jobs (one per platform target), then one publish job that runs
`napi create-npm-dirs` → `napi artifacts` → `napi pre-publish -t npm` →
`npm publish`.

**Do not add `--skip-optional-publish` to `pre-publish`.** It skips the
per-platform packages, which are the whole point — see
[platform packages](platform-packages.md).

`aarch64-unknown-linux-gnu` builds with `--use-napi-cross`: `pkcore` pulls in
rusqlite and zstd, which compile C, so that leg needs a real cross toolchain
and will not build with plain `cargo --target`.

`npm publish --provenance` requires a **public** repository — it fails from a
private one.

# Citations

[1] [CLAUDE.md — CI and releasing](../../CLAUDE.md)
