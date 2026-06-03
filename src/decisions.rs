//! Parser for `DECISIONS.md`.
//!
//! A line-state parser (not regex soup) that walks the decision log and extracts each entry's
//! id, title, and status. It is comment-aware: `<!-- ... -->` blocks are stripped before parsing,
//! so the instructional comment in the template (which contains an example decision heading)
//! is never mistaken for a real decision. See D002.

/// Lifecycle status of a single decision.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Status {
    Proposed,
    Locked,
    /// SUPERSEDED, optionally naming the successor decision's id when one is given.
    Superseded { by: Option<String> },
    /// Any other status string we don't model explicitly.
    Other(String),
}

/// A single parsed decision.
#[derive(Debug, Clone)]
pub struct Decision {
    /// Literal id as written, e.g. `"D001"`.
    pub id: String,
    /// Numeric value of the id, e.g. `1`, used for reference matching across zero-padding.
    pub num: u32,
    /// Short title following the `-`/`-` on the heading line.
    pub title: String,
    pub status: Status,
}

/// A `D###` reference found in free text, with the line it appeared on.
#[derive(Debug, Clone)]
pub struct DRef {
    pub raw: String,
    pub num: u32,
    pub line: usize,
}

/// Remove `<!-- ... -->` HTML comment spans (including multi-line ones) from `text`,
/// preserving line count so reported line numbers stay accurate. Operates on `&str` slices
/// (the `<!--`/`-->` markers are ASCII), so multibyte UTF-8 content is preserved intact.
pub fn strip_html_comments(text: &str) -> String {
    // Helper: append only the newlines from `s`, to keep line numbers stable across removals.
    fn push_newlines(out: &mut String, s: &str) {
        for c in s.chars() {
            if c == '\n' {
                out.push('\n');
            }
        }
    }

    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    loop {
        match rest.find("<!--") {
            None => {
                out.push_str(rest);
                break;
            }
            Some(start) => {
                out.push_str(&rest[..start]);
                let after = &rest[start + 4..];
                match after.find("-->") {
                    None => {
                        // Unterminated comment: drop the body but keep its newlines.
                        push_newlines(&mut out, after);
                        break;
                    }
                    Some(end) => {
                        push_newlines(&mut out, &after[..end]);
                        rest = &after[end + 3..];
                    }
                }
            }
        }
    }
    out
}

/// Extract every `D###` reference from arbitrary text. A reference is an ASCII `D` on a word
/// boundary followed by one or more digits (so `3D`, `ID3`, `UUID4` do not match).
pub fn extract_drefs(text: &str) -> Vec<DRef> {
    let mut refs = Vec::new();
    for (lineno, line) in text.lines().enumerate() {
        let chars: Vec<char> = line.chars().collect();
        let mut i = 0;
        while i < chars.len() {
            if chars[i] == 'D' {
                let prev_ok = i == 0 || !chars[i - 1].is_alphanumeric();
                let mut j = i + 1;
                while j < chars.len() && chars[j].is_ascii_digit() {
                    j += 1;
                }
                if prev_ok && j > i + 1 {
                    let raw: String = chars[i..j].iter().collect();
                    let num: u32 = raw[1..].parse().unwrap_or(0);
                    refs.push(DRef {
                        raw,
                        num,
                        line: lineno + 1,
                    });
                    i = j;
                    continue;
                }
            }
            i += 1;
        }
    }
    refs
}

/// Parse the contents of a `DECISIONS.md` file into a list of decisions.
pub fn parse(contents: &str) -> Vec<Decision> {
    let cleaned = strip_html_comments(contents);
    let mut decisions: Vec<Decision> = Vec::new();

    for line in cleaned.lines() {
        let trimmed = line.trim_start();
        // A decision heading is a markdown heading at ANY level (`#`..`######`) whose text is
        // `D<digits>`. Projects in the wild use `## D001` and `### D001` interchangeably, so we
        // match on "one-or-more '#' then whitespace" rather than a fixed level.
        if let Some(rest) = strip_heading_marker(trimmed) {
            if let Some(d) = parse_heading(rest) {
                decisions.push(d);
                continue;
            }
        }
        if let Some(d) = decisions.last_mut() {
            if let Some(status) = parse_status_line(trimmed) {
                // First status line after a heading wins.
                if matches!(d.status, Status::Other(ref s) if s.is_empty()) {
                    d.status = status;
                }
            }
        }
    }
    decisions
}

/// If `line` is a markdown heading (`#`..`######` followed by whitespace), return the heading
/// text with the marker stripped; otherwise `None`.
fn strip_heading_marker(line: &str) -> Option<&str> {
    if !line.starts_with('#') {
        return None;
    }
    let rest = line.trim_start_matches('#');
    // Require whitespace between the hashes and the text (a real ATX heading).
    if rest.starts_with(char::is_whitespace) {
        Some(rest.trim())
    } else {
        None
    }
}

/// Parse a heading like `D001 - Initial architecture` into a partial `Decision`
/// (status filled in later from the following lines).
fn parse_heading(text: &str) -> Option<Decision> {
    let text = text.trim();
    if !text.starts_with('D') {
        return None;
    }
    let digits: String = text[1..].chars().take_while(|c| c.is_ascii_digit()).collect();
    if digits.is_empty() {
        return None;
    }
    let num: u32 = digits.parse().ok()?;
    let id = format!("D{digits}");
    // Title is whatever follows the id and its separator. Strip the leading run of
    // separators generically (whitespace, any dash codepoint, colon, etc.) rather than
    // enumerating dash characters - robust regardless of em-dash vs hyphen vs encoding.
    let after = &text[1 + digits.len()..];
    let title = after
        .trim_start_matches(|c: char| !c.is_alphanumeric())
        .trim()
        .to_string();
    Some(Decision {
        id,
        num,
        title,
        status: Status::Other(String::new()),
    })
}

/// Parse a `- **Status:** LOCKED` style line into a `Status`, or `None` if it isn't a status line.
fn parse_status_line(line: &str) -> Option<Status> {
    let lower = line.to_lowercase();
    if !lower.contains("status") {
        return None;
    }
    if lower.contains("superseded") {
        // Look for "by D###".
        let by = extract_drefs(line).into_iter().next().map(|r| r.raw);
        return Some(Status::Superseded { by });
    }
    if lower.contains("locked") {
        return Some(Status::Locked);
    }
    if lower.contains("proposed") {
        return Some(Status::Proposed);
    }
    // A status line we don't model - capture the value after the colon.
    let value = line.rsplit(':').next().unwrap_or("").trim();
    Some(Status::Other(value.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    // Sample decision logs used by the parser tests. Wrapped in govctl:ignore markers so that
    // govctl's own `validate` does not read these fixture D-numbers as real references. This is
    // the suppression feature (D003) dog-fooding itself.
    // govctl:ignore-start
    const SAMPLE: &str = "\
# Decisions

### D001 - First decision
- **Status:** LOCKED

### D002 - Second decision
- **Status:** PROPOSED

### D003 - Old decision
- **Status:** SUPERSEDED (by D002)
";

    const WITH_COMMENT: &str = "\
<!--
Template instructions:
### D00X - <short title>
- **Status:** LOCKED | SUPERSEDED (by D00Y)
-->

### D001 - Real decision
- **Status:** LOCKED
";

    // Real-world style: level-2 headings, capitalized status with no leading dash.
    const TWO_HASH: &str = "\
# DECISIONS.md - Architecture Decisions Log

## D001 - Stack Lock
**Date:** 2026-04-21
**Status:** Locked

## D002 - Deferred thing
**Status:** Locked
";
    // govctl:ignore-end

    #[test]
    fn parses_level_two_headings() {
        let d = parse(TWO_HASH);
        assert_eq!(d.len(), 2, "should parse ## D-headings and ignore the title heading");
        assert_eq!(d[0].id, "D001");
        assert_eq!(d[0].title, "Stack Lock");
        assert_eq!(d[0].status, Status::Locked);
        assert_eq!(d[1].id, "D002");
    }

    #[test]
    fn parses_three_decisions() {
        let d = parse(SAMPLE);
        assert_eq!(d.len(), 3);
        assert_eq!(d[0].id, "D001");
        assert_eq!(d[0].num, 1);
        assert_eq!(d[0].title, "First decision");
        assert_eq!(d[0].status, Status::Locked);
        assert_eq!(d[1].status, Status::Proposed);
    }

    #[test]
    fn parses_supersede_pointer() {
        let d = parse(SAMPLE);
        assert_eq!(
            d[2].status,
            Status::Superseded {
                by: Some("D002".to_string())
            }
        );
    }

    #[test]
    fn ignores_decisions_inside_html_comments() {
        let d = parse(WITH_COMMENT);
        // Only the real D001 should survive; the example heading in the comment must be skipped.
        assert_eq!(d.len(), 1);
        assert_eq!(d[0].id, "D001");
    }

    #[test]
    fn dref_word_boundary() {
        // govctl:ignore-start
        let refs = extract_drefs("see D207 and 3D and ID3 and D42x");
        // govctl:ignore-end
        let nums: Vec<u32> = refs.iter().map(|r| r.num).collect();
        assert_eq!(nums, vec![207, 42]);
    }

    #[test]
    fn strip_comments_preserves_line_count() {
        let src = "a\n<!-- x\ny -->\nb\n";
        let out = strip_html_comments(src);
        assert_eq!(out.lines().count(), src.lines().count());
    }

    #[test]
    fn strip_comments_preserves_multibyte_utf8() {
        // Regression: a previous bytes-as-char implementation corrupted multibyte chars.
        // U+2014 (EM DASH) outside comments must survive intact.
        let src = "## D001 \u{2014} Title\n<!-- note \u{2014} here -->\nbody \u{2014} x";
        let out = strip_html_comments(src);
        assert!(out.contains('\u{2014}'), "em-dash outside comments must survive");
        assert!(!out.contains("note"), "comment body should be removed");
    }

    #[test]
    fn parses_emdash_heading_title_cleanly() {
        // The real-world separator is an em-dash; the title must come out clean (no mojibake).
        let d = parse("## D001 \u{2014} Stack Lock\n**Status:** Locked\n");
        assert_eq!(d.len(), 1);
        assert_eq!(d[0].title, "Stack Lock");
        assert_eq!(d[0].status, Status::Locked);
    }
}
