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

## 2026-06-03 - Real decision logs use ## headings, not ###
- **What happened:** govctl parsed 0 decisions in real projects (GravenSpire, Clinic Notes AI).
- **Why:** The parser only matched `###` headings; the actual convention is `## D001`.
- **Lesson / how to apply:** Accept any markdown heading level for decision headings (honors D005).

## 2026-06-03 - UTF-8 corruption from casting bytes to char
- **What happened:** Em-dash separators in real logs (`## D001 - Title`) rendered as mojibake,
  breaking title extraction.
- **Why:** `strip_html_comments` iterated bytes and did `bytes[i] as char`, splitting multibyte
  UTF-8 into Latin-1 chars. (First misdiagnosed as a source-encoding / toolchain issue - it
  was not; making the source ASCII only masked it.)
- **Lesson / how to apply:** Parse on `&str`/`chars`, never cast bytes to `char` (honors D005).

_Last reviewed: 2026-06-03_
