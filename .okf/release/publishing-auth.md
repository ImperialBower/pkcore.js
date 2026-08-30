---
type: Design Decision
title: Publishing auth — migrate to OIDC before January 2027
description: npm removes direct publishing from 2FA-bypass tokens around January 2027; the migration path is publish once with NPM_TOKEN, set trusted publishing per package, then delete the token.
resource: https://github.com/ImperialBower/pkcore.js/blob/main/.github/workflows/publish.yml
tags: [npm, oidc, security, decision]
timestamp: 2026-08-30T00:00:00Z
---

# Overview

`publish.yml` currently authenticates with an `NPM_TOKEN` secret (a granular
access token). npm is removing direct publishing from 2FA-bypass tokens
around **January 2027** — after that, the token can only stage a publish for
a human to approve, which does not work unattended.

# Migration path, in order

1. Publish once with the token. This is what first creates the six packages
   (`pkcore` plus the five `pkcore-<triple>` platform packages).
2. On npmjs.com, set a trusted publisher on **each of the six**: repository
   `ImperialBower/pkcore.js`, workflow `publish.yml`. Trusted publishing is
   configured per package, which is why step 1 has to happen first — unless
   npm has since added pre-registration for packages that don't exist yet, in
   which case skip straight to step 2.
3. Delete the `NPM_TOKEN` secret and drop `NODE_AUTH_TOKEN` from
   `publish.yml`. The `id-token: write` permission is already there, and
   `--provenance` becomes automatic under OIDC.

# Related, already in effect (since August 2026)

A 2FA-bypass token can no longer perform account, package, or organization
management. Create and rotate tokens interactively on the website with 2FA;
it cannot be scripted.

`pkcore.py` already uses PyPI trusted publishing
(`pkcore.py/.github/workflows/publish.yml`, `permissions: id-token: write`),
so this migration ends with both bindings on the same OIDC model.

# Citations

[1] [CLAUDE.md — Publishing auth: migrate to OIDC before January 2027](../../CLAUDE.md)
