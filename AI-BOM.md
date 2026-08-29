# AI Bill of Materials — pkcore.js

_Last updated: 2026-08-28 · pkcore.js v0.9.1 · npm `@imperialbower/pkcore`_

An inventory of every AI component connected to this repository — the tools used
to author it, the machine-readable context those tools consume, and the AI
algorithms reachable through the published npm package. Modeled on the SBOM
concept applied to AI systems, and a companion to
[`pkcore`'s AI-BOM](https://github.com/ImperialBower/pkcore/blob/main/AI-BOM.md).

**Scope note.** This repository is a thin napi-rs binding layer. It contains no
poker logic and no AI algorithms of its own — every `#[napi]` method is a
one-line delegation into the `pkcore` crate. For the algorithm inventory, read
`pkcore`'s AI-BOM; §4 below records only what this package actually exposes to
JavaScript.

---

## 1. Development Tools

AI tools used to author this codebase. Not part of the shipped package, but
relevant to provenance.

| Tool | Vendor | Role | Notes |
|------|--------|------|-------|
| Claude Code | Anthropic | Primary AI coding assistant | The entire repository, from the initial commit (2026-08-27) through EPIC-85 Phase 4, was authored in Claude Code sessions; see [`CLAUDE.md`](./CLAUDE.md) |

The design contract this code implements — **EPIC-85** — lives in the `pkcore`
repository at `docs/epics/EPIC-85_Node_Bindings.md`, not here.

---

## 2. AI Context Infrastructure

Machine-readable knowledge that AI development tools consume when working on this
repository.

| Component | Format | In published npm package? | Notes |
|-----------|--------|---------------------------|-------|
| [`CLAUDE.md`](./CLAUDE.md) | Markdown instructions | **No** | Binding rules, the `Result<Self, napi::Error<String>>` requirement, the `i64` chip-count rule, CI/publish contract |
| [`AI-BOM.md`](./AI-BOM.md) | Markdown inventory | **Yes** — listed in `package.json`'s `files` | This file |
| OKF bundle | — | — | None in this repository. `pkcore` carries the shared 27-concept bundle |

`package.json` uses an explicit `files` allowlist — `index.js`, `index.d.ts`,
`AI-BOM.md`, `LICENSE-MIT`, `LICENSE-APACHE` — plus `README.md` and
`package.json`, which npm always includes. This file therefore travels in the npm
tarball, so a consumer can read the AI provenance without visiting GitHub.
`CLAUDE.md` stays out: it is instructions for authoring the bindings, not
information about what ships.

---

## 3. AI Audits

No AI code audit has been performed on this repository yet. `pkcore`'s three
model audits (Claude Sonnet 4.6, GPT-5.4, Gemini 3.1 — all at pkcore v0.0.40)
cover the poker engine underneath, not this binding layer.

---

## 4. AI Surface Exposed to JavaScript

What of `pkcore`'s AI machinery a JS consumer can reach through this package
today.

| `pkcore` component | Bound here? | Notes |
|--------------------|-------------|-------|
| CFR / CFR+ / DCFR solvers (`analysis::gto`) | **No** | Not yet bound |
| `BotDecider` trait and deciders (`bot::*`) | **No** | Not yet bound |
| `ExploitTrainer`, `SimTable` | **No** | Not yet bound |
| `PlayerStats` / `StatsRegistry` | **No** | Not yet bound |
| Card, evaluation, table, dealer, session types | Yes | 22 `#[napi]` classes; see [`index.d.ts`](./index.d.ts) |

`PokerSession::run_hand` is deliberately **not** bound: it takes a Rust closure,
and re-entering a `&mut self` method from a JS callback would alias the mutable
borrow. JS drives the loop with `startHand` / `nextActor` / `applyAction` /
`endHand` instead. See [`CLAUDE.md`](./CLAUDE.md).

Any future bot or solver binding must add a row here.

---

## 5. External AI Integrations

This package ships **zero** external AI service dependencies and makes no network
calls. The full dependency tree is 118 npm packages plus the Rust crates in
[`Cargo.lock`](./Cargo.lock); none of them is an AI or ML client.

Planned LLM agent clients (Anthropic Claude, OpenAI, Ollama) and LLM
observability (Langfuse) live in the `pkdealer` repository under EPIC-23 and
EPIC-24. If they ever become reachable from Node, they arrive through a `pkcore`
binding and get a row here first.

---

## 6. Supply-Chain Posture

Relevant to AI provenance because a native addon is opaque to a consumer's
auditing tools.

| Property | State |
|----------|-------|
| Install-time lifecycle scripts | **None** — zero across all 118 lockfile packages |
| Git or remote-URL dependencies | **None** |
| Binary delivery | Prebuilt `.node` addons via `optionalDependencies`, not a postinstall build |
| Build provenance | `npm publish --provenance` under GitHub Actions (`publish.yml`), on a `v*` tag |
| Generated bindings | `index.js` and `index.d.ts` are committed, and CI fails on drift via `git diff --exit-code` |

npm v12 turned install-time scripts off by default. Keeping this package free of
them means `npm install @imperialbower/pkcore` never prompts a downstream user
for `npm approve-scripts`.

---

## 7. References

| Document | Purpose |
|----------|---------|
| [`pkcore` AI-BOM](https://github.com/ImperialBower/pkcore/blob/main/AI-BOM.md) | Algorithm, agent, and integration inventory for the engine |
| [`CLAUDE.md`](./CLAUDE.md) | Binding rules, CI and publishing contract |
| [`README.md`](./README.md) | Install and usage |
| EPIC-85 (`pkcore` repo, `docs/epics/EPIC-85_Node_Bindings.md`) | Design contract for these bindings |
| [`pkcore.py`](https://github.com/ImperialBower/pkcore.py) | Sibling Python binding |
