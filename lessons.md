# Lessons - govctl

## 2026-06-03 - Template comments became phantom decisions
- **What happened:** The DECISIONS.md template's instructional `<!-- ### D00X ... -->` block was
  parsed as a real decision.
- **Why:** The parser scanned headings without stripping HTML comments.
- **Lesson / how to apply:** Strip comments before parsing (honors D002).

## 2026-06-03 - Doc examples and fixtures tripped the orphan check
- **What happened:** Example D-numbers in tests and the README were flagged as orphaned refs.
- **Why:** The scanner read every file blindly.
- **Lesson / how to apply:** Path-level `.govctlignore` plus inline suppression markers
  (honors D003). Dogfood it: govctl's own repo must pass `validate --strict`.

## 2026-06-03 - Non-ASCII source mis-decoded on this toolchain
- **What happened:** An em-dash in a Rust string literal compiled to mojibake (U+00E2 ...),
  breaking title parsing.
- **Why:** Source bytes were valid UTF-8 on disk but decoded as Latin-1 during the build.
- **Lesson / how to apply:** Keep all source and templates ASCII-only.

_Last reviewed: 2026-06-03_
