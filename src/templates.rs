//! Embedded governance templates.
//!
//! The seven canonical files are compiled into the binary via `include_str!`, so `govctl`
//! is a true single-file binary with no runtime template directory to ship alongside it.
//! Placeholders `{{PROJECT_NAME}}` and `{{DATE}}` are substituted at scaffold time.

/// One scaffolded file: its relative path and its rendered-template source.
pub struct Template {
    pub filename: &'static str,
    pub body: &'static str,
}

/// The seven canonical governance files, in scaffold order.
pub const TEMPLATES: &[Template] = &[
    Template {
        filename: "CLAUDE.md",
        body: include_str!("../templates/CLAUDE.md"),
    },
    Template {
        filename: "AGENTS.md",
        body: include_str!("../templates/AGENTS.md"),
    },
    Template {
        filename: "DECISIONS.md",
        body: include_str!("../templates/DECISIONS.md"),
    },
    Template {
        filename: "RED_TEAM.md",
        body: include_str!("../templates/RED_TEAM.md"),
    },
    Template {
        filename: "RUNBOOK.md",
        body: include_str!("../templates/RUNBOOK.md"),
    },
    Template {
        filename: "sprint-status.yaml",
        body: include_str!("../templates/sprint-status.yaml"),
    },
    Template {
        filename: "lessons.md",
        body: include_str!("../templates/lessons.md"),
    },
];

/// The default `.govctlignore` scaffolded by `init`. Not a governance file - config that keeps
/// `validate` from reading build artifacts and test fixtures as real decision references.
pub const GOVCTLIGNORE: &str = include_str!("../templates/govctlignore.default");

/// Substitute `{{PROJECT_NAME}}` and `{{DATE}}` placeholders in a template body.
pub fn render(body: &str, project_name: &str, date: &str) -> String {
    body.replace("{{PROJECT_NAME}}", project_name)
        .replace("{{DATE}}", date)
}

/// The seven canonical governance filenames (used by `validate` for the presence check).
pub fn governance_filenames() -> Vec<&'static str> {
    TEMPLATES.iter().map(|t| t.filename).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn there_are_seven_templates() {
        assert_eq!(TEMPLATES.len(), 7);
    }

    #[test]
    fn render_substitutes_both_placeholders() {
        let out = render("# {{PROJECT_NAME}} on {{DATE}}", "Acme", "2026-06-03");
        assert_eq!(out, "# Acme on 2026-06-03");
        assert!(!out.contains("{{"));
    }

    #[test]
    fn every_template_is_nonempty() {
        for t in TEMPLATES {
            assert!(!t.body.trim().is_empty(), "{} is empty", t.filename);
        }
    }

    #[test]
    fn sprint_status_template_parses_as_yaml() {
        let body = render(
            TEMPLATES
                .iter()
                .find(|t| t.filename == "sprint-status.yaml")
                .unwrap()
                .body,
            "Demo",
            "2026-06-03",
        );
        let parsed: serde_yaml::Value = serde_yaml::from_str(&body).expect("valid YAML");
        assert!(parsed.is_mapping());
    }
}
