# Contributing to govctl

Thanks for considering a contribution. govctl is a governance tool, so it holds itself to its
own rules - the most important thing to know is that **every change must keep govctl passing its
own checks.**

## Local checks (all must pass)

```
cargo test                                   # unit + integration tests
cargo clippy --all-targets -- -D warnings    # lint; warnings are errors
cargo build --release
govctl validate . --strict                   # govctl validates its own governance stack
```

CI runs exactly these on every pull request (see `.github/workflows/ci.yml`).

## The one rule that's easy to miss

If your change makes an architectural decision, **add an entry to `DECISIONS.md`** (status
`LOCKED` once settled) and reference it where it's honored. If you reference a decision number
(`D001`) in code or commits that isn't in the log, `govctl validate` will fail the build - on
purpose. That's the whole point of the tool, applied to itself.

Examples and test fixtures that legitimately contain fake D-numbers should be excluded via
`.govctlignore` or wrapped in `govctl:ignore` / `govctl:ignore-start` ... `govctl:ignore-end`
markers.

## Pull requests

1. Branch from `main`.
2. Make a focused change; match the surrounding code style.
3. Run the local checks above.
4. Open the PR. Keep the description honest about what you changed and verified.

## Reporting issues

Open a GitHub issue with: what you expected, what happened, and the smallest repro you can manage
(a tiny `DECISIONS.md` + the `govctl` command is ideal).

## Releases

Maintainers tag `vX.Y.Z`; `.github/workflows/release.yml` builds cross-platform binaries, and the
crate is published to crates.io as `decision-drift` (the installed command stays `govctl`).
