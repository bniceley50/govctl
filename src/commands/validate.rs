//! `govctl validate` - detect drift between the decision log and the actual repo.
//!
//! Checks, in order:
//!   1. Presence - all seven governance files exist.
//!   2. sprint-status.yaml parses as YAML.
//!   3. honors_decisions entries are defined AND LOCKED.
//!   4. Orphaned references - a `D###` cited in source/git but absent from DECISIONS.md (error).
//!   5. Supersede-chain integrity - SUPERSEDED entries name an existing successor (error).
//!   6. Dangling LOCKED - a LOCKED decision referenced nowhere (warning).
//!
//! Output is human-readable by default; `--format json` emits a stable machine-readable report
//! to stdout for CI, bots, and agents. The JSON contract (codes, fix kinds, exit reasons) is D008.

use crate::decisions::{self, Status};
use crate::repo_scan;
use crate::templates;
use anyhow::{Context, Result};
use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::Path;

/// Output format for `validate`. Human is the default; JSON writes only the report to stdout.
#[derive(Clone, Copy, ValueEnum)]
pub enum Format {
    Human,
    Json,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
enum Severity {
    Error,
    Warning,
}

/// Stable machine-readable finding code (D008 contract). Changing a value is a breaking change.
#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum Code {
    MissingFile,
    InvalidSprintStatus,
    HonoredDecisionMissing,
    HonorNotLocked,
    OrphanedReference,
    SupersededWithoutSuccessor,
    BrokenSupersedeChain,
    DanglingLocked,
}

/// Stable hint for what kind of fix resolves a finding (D008 contract).
#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum FixKind {
    AddFile,
    FixSprintStatus,
    AddDecision,
    LockDecision,
    FixReference,
    AddReference,
    NameSuccessor,
}

/// Why the process exits the way it does (D008 contract).
#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum ExitReason {
    Passed,
    Errors,
    StrictWarnings,
}

#[derive(Serialize)]
struct Finding {
    severity: Severity,
    code: Code,
    message: String,
    #[serde(rename = "decisionId")]
    decision_id: Option<String>,
    source: Option<String>,
    line: Option<usize>,
    #[serde(rename = "suggestedFixKind")]
    suggested_fix_kind: FixKind,
}

#[derive(Serialize)]
struct Summary {
    #[serde(rename = "decisionsDefined")]
    decisions_defined: usize,
    referenced: usize,
    errors: usize,
    warnings: usize,
}

#[derive(Serialize)]
struct Report {
    #[serde(rename = "schemaVersion")]
    schema_version: u32,
    ok: bool,
    strict: bool,
    summary: Summary,
    findings: Vec<Finding>,
    #[serde(rename = "exitReason")]
    exit_reason: ExitReason,
}

const SCHEMA_VERSION: u32 = 1;

#[derive(Deserialize)]
struct SprintStatus {
    #[serde(default)]
    honors_decisions: Vec<String>,
}

/// Run validation. Returns `Ok(true)` when the stack is clean (no errors, and no warnings under
/// `--strict`), `Ok(false)` when drift was found. Exit semantics are identical across formats.
pub fn run(root: &Path, strict: bool, format: Format) -> Result<bool> {
    let mut findings: Vec<Finding> = Vec::new();

    // Check 1: presence of the seven governance files.
    for name in templates::governance_filenames() {
        if !root.join(name).exists() {
            findings.push(Finding {
                severity: Severity::Error,
                code: Code::MissingFile,
                message: format!("missing governance file: {name}"),
                decision_id: None,
                source: None, // repo-level finding; no source file to point at
                line: None,
                suggested_fix_kind: FixKind::AddFile,
            });
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
            Err(e) => findings.push(Finding {
                severity: Severity::Error,
                code: Code::InvalidSprintStatus,
                message: format!("sprint-status.yaml is not valid YAML: {e}"),
                decision_id: None,
                source: Some("sprint-status.yaml".to_string()),
                line: None,
                suggested_fix_kind: FixKind::FixSprintStatus,
            }),
        }
    }

    // Scan the repo + git history for references.
    let references = repo_scan::scan(root);
    // A token only counts as a decision reference if it is written like a decision ID: zero-padded
    // to at least the width used in the log (e.g. `D001`). This keeps analytics-style notation such
    // as `D30` (day-30 retention) or `D7` from masquerading as references to decisions 30 / 7.
    // See D007.
    let min_id_digits = decisions
        .iter()
        .map(|d| d.id.len().saturating_sub(1))
        .min()
        .unwrap_or(3);
    let references: Vec<repo_scan::Reference> = references
        .into_iter()
        .filter(|r| r.raw.len().saturating_sub(1) >= min_id_digits)
        .collect();
    let referenced_nums: HashSet<u32> = references.iter().map(|r| r.num).collect();

    // Check 3: honored decisions must exist and be LOCKED.
    for h in &honored {
        match dnum(h).and_then(|n| decisions.iter().find(|d| d.num == n)) {
            None => findings.push(Finding {
                severity: Severity::Error,
                code: Code::HonoredDecisionMissing,
                message: format!("sprint-status honors {h}, which is not defined in DECISIONS.md"),
                decision_id: Some(h.clone()),
                source: Some("sprint-status.yaml".to_string()),
                line: None,
                suggested_fix_kind: FixKind::AddDecision,
            }),
            Some(d) if d.status != Status::Locked => findings.push(Finding {
                severity: Severity::Error,
                code: Code::HonorNotLocked,
                message: format!(
                    "sprint-status honors {} but it is {}, not LOCKED",
                    d.id,
                    status_label(&d.status)
                ),
                decision_id: Some(d.id.clone()),
                source: Some("sprint-status.yaml".to_string()),
                line: None,
                suggested_fix_kind: FixKind::LockDecision,
            }),
            _ => {}
        }
    }

    // Check 4: orphaned references (report each distinct orphan once, at its first site).
    let mut reported: HashSet<u32> = HashSet::new();
    for r in &references {
        if !defined_nums.contains(&r.num) && reported.insert(r.num) {
            // File references get a line number; git-commit references already carry a hash.
            let is_git = r.source.starts_with("git commit");
            let loc = if is_git {
                r.source.clone()
            } else {
                format!("{}:{}", r.source, r.line)
            };
            findings.push(Finding {
                severity: Severity::Error,
                code: Code::OrphanedReference,
                message: format!("orphaned reference {} in {loc} - not found in DECISIONS.md", r.raw),
                decision_id: Some(r.raw.clone()),
                source: Some(r.source.clone()),
                line: if is_git { None } else { Some(r.line) },
                suggested_fix_kind: FixKind::FixReference,
            });
        }
    }

    // Check 5: supersede-chain integrity.
    for d in &decisions {
        if let Status::Superseded { by } = &d.status {
            match by {
                None => findings.push(Finding {
                    severity: Severity::Error,
                    code: Code::SupersededWithoutSuccessor,
                    message: format!("{} '{}' is SUPERSEDED but names no successor", d.id, d.title),
                    decision_id: Some(d.id.clone()),
                    source: Some("DECISIONS.md".to_string()),
                    line: None,
                    suggested_fix_kind: FixKind::NameSuccessor,
                }),
                Some(succ) => {
                    if dnum(succ).is_none_or(|n| !defined_nums.contains(&n)) {
                        findings.push(Finding {
                            severity: Severity::Error,
                            code: Code::BrokenSupersedeChain,
                            message: format!(
                                "{} '{}' is SUPERSEDED by {}, which does not exist",
                                d.id, d.title, succ
                            ),
                            decision_id: Some(d.id.clone()),
                            source: Some("DECISIONS.md".to_string()),
                            line: None,
                            suggested_fix_kind: FixKind::NameSuccessor,
                        });
                    }
                }
            }
        }
    }

    // Check 6: dangling LOCKED decisions (warning).
    for d in &decisions {
        if d.status == Status::Locked && !referenced_nums.contains(&d.num) {
            findings.push(Finding {
                severity: Severity::Warning,
                code: Code::DanglingLocked,
                message: format!(
                    "{} '{}' is LOCKED but is referenced nowhere in the repo",
                    d.id, d.title
                ),
                decision_id: Some(d.id.clone()),
                source: None, // referenced nowhere - there is no source to cite
                line: None,
                suggested_fix_kind: FixKind::AddReference,
            });
        }
    }

    // Compute outcome (identical across formats).
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
    let exit_reason = if errors > 0 {
        ExitReason::Errors
    } else if strict && warnings > 0 {
        ExitReason::StrictWarnings
    } else {
        ExitReason::Passed
    };

    match format {
        // JSON mode: only the report goes to stdout. Read/parse errors surface via `Err` -> stderr.
        Format::Json => {
            let report = Report {
                schema_version: SCHEMA_VERSION,
                ok,
                strict,
                summary: Summary {
                    decisions_defined: decisions.len(),
                    referenced: referenced_defined,
                    errors,
                    warnings,
                },
                findings,
                exit_reason,
            };
            println!("{}", serde_json::to_string_pretty(&report)?);
        }
        Format::Human => {
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
        }
    }

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
