# govctl

[![crates.io](https://img.shields.io/crates/v/decision-drift.svg)](https://crates.io/crates/decision-drift) [![CI](https://github.com/bniceley50/govctl/actions/workflows/ci.yml/badge.svg)](https://github.com/bniceley50/govctl/actions/workflows/ci.yml) [![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

**Your decision log and your code drift apart, and nobody notices until it hurts.** `govctl` is a
decision-drift detector: it reads your `DECISIONS.md`, scans your source and git history, and
fails the build when they disagree - a reference to a decision that was never logged, a superseded
decision pointing nowhere, a locked choice quietly contradicted.

A human reviewer can't catch "this cites a decision that doesn't exist" by eye. `govctl` can - in
under a second, in one line of CI. It's a single self-contained binary (no runtime services or
dependencies), and it dogfoods its own rules: `govctl validate . --strict` passes on this very
repository.

<!-- govctl:ignore-start -->
> **What it caught on a real repo it had never seen** (a Next.js codebase): a pre-launch cutover
> checklist committed to decision `D042` ("rate-limit policy") - but the decision log skipped
> straight from D041 to D043. That decision was never written down. `govctl` flagged it in 0.3s,
> with zero other false alarms.
<!-- govctl:ignore-end -->

![govctl flagging a decision cited in a launch checklist that was never logged](docs/demo.gif)

## Who it's for

You keep a decision log (`DECISIONS.md` / ADRs) **and** you work with AI coding agents
(Claude Code, Cursor, Codex). That combination is where drift explodes: agents confidently cite
decision numbers and contradict locked choices faster than a human can track. If you don't keep a
decision log, or it's a tiny solo project, `govctl` is solving a problem you don't have - and
that's honest to admit.

## Why

Teams write down architectural decisions and then quietly violate them. References to decisions
that were never logged pile up. Superseded decisions point nowhere. `govctl` makes that drift a
build failure instead of a surprise six months later.

## Install

```
# From crates.io (requires Rust) - installs the `govctl` command:
cargo install decision-drift

# ...or download a prebuilt binary for your OS (no Rust needed):
#   https://github.com/bniceley50/govctl/releases
```

> The crate is named `decision-drift` (the obvious `govctl` was already taken on crates.io);
> the installed command is still `govctl`.

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
- uses: bniceley50/govctl/.github/actions/govctl-validate@v0.3.3
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
