//! `govctl validate` - detect drift between the decision log and the actual repo.
//!
//! Checks, in order:
//!   1. Presence - all seven governance files exist.
//!   2. sprint-status.yaml parses as YAML.
//!   3. honors_decisions entries are defined AND LOCKED.
//!   4. Orphaned references - a `D###` cited in source/git but absent from DECISIONS.md (error).
//!   5. Supersede-chain integrity - SUPERSEDED entries name an existing successor (error).
//!   6. Dangling LOCKED - a LOCKED decision referenced nowhere (warning).

use crate::decisions::{self, Status};
use crate::repo_scan;
use crate::templates;
use anyhow::{Context, Result};
use serde::Deserialize;
use std::collections::HashSet;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Severity {
    Error,
    Warning,
}

struct Finding {
    severity: Severity,
    message: String,
}

impl Finding {
    fn error(message: String) -> Self {
        Finding {
            severity: Severity::Error,
            message,
        }
    }
    fn warning(message: String) -> Self {
        Finding {
            severity: Severity::Warning,
            message,
        }
    }
}

#[derive(Deserialize)]
struct SprintStatus {
    #[serde(default)]
    honors_decisions: Vec<String>,
}

/// Run validation. Returns `Ok(true)` when the stack is clean (no errors, and no warnings under
/// `--strict`), `Ok(false)` when drift was found.
pub fn run(root: &Path, strict: bool) -> Result<bool> {
    let mut findings: Vec<Finding> = Vec::new();

    // Check 1: presence of the seven governance files.
    for name in templates::governance_filenames() {
        if !root.join(name).exists() {
            findings.push(Finding::error(format!("missing governance file: {name}")));
        }
    }

    // Parse DECISIONS.md.
    let decisions_path = root.join("DECISIONS.md");
    let decisions = if decisions_path.exists() {
        let raw = fs::read_to_string(&decisions_path)
            .with_context(|| format!("reading {}", decisions_path.display()))?;
        decisions::parse(&raw)
    } else {
        Vec::new()
    };
    let defined_nums: HashSet<u32> = decisions.iter().map(|d| d.num).collect();

    // Check 2: sprint-status.yaml parses; collect honored decisions.
    let mut honored: Vec<String> = Vec::new();
    let sprint_path = root.join("sprint-status.yaml");
    if sprint_path.exists() {
        let raw = fs::read_to_string(&sprint_path)
            .with_context(|| format!("reading {}", sprint_path.display()))?;
        match serde_yaml::from_str::<SprintStatus>(&raw) {
            Ok(s) => honored = s.honors_decisions,
            Err(e) => findings.push(Finding::error(format!(
                "sprint-status.yaml is not valid YAML: {e}"
            ))),
        }
    }

    // Scan the repo + git history for references.
    let references = repo_scan::scan(root);
    let referenced_nums: HashSet<u32> = references.iter().map(|r| r.num).collect();

    // Check 3: honored decisions must exist and be LOCKED.
    for h in &honored {
        match dnum(h).and_then(|n| decisions.iter().find(|d| d.num == n)) {
            None => findings.push(Finding::error(format!(
                "sprint-status honors {h}, which is not defined in DECISIONS.md"
            ))),
            Some(d) if d.status != Status::Locked => findings.push(Finding::error(format!(
                "sprint-status honors {} but it is {}, not LOCKED",
                d.id,
                status_label(&d.status)
            ))),
            _ => {}
        }
    }

    // Check 4: orphaned references (report each distinct orphan once, at its first site).
    let mut reported: HashSet<u32> = HashSet::new();
    for r in &references {
        if !defined_nums.contains(&r.num) && reported.insert(r.num) {
            // File references get a line number; git-commit references already carry a hash.
            let loc = if r.source.starts_with("git commit") {
                r.source.clone()
            } else {
                format!("{}:{}", r.source, r.line)
            };
            findings.push(Finding::error(format!(
                "orphaned reference {} in {loc} - not found in DECISIONS.md",
                r.raw
            )));
        }
    }

    // Check 5: supersede-chain integrity.
    for d in &decisions {
        if let Status::Superseded { by } = &d.status {
            match by {
                None => findings.push(Finding::error(format!(
                    "{} '{}' is SUPERSEDED but names no successor",
                    d.id, d.title
                ))),
                Some(succ) => {
                    if dnum(succ).is_none_or(|n| !defined_nums.contains(&n)) {
                        findings.push(Finding::error(format!(
                            "{} '{}' is SUPERSEDED by {}, which does not exist",
                            d.id, d.title, succ
                        )));
                    }
                }
            }
        }
    }

    // Check 6: dangling LOCKED decisions (warning).
    for d in &decisions {
        if d.status == Status::Locked && !referenced_nums.contains(&d.num) {
            findings.push(Finding::warning(format!(
                "{} '{}' is LOCKED but is referenced nowhere in the repo",
                d.id, d.title
            )));
        }
    }

    // Report.
    let errors = findings
        .iter()
        .filter(|f| f.severity == Severity::Error)
        .count();
    let warnings = findings
        .iter()
        .filter(|f| f.severity == Severity::Warning)
        .count();
    let referenced_defined = decisions
        .iter()
        .filter(|d| referenced_nums.contains(&d.num))
        .count();
    let ok = errors == 0 && (!strict || warnings == 0);

    println!("govctl validate - {}", root.display());
    println!();
    if findings.is_empty() {
        println!("  No drift detected.");
    } else {
        for f in &findings {
            let tag = match f.severity {
                Severity::Error => "ERROR",
                Severity::Warning => "WARN ",
            };
            println!("  {tag}  {}", f.message);
        }
    }
    println!();
    println!(
        "Summary: {} decision(s) defined, {} referenced; {} error(s), {} warning(s).",
        decisions.len(),
        referenced_defined,
        errors,
        warnings
    );
    if strict && warnings > 0 && errors == 0 {
        println!("(--strict: warnings are treated as failures)");
    }
    println!("{}", if ok { "validation passed" } else { "validation failed" });

    Ok(ok)
}

/// Parse the numeric value out of a `D###` token (the id for decision seven yields `Some(7)`).
fn dnum(s: &str) -> Option<u32> {
    decisions::extract_drefs(s).first().map(|r| r.num)
}

fn status_label(status: &Status) -> String {
    match status {
        Status::Proposed => "PROPOSED".to_string(),
        Status::Locked => "LOCKED".to_string(),
        Status::Superseded { .. } => "SUPERSEDED".to_string(),
        Status::Other(s) if s.is_empty() => "(no status)".to_string(),
        Status::Other(s) => s.clone(),
    }
}
