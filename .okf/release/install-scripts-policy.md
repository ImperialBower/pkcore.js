---
type: Policy
title: Keep this package free of install scripts
description: Zero install hooks of our own, and zero across all lockfile packages; do not add an install hook, a git dependency, or a remote-URL dependency.
resource: https://github.com/ImperialBower/pkcore.js/blob/main/package-lock.json
tags: [npm, supply-chain, policy]
timestamp: 2026-08-30T00:00:00Z
---

# Overview

npm v12 turned install-time lifecycle scripts **off by default**: a package
with `preinstall`/`install`/`postinstall` now needs the consumer to run
`npm approve-scripts` or pass `--allow-scripts`. Git and remote-URL
dependencies are likewise opt-in.

# The rule

**Do not add an install hook, a git dependency, or a remote-URL dependency.**
Any one of them turns `npm install @imperialbower/pkcore` into a flagged
install for every downstream user.

As of 2026-08-28 this package needs none of that — zero install hooks of our
own, and zero across every package in the lockfile. That is the point of
napi-rs's prebuilt-binary model: the platform addon arrives through
`optionalDependencies`, not a postinstall build.

# How it's checked

`npm`'s lockfile flags any dependency that declares an install script with
`"hasInstallScript": true`. The [Makefile](/tooling/makefile.md)'s
`check-scripts` target greps `package-lock.json` for that marker and fails
loud if it ever appears.

# Citations

[1] [CLAUDE.md — Keep this package free of install scripts](../../CLAUDE.md)
