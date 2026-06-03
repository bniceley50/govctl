//! Repo scanner: collect every `D###` reference across the working tree and git history.
//!
//! References come from source files, docs, and commit messages - everywhere *except* the
//! decision log itself (whose headings are definitions, not references). The scan honors a
//! `.govctlignore` file (glob-lite patterns) and inline `govctl:ignore` suppression so that
//! build artifacts, test fixtures, and documentation examples don't masquerade as real
//! references. See D003.

use crate::decisions::{extract_drefs, strip_html_comments};
use std::path::Path;
use std::process::Command;
use walkdir::WalkDir;

/// A `D###` reference discovered somewhere in the repo.
#[derive(Debug, Clone)]
pub struct Reference {
    pub raw: String,
    pub num: u32,
    /// Human-readable origin: a relative file path, or `git commit <shorthash>`.
    pub source: String,
    pub line: usize,
}

/// Patterns from `.govctlignore` plus built-in defaults.
pub struct IgnoreSet {
    patterns: Vec<String>,
}

impl IgnoreSet {
    /// Load `.govctlignore` from `root` (if present) and prepend the built-in defaults.
    pub fn load(root: &Path) -> Self {
        let mut patterns: Vec<String> = vec![".git/".into(), "target/".into()];
        if let Ok(contents) = std::fs::read_to_string(root.join(".govctlignore")) {
            for line in contents.lines() {
                let line = line.trim();
                if !line.is_empty() && !line.starts_with('#') {
                    patterns.push(line.to_string());
                }
            }
        }
        IgnoreSet { patterns }
    }

    /// Does `relpath` (forward-slash separated) match any ignore pattern?
    pub fn is_ignored(&self, relpath: &str) -> bool {
        self.patterns.iter().any(|p| pattern_matches(p, relpath))
    }
}

/// Match a single glob-lite pattern against a relative path.
fn pattern_matches(pattern: &str, relpath: &str) -> bool {
    let pat = pattern.trim();
    if pat.is_empty() || pat.starts_with('#') {
        return false;
    }
    let is_dir = pat.ends_with('/');
    let pat = pat.trim_end_matches('/');
    let components: Vec<&str> = relpath.split('/').collect();
    if is_dir {
        return components.iter().any(|c| wildcard_match(pat, c));
    }
    if pat.contains('/') {
        return wildcard_match(pat, relpath);
    }
    components.iter().any(|c| wildcard_match(pat, c))
}

/// Classic two-pointer wildcard match: `*` matches any run (including empty), `?` one char.
fn wildcard_match(pattern: &str, text: &str) -> bool {
    let p: Vec<char> = pattern.chars().collect();
    let t: Vec<char> = text.chars().collect();
    let (mut pi, mut ti) = (0usize, 0usize);
    let mut star: Option<usize> = None;
    let mut mark = 0usize;
    while ti < t.len() {
        if pi < p.len() && (p[pi] == '?' || p[pi] == t[ti]) {
            pi += 1;
            ti += 1;
        } else if pi < p.len() && p[pi] == '*' {
            star = Some(pi);
            mark = ti;
            pi += 1;
        } else if let Some(s) = star {
            pi = s + 1;
            mark += 1;
            ti = mark;
        } else {
            return false;
        }
    }
    while pi < p.len() && p[pi] == '*' {
        pi += 1;
    }
    pi == p.len()
}

/// Strip suppressed lines (`govctl:ignore`, and `govctl:ignore-start`/`-end` ranges) while
/// preserving line count, so reported line numbers stay accurate.
fn apply_suppression(raw: &str) -> String {
    let mut kept: Vec<&str> = Vec::new();
    let mut skipping = false;
    for line in raw.lines() {
        if line.contains("govctl:ignore-start") {
            skipping = true;
            kept.push("");
        } else if line.contains("govctl:ignore-end") {
            skipping = false;
            kept.push("");
        } else if skipping || line.contains("govctl:ignore") {
            kept.push("");
        } else {
            kept.push(line);
        }
    }
    kept.join("\n")
}

/// True if the byte slice looks like binary (contains a NUL in the sampled prefix).
fn looks_binary(bytes: &[u8]) -> bool {
    bytes.iter().take(8000).any(|&b| b == 0)
}

const MAX_FILE_BYTES: u64 = 1_048_576; // 1 MiB

/// Scan the working tree and git history under `root` for `D###` references.
pub fn scan(root: &Path) -> Vec<Reference> {
    let ignore = IgnoreSet::load(root);
    let mut refs = Vec::new();
    scan_files(root, &ignore, &mut refs);
    scan_git(root, &mut refs);
    refs
}

fn scan_files(root: &Path, ignore: &IgnoreSet, refs: &mut Vec<Reference>) {
    let walker = WalkDir::new(root).into_iter().filter_entry(|e| {
        // Prune ignored directories early.
        if e.file_type().is_dir() {
            if let Ok(rel) = e.path().strip_prefix(root) {
                let relstr = rel.to_string_lossy().replace('\\', "/");
                if !relstr.is_empty() && ignore.is_ignored(&format!("{relstr}/")) {
                    return false;
                }
            }
        }
        true
    });

    for entry in walker.filter_map(Result::ok) {
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        let rel = match path.strip_prefix(root) {
            Ok(r) => r.to_string_lossy().replace('\\', "/"),
            Err(_) => continue,
        };
        // The decision log holds definitions, not references - never scan it.
        if path.file_name().and_then(|n| n.to_str()) == Some("DECISIONS.md") {
            continue;
        }
        if ignore.is_ignored(&rel) {
            continue;
        }
        if let Ok(meta) = entry.metadata() {
            if meta.len() > MAX_FILE_BYTES {
                continue;
            }
        }
        let bytes = match std::fs::read(path) {
            Ok(b) => b,
            Err(_) => continue,
        };
        if looks_binary(&bytes) {
            continue;
        }
        let text = String::from_utf8_lossy(&bytes);
        let suppressed = apply_suppression(&text);
        let cleaned = strip_html_comments(&suppressed);
        for d in extract_drefs(&cleaned) {
            refs.push(Reference {
                raw: d.raw,
                num: d.num,
                source: rel.clone(),
                line: d.line,
            });
        }
    }
}

fn scan_git(root: &Path, refs: &mut Vec<Reference>) {
    if !root.join(".git").exists() {
        return;
    }
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["log", "-z", "--format=%H%n%B"])
        .output();
    let output = match output {
        Ok(o) if o.status.success() => o,
        _ => return,
    };
    let text = String::from_utf8_lossy(&output.stdout);
    for record in text.split('\0') {
        let record = record.trim();
        if record.is_empty() {
            continue;
        }
        let mut lines = record.lines();
        let hash = lines.next().unwrap_or("");
        let short = &hash[..hash.len().min(8)];
        let body: String = lines.collect::<Vec<_>>().join("\n");
        for d in extract_drefs(&body) {
            refs.push(Reference {
                raw: d.raw,
                num: d.num,
                source: format!("git commit {short}"),
                line: d.line,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wildcard_basics() {
        assert!(wildcard_match("*.lock", "Cargo.lock"));
        assert!(wildcard_match("target", "target"));
        assert!(!wildcard_match("*.lock", "Cargo.toml"));
        assert!(wildcard_match("*tests*", "integration_tests"));
        assert!(wildcard_match("src/*.rs", "src/main.rs"));
    }

    #[test]
    fn ignore_directory_component() {
        let ig = IgnoreSet {
            patterns: vec!["target/".into(), "*.lock".into()],
        };
        assert!(ig.is_ignored("target/debug/foo"));
        assert!(ig.is_ignored("Cargo.lock"));
        assert!(!ig.is_ignored("src/main.rs"));
    }

    #[test]
    fn suppression_inline_and_range() {
        // govctl:ignore-start
        let raw = "keep D001\nskip D900 govctl:ignore\ngovctl:ignore-start\nD901\nD902\ngovctl:ignore-end\nkeep D002";
        // govctl:ignore-end
        let out = apply_suppression(raw);
        let nums: Vec<u32> = extract_drefs(&out).iter().map(|r| r.num).collect();
        assert_eq!(nums, vec![1, 2]);
    }
}
