# RUNBOOK.md - govctl

How to build, test, and operate govctl.

## Build

```
cargo build --release        # -> target/release/govctl(.exe)
```

## Test

```
cargo test                                   # unit + integration
cargo clippy --all-targets -- -D warnings    # lint, warnings are errors
govctl validate . --strict                   # govctl validates itself
```

## Use

```
govctl init <dir> --project-name "My Project"   # scaffold the stack
govctl validate <dir>                            # check for drift
govctl validate <dir> --strict                   # CI mode: warnings fail
```

## Release

Tag `vX.Y.Z` and push; `.github/workflows/release.yml` builds cross-platform binaries and
attaches them as release assets (see D004).

_Last reviewed: 2026-06-03_
