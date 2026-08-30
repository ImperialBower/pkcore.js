.PHONY: default help install clean build build-debug demo test typecheck fmt fmt-check clippy check-bindings check-scripts version-check actionlint ci ayce tree tree-duplicates bump-pkcore

# Default target
default: ayce

# Display help information
help:
	@echo "Available targets:"
	@echo "  make (default)       - Run ayce"
	@echo "  make ayce            - Clean, format, build, test, typecheck, clippy, and every CI-parity check"
	@echo "  make install         - npm ci (install pinned Node deps)"
	@echo "  make build           - Build the release addon (napi build --release)"
	@echo "  make build-debug     - Build the debug addon (fast, unoptimized)"
	@echo "  make test            - Run the Node test suite against the built addon"
	@echo "  make typecheck       - tsc --noEmit against the generated index.d.ts"
	@echo "  make demo            - Run demo.mjs against the built addon"
	@echo "  make clean           - cargo clean, plus stray local *.node/artifacts/npm dirs"
	@echo "  make fmt             - Format Rust code"
	@echo "  make fmt-check       - Check Rust formatting without writing (what CI runs)"
	@echo "  make clippy          - Run clippy with warnings denied (what CI runs)"
	@echo "  make check-bindings  - Fail if index.js/index.d.ts are stale vs the last build"
	@echo "  make check-scripts   - Fail if any dependency in package-lock.json has an install script"
	@echo "  make version-check   - Fail unless Cargo.toml, package.json, and the pkcore dep pin all agree"
	@echo "  make actionlint      - Lint .github/workflows/ (skips with a warning if not installed)"
	@echo "  make ci              - Mirror ci.yml exactly, in order"
	@echo "  make tree            - Show the Cargo dependency tree"
	@echo "  make tree-duplicates - Show duplicate Cargo dependencies"
	@echo "  make bump-pkcore VERSION=x.y.z - Bump the pkcore dep + this package's version everywhere, rebuild"
	@echo "  make help            - Display this help message"

# Install pinned Node dependencies
install:
	npm ci

# Build the release addon (what CI ships)
build:
	npm run build

# Build the debug addon (fast, unoptimized)
build-debug:
	npm run build:debug

# Run the Node test suite. Loads the compiled .node addon, not the Rust
# source directly, so run `make build` first after any Rust change.
test:
	npm test

# Type-check the generated definitions
typecheck:
	npm run typecheck

# Run the feature showcase against the built addon
demo:
	npm run demo

# cargo clean, plus the local platform addons and generated release dirs.
# `napi artifacts` (run during a release, or by hand) copies .node files into
# the repo root; they're gitignored but a stale one can shadow a real build,
# per CLAUDE.md.
clean:
	cargo clean
	rm -f ./*.node
	rm -rf artifacts artifacts-raw npm

# Format Rust code
fmt:
	cargo fmt --all

# Check Rust formatting without writing (what CI runs)
fmt-check:
	cargo fmt --check

# Run clippy with warnings denied (what CI runs)
clippy:
	cargo clippy --all-targets -- -D warnings

# `napi build` regenerates index.js and index.d.ts. They are committed, so a
# signature change that was not rebuilt shows up here as a dirty tree rather
# than as wrong types shipped to npm. Run `make build` first.
check-bindings:
	@if ! git diff --exit-code -- index.js index.d.ts; then \
		echo "index.js / index.d.ts are stale. Run 'make build' and commit the result."; \
		exit 1; \
	fi

# npm v12 disables lifecycle scripts by default; this package promises to need
# none. npm's lockfile flags any dependency that declares one with
# "hasInstallScript": true, so a clean grep proves the whole tree stays hook-free.
check-scripts:
	@if grep -q '"hasInstallScript"' package-lock.json; then \
		echo "A dependency declares an install script — this package must stay hook-free. See package-lock.json."; \
		grep -B3 '"hasInstallScript"' package-lock.json; \
		exit 1; \
	fi
	@echo "No install scripts in the dependency tree."

# Enforce the version lockstep rule from CLAUDE.md: pkcore-js's own version
# (Cargo.toml, package.json) and the pkcore dependency it pins must all match.
version-check:
	@crate_version=$$(grep -m1 '^version = ' Cargo.toml | cut -d'"' -f2); \
	pkg_version=$$(node -p "require('./package.json').version"); \
	dep_version=$$(grep -m1 '^pkcore = ' Cargo.toml | sed -E 's/.*version = "([^"]*)".*/\1/'); \
	if [ "$$crate_version" != "$$pkg_version" ] || [ "$$crate_version" != "$$dep_version" ]; then \
		echo "Version mismatch: Cargo.toml=$$crate_version package.json=$$pkg_version pkcore dep=$$dep_version"; \
		exit 1; \
	fi; \
	echo "Versions match: $$crate_version"

# Lint GitHub Actions workflow files. Not a cargo tool — install with
# `brew install actionlint`. Missing tool = skip with a warning so `make ci`
# still works on machines without it; an actual lint failure still fails.
actionlint:
	@if command -v actionlint >/dev/null 2>&1; then \
		actionlint; \
	else \
		echo "WARNING: actionlint not installed — skipping workflow lint. Install: https://github.com/rhysd/actionlint#installation"; \
	fi

# Mirror ci.yml exactly, in the same order, so a local pass predicts a CI pass.
ci: fmt-check clippy build test typecheck check-bindings

# All You Can Eat: a full clean build, formatted, tested, linted, and checked
# against every repo-specific rule (bindings freshness, no install scripts,
# version lockstep) before you push. Slower than `ci` on purpose — `clean`
# forces a from-scratch compile instead of reusing the incremental cache.
ayce: clean fmt build test typecheck clippy check-bindings check-scripts version-check actionlint

# Show the Cargo dependency tree
tree:
	cargo tree

# Show duplicate Cargo dependencies
tree-duplicates:
	cargo tree --duplicates

# Bump the pkcore dependency (and this package's own version, which must
# match it per CLAUDE.md's version rule) everywhere, then rebuild.
# Usage: make bump-pkcore VERSION=0.11.0
bump-pkcore:
	@if [ -z "$(VERSION)" ]; then \
		echo "Usage: make bump-pkcore VERSION=x.y.z"; \
		exit 1; \
	fi
	sed -i.bak 's/^version = ".*"/version = "$(VERSION)"/' Cargo.toml
	sed -i.bak 's/^pkcore = { version = "[^"]*"/pkcore = { version = "$(VERSION)"/' Cargo.toml
	rm -f Cargo.toml.bak
	sed -i.bak 's/"version": "[^"]*"/"version": "$(VERSION)"/' package.json
	rm -f package.json.bak
	cargo update -p pkcore --precise $(VERSION)
	npm run build
	@echo "Bumped to $(VERSION). Review the diff, then: make test && make typecheck"
