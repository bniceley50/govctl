//! Guard against version-pin drift.
//!
//! govctl is a drift detector that — twice, in external review — was caught with stale version
//! pins in its own action/install/docs. govctl can't catch this itself (it tracks decision
//! references, not version strings), so we enforce it the way Codex recommended: a test that reads
//! the crate version and asserts every pinned `vX.Y.Z` in the action, the installer fallback, and
//! the drop-in CI example matches it. Fails CI on any future drift.

use std::fs;
use std::path::PathBuf;

fn read(rel: &str) -> String {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel);
    fs::read_to_string(&p).unwrap_or_else(|e| panic!("reading {}: {e}", p.display()))
}

/// Extract every `vMAJOR.MINOR.PATCH` token (boundary-anchored) from `text`.
fn version_pins(text: &str) -> Vec<String> {
    let b: Vec<char> = text.chars().collect();
    let n = b.len();
    let mut out = Vec::new();
    let mut i = 0;
    while i < n {
        let boundary = i == 0 || !b[i - 1].is_ascii_alphanumeric();
        if b[i] == 'v' && boundary {
            let mut j = i + 1;
            let mut ok = true;
            for seg in 0..3 {
                let start = j;
                while j < n && b[j].is_ascii_digit() {
                    j += 1;
                }
                if j == start {
                    ok = false;
                    break;
                }
                if seg < 2 {
                    if j < n && b[j] == '.' {
                        j += 1;
                    } else {
                        ok = false;
                        break;
                    }
                }
            }
            if ok {
                out.push(b[i..j].iter().collect::<String>());
                i = j;
                continue;
            }
        }
        i += 1;
    }
    out
}

#[test]
fn pinned_versions_match_crate_version() {
    let want = format!("v{}", env!("CARGO_PKG_VERSION"));

    // In these files EVERY version pin must equal the current crate version.
    let strict = [
        ".github/actions/govctl-validate/action.yml",
        ".github/actions/govctl-validate/install.sh",
        ".github/governance-dropin.yml",
    ];
    for f in strict {
        let pins = version_pins(&read(f));
        assert!(!pins.is_empty(), "{f}: expected a version pin {want}, found none");
        for pin in &pins {
            assert_eq!(pin, &want, "{f}: pins {pin} but crate is {want} (version drift)");
        }
    }

    // The README CI example must reference the action at the current version.
    let readme = read("README.md");
    assert!(
        readme.contains(&format!("govctl-validate@{want}")),
        "README CI example must reference govctl-validate@{want}"
    );
}

#[test]
fn version_pin_extractor_is_sane() {
    assert_eq!(version_pins("uses foo@v0.3.3\n"), vec!["v0.3.3"]);
    assert_eq!(version_pins("default: \"v0.2.0\""), vec!["v0.2.0"]);
    assert_eq!(version_pins("end of sentence v1.2.3."), vec!["v1.2.3"]); // trailing dot excluded
    assert!(version_pins("nodev1.2.3").is_empty()); // not on a word boundary
    assert!(version_pins("v1.2").is_empty()); // needs three segments
}
