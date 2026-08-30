---
type: Design Decision
title: Platform packages are scoped; the root package is not
description: package.json's name stays unscoped "pkcore" while napi.packageName is the scoped @imperialbower/pkcore, because npm's spam heuristic rejected unscoped platform package names.
resource: https://github.com/ImperialBower/pkcore.js/blob/main/package.json
tags: [npm, napi-rs, decision]
timestamp: 2026-08-30T00:00:00Z
---

# Overview

`package.json` sets `napi.packageName` to `@imperialbower/pkcore` while the
package `name` field stays `pkcore`. That split is deliberate:

* users still run `npm install @imperialbower/pkcore`;
* the five platform packages publish as `@imperialbower/pkcore-<triple>`.

**Do not un-scope them.**

# Why

The first release attempt used unscoped names, and npm rejected
`pkcore-win32-x64-msvc` with `403 Package name triggered spam detection`. The
other four platform packages went through; a later retry by hand with
interactive 2FA hit the identical 403. So the cause is the name, not a rate
limit and not the token — no colliding package exists, npm's heuristic on
unscoped names is simply opaque. A scope proves ownership and skips the
check, which is why the rest of the napi-rs ecosystem scopes platform
packages too (`@swc/core-win32-x64-msvc`, `@napi-rs/canvas-win32-x64-msvc`).

Changing `napi.packageName` rewrites the `require` calls in the generated
`index.js` — rebuild and commit `index.js` after any change to it. See
[generated files](generated-files.md).

# Orphaned packages

Four orphan unscoped packages exist at `0.9.1` from the failed first attempt:
`pkcore-darwin-arm64`, `-darwin-x64`, `-linux-x64-gnu`, `-linux-arm64-gnu`.
Nothing references them; deprecate them rather than reuse them.

# Citations

[1] [CLAUDE.md — Platform packages are scoped; the root package is not](../../CLAUDE.md)
