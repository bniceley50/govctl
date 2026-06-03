//! govctl - governance-stack scaffolder and decision-drift detector.
//!
//! Two commands:
//!   * `init`     - scaffold the seven-file governance stack into a project.
//!   * `validate` - check that the decision log and the actual repo have not drifted apart.
//!
//! See DECISIONS.md for the architectural decisions this tool both implements and enforces.

mod commands;
mod decisions;
mod repo_scan;
mod templates;

use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::process::ExitCode;

#[derive(Parser)]
#[command(
    name = "govctl",
    version,
    about = "Scaffold and enforce a governance stack (CLAUDE.md, DECISIONS.md, sprint-status.yaml, ...)"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Scaffold the full governance stack into a directory.
    Init {
        /// Target directory (created if missing). Defaults to the current directory.
        #[arg(default_value = ".")]
        path: PathBuf,

        /// Project name substituted into the templates.
        #[arg(long)]
        project_name: Option<String>,

        /// Overwrite existing governance files instead of refusing.
        #[arg(long)]
        force: bool,

        /// Print what would be written without touching the filesystem.
        #[arg(long)]
        dry_run: bool,
    },

    /// Check the governance stack for drift against the actual repo.
    Validate {
        /// Project directory to validate. Defaults to the current directory.
        #[arg(default_value = ".")]
        path: PathBuf,

        /// Treat warnings as failures (suitable for CI).
        #[arg(long)]
        strict: bool,
    },
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let result = match cli.command {
        Command::Init {
            path,
            project_name,
            force,
            dry_run,
        } => commands::init::run(&path, project_name.as_deref(), force, dry_run).map(|_| true),
        Command::Validate { path, strict } => commands::validate::run(&path, strict),
    };

    match result {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => ExitCode::FAILURE,
        Err(e) => {
            eprintln!("error: {e:#}");
            ExitCode::FAILURE
        }
    }
}
