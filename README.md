# govctl

A governance-stack scaffolder and **decision-drift detector**. `govctl` scaffolds a seven-file
governance stack into any project and then enforces that your decision log and your actual code
(and git history) have not drifted apart.

It dogfoods its own rules: `govctl validate . --strict` passes on this very repository.

## Why

Teams write down architectural decisions and then quietly violate them. References to decisions
that were never logged pile up. Superseded decisions point nowhere. `govctl` makes that drift a
build failure instead of a surprise six months later.

## Install

```
cargo install --path .          # from a checkout
# or grab a prebuilt binary from the Releases page
```

## Usage

### Scaffold a stack

```
govctl init . --project-name "My Project"
```

Writes seven governance files (`CLAUDE.md`, `AGENTS.md`, `DECISIONS.md`, `RED_TEAM.md`,
`RUNBOOK.md`, `sprint-status.yaml`, `lessons.md`), a `.govctlignore`, and `.govctl/manifest.toml`.
Refuses to overwrite without `--force`; preview with `--dry-run`.

Adopting govctl on a project that already has some of these files? Use `--merge` to add only the
missing ones and leave your existing files untouched:

```
govctl init . --merge        # add missing governance files, keep what's already there
```

### Check for drift

```
govctl validate .            # report drift
govctl validate . --strict   # CI mode: warnings become failures
```

`validate` runs these checks:

1. **Presence** - all seven governance files exist.
2. **YAML integrity** - `sprint-status.yaml` parses.
3. **Honor integrity** - every `honors_decisions` id is defined AND `LOCKED`.
4. **Orphaned references** - a `D###` cited in source or a commit message that isn't in
   `DECISIONS.md` (error).
5. **Supersede-chain integrity** - a `SUPERSEDED` entry must name an existing successor (error).
6. **Dangling LOCKED** - a `LOCKED` decision referenced nowhere (warning).

Example, on a repo with three planted problems:

<!-- govctl:ignore-start -->
```
govctl validate - .

  ERROR  orphaned reference D207 in src/lib.rs:12 - not found in DECISIONS.md
  ERROR  orphaned reference D310 in git commit e92f4999 - not found in DECISIONS.md
  ERROR  D003 'Old approach' is SUPERSEDED by D099, which does not exist

Summary: 2 decision(s) defined, 1 referenced; 3 error(s), 0 warning(s).
validation failed
```
<!-- govctl:ignore-end -->

## Suppressing false positives

Documentation examples and test fixtures legitimately contain fake D-numbers. Two mechanisms keep
them from tripping the orphan check (see decision D003):

- **`.govctlignore`** - glob-lite patterns for whole paths (`target/`, `tests/`, `*.lock`).
- **Inline markers** - put `govctl:ignore` on a line to skip it, or bracket a block with
  `govctl:ignore-start` / `govctl:ignore-end`.
- **Write references like IDs** - a reference must be zero-padded to your decision-ID width
  (`D001`), so analytics-style notation such as `D7` or `D30` is never mistaken for a reference.

## CI

Add governance checks to any repo in one step. Copy
[`governance-dropin.yml`](.github/governance-dropin.yml) to
`.github/workflows/governance.yml` in your project - it calls the reusable composite action, which installs
`govctl` (prebuilt-binary fast path, source-build fallback) and runs `validate --strict` on every
PR (see decision D004).

```yaml
- uses: bniceley50/govctl/.github/actions/govctl-validate@v0.3.1
  with:
    path: "."
    strict: "true"
```

## Development

```
cargo test                                   # unit + integration
cargo clippy --all-targets -- -D warnings    # lint
cargo build --release
govctl validate . --strict                   # govctl validates itself
```

Source is kept ASCII-only (see lessons.md). Architectural decisions are in
[`DECISIONS.md`](DECISIONS.md).

## License

MIT
