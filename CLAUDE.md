# Claude Instructions for pkcore.js

Node.js bindings for [`pkcore`](https://github.com/ImperialBower/pkcore), built
with [napi-rs](https://napi.rs). The design contract is **EPIC-85** in the
`pkcore` repo (`docs/epics/EPIC-85_Node_Bindings.md`). Read it before adding a
binding.

Everything here is what the repo cannot tell you on its own.

## Naming

Three different names, on purpose, matching the sibling `pkcore.py`:

| Thing | Name |
| --- | --- |
| Repository | `pkcore.js` |
| Rust crate | `pkcore-js` |
| npm package | `pkcore` (`require('pkcore')`) |
| Built addon | `pkcore.<platform>.node` |

## Version rule

`pkcore.js`'s version in `Cargo.toml` **and** `package.json` must always match
the `pkcore` dependency version. When `pkcore` is bumped, bump both in the same
change. `pkcoreVersion()` reads `CARGO_PKG_VERSION` and a test asserts it equals
`package.json`'s version, so a drift fails the suite.

## Binding rules you would not guess

- **No poker logic in this crate.** Every method is a one-line delegation to a
  `pkcore` type. If a binding needs a calculation, the calculation belongs in
  `pkcore`.
- **Fallible methods must spell out `Result<Self, napi::Error<String>>`.** The
  `#[napi]` macro matches the literal token `Result<...>` in the signature to
  detect a fallible method. A type alias such as `PkResult<T>` compiles as a
  *return class* and fails with a confusing `ObjectFinalize` error.
- **`napi::Error<String>` is the only way to set a JS `.code`.** The default
  `napi::Error` is `Error<Status>` and can only report fixed N-API status names
  (`InvalidArg`, `GenericFailure`). napi-rs feeds `status.as_ref()` to
  `napi_create_error` as the JS error code, so a `String` status carries the
  `pkcore` error-variant name. EPIC-85 Scope requires this.
- **Chip counts are `i64`, never `u32`.** `pkcore` chip fields are `usize`.
  napi-rs maps `i64` to a plain JS `number` (exact below 2^53); an `as u32` cast
  wraps silently at 4,294,967,295. Small counts (seat indices, seat counts) stay
  `u32`.
- **Tuple structs work as `#[napi]` classes.** `pub struct Card(PkCard);` is the
  house shape, mirroring `pkcore.py`'s `#[pyclass] pub struct Card(PkCard)`.
- **napi-rs converts `snake_case` to `camelCase` automatically.** Do not
  hand-write `#[napi(js_name = ...)]` except where JS demands a reserved shape,
  such as `toString`.

## Two things that are deliberately missing

- **`PokerSession::run_hand` is not bound.** It takes a Rust closure. Calling a
  JS callback from inside a `&mut self` method would let that callback re-enter
  the same session object and alias the mutable borrow — undefined behaviour,
  and napi-rs does not guard against it. JS drives the loop with
  `startHand` / `nextActor` / `applyAction` / `endHand` instead. Do not "fix"
  this by adding a callback parameter.
- **`Winnings` has no `total()`.** Summing the pots is one line of JS, and all
  arithmetic belongs in `pkcore`.

## Driving a hand

`Dealer` and `PokerSession` stop on different signals. Getting this wrong is the
most common mistake:

- `PokerSession.nextActor()` returns `null` when betting is done, and deals each
  street for you. This is the loop you want.
- `Dealer.isHandInProgress()` stays `true` until `endHand()` runs, so it is the
  **wrong** loop terminator. Use `dealer.table.isGameOver()`, which flips as soon
  as the river betting closes.

## CI and releasing

- `ci.yml` runs on push/PR to `main`: `cargo fmt --check`, `cargo clippy -D
  warnings`, `npm run build`, `npm test`, `npm run typecheck`, then a
  **`git diff --exit-code` on `index.js` and `index.d.ts`**. That last step is
  the one that catches a signature change nobody rebuilt: the generated
  bindings are committed, so stale ones show up as a dirty tree rather than as
  wrong types on npm.
- `publish.yml` runs on a `v*` tag. Five build jobs, then one publish job that
  runs `napi create-npm-dirs` → `napi artifacts` → `napi pre-publish -t npm` →
  `npm publish`.
- **Do not add `--skip-optional-publish` to `pre-publish`.** It skips the
  per-platform packages, which are the whole point — the root package is just a
  loader that depends on them through `optionalDependencies`.
- `aarch64-unknown-linux-gnu` builds with `--use-napi-cross`. `pkcore` pulls in
  rusqlite and zstd, which compile C, so that leg needs a real cross toolchain
  and will not build with plain `cargo --target`.
- `artifacts/` and `npm/` are generated during a release and gitignored. Running
  `napi artifacts` locally also copies `.node` files into the repo root; they
  are gitignored too, but delete them or a stub can shadow a real build.
- `npm publish --provenance` requires a **public** repository. Provenance fails
  from a private one.

## Publishing auth: migrate to OIDC before January 2027

`publish.yml` currently authenticates with an `NPM_TOKEN` secret (a granular
access token). **npm is removing direct publishing from 2FA-bypass tokens around
January 2027.** After that the token can only stage a publish for a human to
approve, which does not work unattended.

Migration, in order:

1. Publish once with the token. This is what first creates the six packages
   (`pkcore` plus `pkcore-darwin-arm64`, `-darwin-x64`, `-linux-x64-gnu`,
   `-linux-arm64-gnu`, `-win32-x64-msvc`).
2. On npmjs.com, set a trusted publisher on **each of the six**: repository
   `ImperialBower/pkcore.js`, workflow `publish.yml`. Trusted publishing is
   configured per package, which is why step 1 has to happen first — unless npm
   has since added pre-registration for packages that do not exist yet, in which
   case skip straight to step 2 and never make a token.
3. Delete the `NPM_TOKEN` secret and drop `NODE_AUTH_TOKEN` from `publish.yml`.
   The `id-token: write` permission is already there, and `--provenance` becomes
   automatic under OIDC.

Related, already in effect since August 2026: a 2FA-bypass token can no longer
perform account, package, or organization management. Create and rotate tokens
interactively on the website with 2FA; it cannot be scripted.

`pkcore.py` uses PyPI trusted publishing already
(`pkcore.py/.github/workflows/publish.yml`, `permissions: id-token: write`), so
this ends with both bindings on the same OIDC model.

## Keep this package free of install scripts

npm v12 turned install-time lifecycle scripts **off by default**: a package with
`preinstall`/`install`/`postinstall` now needs the consumer to run
`npm approve-scripts` or pass `--allow-scripts`. Git and remote-URL dependencies
are likewise opt-in.

As of 2026-08-28 this package needs none of that — zero install hooks of our own,
and zero across all 118 packages in the lockfile. That is the point of napi-rs's
prebuilt-binary model: the platform addon arrives through
`optionalDependencies`, not a postinstall build.

**Do not add an install hook, a git dependency, or a remote-URL dependency.**
Any one of them turns `npm install pkcore` into a flagged install for every
downstream user.

## Generated files

`index.js` and `index.d.ts` are produced by `napi build` and **are committed**.
Re-run `npm run build` after any signature change, or the checked-in types drift
from the addon. `*.node` binaries are not committed.

## Commands you would not guess

```bash
npm run build:debug   # napi build --platform (fast, unoptimized)
npm run build         # napi build --platform --release (what CI ships)
npm test              # node --test, no test framework dependency
npm run typecheck     # tsc --noEmit against the generated index.d.ts
```

`npm test` requires a build first; it loads the compiled `.node` addon, not the
Rust source.
