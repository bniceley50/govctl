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
