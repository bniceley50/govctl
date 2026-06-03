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
/// preserving line count so reported line numbers stay accurate.
pub fn strip_html_comments(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let bytes = text.as_bytes();
    let mut i = 0;
    let mut in_comment = false;
    while i < bytes.len() {
        if !in_comment && bytes[i..].starts_with(b"<!--") {
            in_comment = true;
            i += 4;
        } else if in_comment && bytes[i..].starts_with(b"-->") {
            in_comment = false;
            i += 3;
        } else {
            // Preserve newlines so downstream line numbering is unaffected.
            if bytes[i] == b'\n' {
                out.push('\n');
            } else if !in_comment {
                out.push(bytes[i] as char);
            }
            i += 1;
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
        if let Some(rest) = trimmed.strip_prefix("###") {
            if let Some(d) = parse_heading(rest.trim()) {
                decisions.push(d);
            }
        } else if let Some(d) = decisions.last_mut() {
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
    // govctl:ignore-end

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
}
