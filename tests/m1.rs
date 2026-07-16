//! M1 acceptance tests (SPEC §10):
//! - every embedded recipe parses and validates
//! - planning resolves variables and covers the acceptance apps
//! - the scan report is complete and honest about unknowns
//!
//! System actions (services/registry/firewall) are exercised on a real
//! Windows machine in the M1 verification pass; their planning, journaling,
//! and dry-run paths are covered here on any OS.

use std::path::{Path, PathBuf};
use std::process::Command;

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_stasis")
}

fn run(args: &[&str]) -> (bool, String, String) {
    // Planning resolves Windows %VAR%s; provide them the way a Windows
    // session would so dry-run plans are testable on any OS.
    let out = Command::new(bin())
        .args(args)
        .env("LOCALAPPDATA", "C:\\Users\\dev\\AppData\\Local")
        .env("APPDATA", "C:\\Users\\dev\\AppData\\Roaming")
        .env("ProgramFiles", "C:\\Program Files")
        .env("ProgramFiles(x86)", "C:\\Program Files (x86)")
        .env("ProgramData", "C:\\ProgramData")
        .output()
        .expect("run stasis");
    (
        out.status.success(),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

fn fixture() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/dev-machine.json")
}

#[test]
fn catalog_lists_all_ten_recipes() {
    let (ok, stdout, _) = run(&["recipes"]);
    assert!(ok);
    for id in [
        "chrome", "edge", "firefox", "vscode", "slack", "discord", "spotify", "zoom",
        "notepad-plus-plus", "github-desktop",
    ] {
        assert!(stdout.contains(id), "catalog missing `{id}`:\n{stdout}");
    }
    assert!(stdout.contains("10 recipe(s)"));
}

#[test]
fn scan_report_is_complete_and_honest() {
    let f = fixture();
    let (ok, stdout, _) = run(&["scan", "--fixture", f.to_str().unwrap()]);
    assert!(ok, "scan failed:\n{stdout}");
    // Mechanisms the catalog can silence, mapped to recipes.
    for needle in ["gupdate", "GoogleUpdateTaskMachineCore", "[chrome]", "[slack]", "[discord]"] {
        assert!(stdout.contains(needle), "scan missing `{needle}`:\n{stdout}");
    }
    // In-app updaters surfaced via detect rules even without a visible mechanism.
    assert!(stdout.contains("[vscode]"), "vscode in-app updater not reported:\n{stdout}");
    assert!(stdout.contains("[spotify]"), "spotify in-app updater not reported:\n{stdout}");
    // Honesty: unknown updaters must be reported, not swallowed.
    assert!(stdout.contains("Honesty section"), "no honesty section:\n{stdout}");
    assert!(stdout.contains("WidgetStudioUpdateSvc"), "unknown service not flagged:\n{stdout}");
    assert!(stdout.contains("SomeRandomApp"), "unknown Squirrel app not flagged:\n{stdout}");
}

#[test]
fn scan_json_output_parses() {
    let f = fixture();
    let (ok, stdout, _) = run(&["scan", "--fixture", f.to_str().unwrap(), "--json"]);
    assert!(ok);
    let v: serde_json::Value = serde_json::from_str(&stdout).expect("valid JSON");
    assert!(v["findings"].as_array().unwrap().len() >= 5);
    assert!(!v["unknown"].as_array().unwrap().is_empty());
}

#[test]
fn freeze_dry_run_plans_acceptance_apps() {
    // The M1 acceptance set: Chrome, VS Code, Slack, Spotify, Discord.
    let (ok, stdout, _) = run(&[
        "freeze", "chrome", "vscode", "slack", "spotify", "discord", "--dry-run",
    ]);
    assert!(ok, "dry-run failed:\n{stdout}");
    assert!(stdout.contains("DRY RUN"));
    for needle in [
        "disable service `gupdate`",
        "SOFTWARE\\Policies\\Google\\Update",
        "UpdateMode",
        "Update.exe",       // slack stub
        "SKIP_HOST_UPDATE", // discord json
        "Spotify",          // spotify block path
    ] {
        assert!(stdout.contains(needle), "plan missing `{needle}`:\n{stdout}");
    }
}

#[test]
fn freeze_unknown_recipe_fails_clearly() {
    let (ok, _, stderr) = run(&["freeze", "definitely-not-real", "--dry-run"]);
    assert!(!ok);
    assert!(stderr.contains("no recipe with id"));
}

#[test]
fn thaw_with_no_journal_is_a_noop() {
    let (ok, stdout, _) = run(&["thaw", "--dry-run"]);
    assert!(ok);
    // Fresh checkout has no ./stasis-data journal.
    assert!(
        stdout.contains("Nothing to thaw") || stdout.contains("outstanding change"),
        "unexpected thaw output:\n{stdout}"
    );
}
