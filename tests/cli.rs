//! End-to-end CLI tests: run the real `govctl` binary against temp directories.

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::path::Path;

fn govctl() -> Command {
    Command::cargo_bin("govctl").expect("binary builds")
}

/// Write a minimal valid governance stack into `dir`, then return its path.
/// `decisions` is the body of DECISIONS.md; the rest are stubs sufficient for validate.
fn scaffold(dir: &Path, decisions: &str, sprint_honors: &str) {
    for name in [
        "CLAUDE.md",
        "AGENTS.md",
        "RED_TEAM.md",
        "RUNBOOK.md",
        "lessons.md",
    ] {
        fs::write(dir.join(name), format!("# {name}\n")).unwrap();
    }
    fs::write(dir.join("DECISIONS.md"), decisions).unwrap();
    fs::write(
        dir.join("sprint-status.yaml"),
        format!("project: t\nsprint: 1\nhonors_decisions: {sprint_honors}\n"),
    )
    .unwrap();
}

#[test]
fn init_creates_all_files() {
    let tmp = tempfile::tempdir().unwrap();
    govctl()
        .args(["init", "."])
        .current_dir(tmp.path())
        .assert()
        .success();

    for name in [
        "CLAUDE.md",
        "AGENTS.md",
        "DECISIONS.md",
        "RED_TEAM.md",
        "RUNBOOK.md",
        "sprint-status.yaml",
        "lessons.md",
        ".govctlignore",
    ] {
        assert!(tmp.path().join(name).exists(), "{name} should exist");
    }
    assert!(tmp.path().join(".govctl").join("manifest.toml").exists());
}

#[test]
fn init_refuses_to_clobber_then_force_overwrites() {
    let tmp = tempfile::tempdir().unwrap();
    govctl().args(["init", "."]).current_dir(tmp.path()).assert().success();

    // Second run without --force must fail and mention overwrite.
    govctl()
        .args(["init", "."])
        .current_dir(tmp.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("refusing to overwrite"));

    // With --force it succeeds.
    govctl()
        .args(["init", ".", "--force"])
        .current_dir(tmp.path())
        .assert()
        .success();
}

#[test]
fn init_merge_adds_only_missing_and_preserves_existing() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path();
    // Simulate a project with a partial, hand-authored stack.
    let custom = "# my real decisions\n\n## D001 - Keep me\n- **Status:** LOCKED\n";
    fs::write(dir.join("DECISIONS.md"), custom).unwrap();
    fs::write(dir.join("CLAUDE.md"), "# my real CLAUDE\n").unwrap();

    govctl()
        .args(["init", ".", "--merge"])
        .current_dir(dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("added"))
        .stdout(predicate::str::contains("kept (unchanged)"));

    // Existing files are byte-for-byte preserved.
    assert_eq!(fs::read_to_string(dir.join("DECISIONS.md")).unwrap(), custom);
    assert_eq!(fs::read_to_string(dir.join("CLAUDE.md")).unwrap(), "# my real CLAUDE\n");
    // Missing files were added.
    for name in ["AGENTS.md", "RED_TEAM.md", "RUNBOOK.md", "sprint-status.yaml", "lessons.md", ".govctlignore"] {
        assert!(dir.join(name).exists(), "{name} should have been added");
    }
}

#[test]
fn init_merge_dry_run_writes_nothing() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path();
    fs::write(dir.join("DECISIONS.md"), "# existing\n").unwrap();

    govctl()
        .args(["init", ".", "--merge", "--dry-run"])
        .current_dir(dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("would add"))
        .stdout(predicate::str::contains("keep"));

    assert!(!dir.join("AGENTS.md").exists(), "dry-run must not write");
}

#[test]
fn short_d_notation_is_not_treated_as_a_reference() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path();
    // Decisions use the 3-digit convention.
    scaffold(dir, "# Decisions\n\n## D001 - Real\n- **Status:** LOCKED\n", "[]");
    // Analytics-style Day-N markers must NOT be read as references to decisions 1/7/30/...
    fs::write(dir.join("notes.md"), "Retention cohort: D1 D7 D14 D30 D60 D90\n").unwrap();

    govctl()
        .args(["validate", "."])
        .current_dir(dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("0 error(s)"));
}

#[test]
fn pnpm_lockfile_is_ignored_by_default() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path();
    // No .govctlignore is written, so this relies on the built-in defaults.
    scaffold(dir, "# Decisions\n\n## D001 - Real\n- **Status:** LOCKED\n", "[]");
    // A hash fragment that looks like a 3-digit decision reference, in a pnpm lockfile.
    fs::write(dir.join("pnpm-lock.yaml"), "  /pkg@1.0.0:\n    resolution: {integrity: D090deadbeef}\n").unwrap();

    govctl()
        .args(["validate", "."])
        .current_dir(dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("0 error(s)"));
}

#[test]
fn init_merge_and_force_conflict() {
    let tmp = tempfile::tempdir().unwrap();
    govctl()
        .args(["init", ".", "--merge", "--force"])
        .current_dir(tmp.path())
        .assert()
        .failure();
}

#[test]
fn init_dry_run_writes_nothing() {
    let tmp = tempfile::tempdir().unwrap();
    govctl()
        .args(["init", ".", "--dry-run"])
        .current_dir(tmp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("dry run"));
    assert!(!tmp.path().join("DECISIONS.md").exists());
}

#[test]
fn freshly_scaffolded_project_validates_clean() {
    let tmp = tempfile::tempdir().unwrap();
    govctl().args(["init", "."]).current_dir(tmp.path()).assert().success();

    govctl()
        .args(["validate", "."])
        .current_dir(tmp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("validation passed"));
}

#[test]
fn orphaned_reference_in_source_fails() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path();
    scaffold(
        dir,
        "# Decisions\n\n### D001 - Real\n- **Status:** LOCKED\n",
        "[]",
    );
    // A source file cites a decision that was never logged.
    fs::create_dir_all(dir.join("src")).unwrap();
    fs::write(dir.join("src").join("lib.rs"), "// implements D207\n").unwrap();

    govctl()
        .args(["validate", "."])
        .current_dir(dir)
        .assert()
        .failure()
        .stdout(predicate::str::contains("orphaned reference D207"))
        .stdout(predicate::str::contains("validation failed"));
}

#[test]
fn broken_supersede_chain_fails() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path();
    scaffold(
        dir,
        "# Decisions\n\n### D003 - Old\n- **Status:** SUPERSEDED (by D099)\n",
        "[]",
    );
    govctl()
        .args(["validate", "."])
        .current_dir(dir)
        .assert()
        .failure()
        .stdout(predicate::str::contains("SUPERSEDED by D099"));
}

#[test]
fn cyclic_supersede_chain_fails() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path();
    scaffold(
        dir,
        "# Decisions\n\n### D001 - Loops\n- **Status:** SUPERSEDED (by D001)\n",
        "[]",
    );
    govctl()
        .args(["validate", "."])
        .current_dir(dir)
        .assert()
        .failure()
        .stdout(predicate::str::contains("supersede cycle"));
}

#[test]
fn two_decision_supersede_cycle_fails() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path();
    scaffold(
        dir,
        "# Decisions\n\n### D001 - Old\n- **Status:** SUPERSEDED (by D002)\n\n### D002 - Also old\n- **Status:** SUPERSEDED (by D001)\n",
        "[]",
    );
    govctl()
        .args(["validate", "."])
        .current_dir(dir)
        .assert()
        .failure()
        .stdout(predicate::str::contains("supersede cycle"));
}

#[test]
fn honoring_non_locked_decision_fails() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path();
    scaffold(
        dir,
        "# Decisions\n\n### D001 - Proposed thing\n- **Status:** PROPOSED\n",
        "[D001]",
    );
    govctl()
        .args(["validate", "."])
        .current_dir(dir)
        .assert()
        .failure()
        .stdout(predicate::str::contains("not LOCKED"));
}

#[test]
fn honoring_unlocked_decision_fails() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path();
    scaffold(
        dir,
        "# Decisions\n\n### D001 - Explicitly not locked\n- **Status:** unlocked\n",
        "[D001]",
    );
    govctl()
        .args(["validate", "."])
        .current_dir(dir)
        .assert()
        .failure()
        .stdout(predicate::str::contains("not LOCKED"));
}

#[test]
fn oversized_honored_decision_id_does_not_alias_d0() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path();
    scaffold(
        dir,
        "# Decisions\n\n### D0 - Zero\n- **Status:** LOCKED\n",
        "[D4294967296]",
    );
    govctl()
        .args(["validate", "."])
        .current_dir(dir)
        .assert()
        .failure()
        .stdout(predicate::str::contains("honors D4294967296"))
        .stdout(predicate::str::contains("not defined"));
}

#[test]
fn dangling_locked_is_warning_then_strict_fails() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path();
    // D050 is LOCKED but referenced nowhere.
    scaffold(
        dir,
        "# Decisions\n\n### D050 - Ambient rule\n- **Status:** LOCKED\n",
        "[]",
    );

    // Non-strict: a warning, but validation still passes.
    govctl()
        .args(["validate", "."])
        .current_dir(dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("WARN"))
        .stdout(predicate::str::contains("validation passed"));

    // Strict: the warning becomes a failure.
    govctl()
        .args(["validate", ".", "--strict"])
        .current_dir(dir)
        .assert()
        .failure()
        .stdout(predicate::str::contains("validation failed"));
}

#[test]
fn orphaned_reference_in_git_commit_fails() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path();
    scaffold(
        dir,
        "# Decisions\n\n### D001 - Real\n- **Status:** LOCKED\n",
        "[]",
    );

    // Make a git repo with a commit message citing an unlogged decision.
    let git = |args: &[&str]| {
        std::process::Command::new("git")
            .args(args)
            .current_dir(dir)
            .output()
            .expect("git available");
    };
    git(&["init", "-q"]);
    git(&["config", "user.email", "t@example.com"]);
    git(&["config", "user.name", "Test"]);
    git(&["add", "-A"]);
    git(&["commit", "-q", "-m", "wire up feature, honors D310"]);

    govctl()
        .args(["validate", "."])
        .current_dir(dir)
        .assert()
        .failure()
        .stdout(predicate::str::contains("orphaned reference D310"))
        .stdout(predicate::str::contains("git commit"));
}

#[test]
fn validate_json_clean_parses_and_reports_passed() {
    let tmp = tempfile::tempdir().unwrap();
    govctl().args(["init", "."]).current_dir(tmp.path()).assert().success();

    let assert = govctl()
        .args(["validate", ".", "--format", "json"])
        .current_dir(tmp.path())
        .assert()
        .success();

    // Parse stdout AS JSON (not a string-contains), per the contract.
    let v: serde_json::Value =
        serde_json::from_slice(&assert.get_output().stdout).expect("stdout must be valid JSON");
    assert_eq!(v["schemaVersion"], 1);
    assert_eq!(v["ok"], true);
    assert_eq!(v["exitReason"], "PASSED");
    assert!(v["findings"].as_array().unwrap().is_empty());
}

#[test]
fn validate_json_failing_emits_structured_finding() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path();
    scaffold(dir, "# Decisions\n\n## D001 - Real\n- **Status:** LOCKED\n", "[]");
    fs::create_dir_all(dir.join("src")).unwrap();
    fs::write(dir.join("src").join("lib.rs"), "// implements D207\n").unwrap();

    let assert = govctl()
        .args(["validate", ".", "--format", "json"])
        .current_dir(dir)
        .assert()
        .failure(); // exit code identical to human mode

    let v: serde_json::Value =
        serde_json::from_slice(&assert.get_output().stdout).expect("valid JSON even on failure");
    assert_eq!(v["ok"], false);
    assert_eq!(v["exitReason"], "ERRORS");
    let findings = v["findings"].as_array().unwrap();
    let orphan = findings
        .iter()
        .find(|f| f["code"] == "ORPHANED_REFERENCE")
        .expect("an ORPHANED_REFERENCE finding");
    assert_eq!(orphan["decisionId"], "D207");
    assert_eq!(orphan["suggestedFixKind"], "FIX_REFERENCE");
    assert_eq!(orphan["source"], "src/lib.rs");
    assert_eq!(orphan["line"], 1);
}

#[test]
fn validate_json_strict_warning_has_exit_reason() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path();
    // A LOCKED decision referenced nowhere -> a warning, which --strict turns into a failure.
    scaffold(dir, "# Decisions\n\n## D050 - Ambient\n- **Status:** LOCKED\n", "[]");

    let assert = govctl()
        .args(["validate", ".", "--format", "json", "--strict"])
        .current_dir(dir)
        .assert()
        .failure();

    let v: serde_json::Value =
        serde_json::from_slice(&assert.get_output().stdout).expect("valid JSON");
    assert_eq!(v["ok"], false);
    assert_eq!(v["exitReason"], "STRICT_WARNINGS");
    let f = &v["findings"].as_array().unwrap()[0];
    assert_eq!(f["code"], "DANGLING_LOCKED");
    assert_eq!(f["severity"], "warning");
    assert!(f["source"].is_null()); // dangling = referenced nowhere -> null source
}
