//! Inventory: the Change Surface Report (SPEC §4 F1).
//!
//! The probe answers "what is on this machine"; the report answers "what can
//! change by itself, and do we know how to stop it". Anything we can't map to
//! a recipe is reported as unknown — the report is honest before it is pretty.

use crate::recipe::RecipeFile;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MachineFacts {
    /// Installed applications: (display name, version).
    pub apps: Vec<(String, String)>,
    /// Windows services: (service name, display name).
    pub services: Vec<(String, String)>,
    /// Scheduled task names (full paths, e.g. "\\GoogleUpdateTaskMachineCore{...}").
    pub tasks: Vec<String>,
    /// Directories under %LOCALAPPDATA% that contain a Squirrel `Update.exe`.
    pub squirrel_dirs: Vec<String>,
}

pub trait SystemProbe {
    fn facts(&self) -> Result<MachineFacts, String>;
}

/// Probe from a JSON fixture — the test/dev probe on every platform, and a
/// support tool ("send me your scan fixture") later.
pub struct FixtureProbe {
    pub facts: MachineFacts,
}

impl FixtureProbe {
    pub fn from_file(path: &std::path::Path) -> Result<Self, String> {
        let src = std::fs::read_to_string(path)
            .map_err(|e| format!("cannot read fixture {}: {e}", path.display()))?;
        let facts: MachineFacts =
            serde_json::from_str(&src).map_err(|e| format!("fixture: {e}"))?;
        Ok(Self { facts })
    }
}

impl SystemProbe for FixtureProbe {
    fn facts(&self) -> Result<MachineFacts, String> {
        Ok(self.facts.clone())
    }
}

/// Updater fingerprints the scanner recognizes even without a recipe match.
/// (name-substring, mechanism label, recipe id if we ship one)
const SERVICE_FINGERPRINTS: &[(&str, &str)] = &[
    ("gupdate", "chrome"),
    ("gupdatem", "chrome"),
    ("edgeupdate", "edge"),
    ("edgeupdatem", "edge"),
    ("MozillaMaintenance", "firefox"),
];

const TASK_FINGERPRINTS: &[(&str, &str)] = &[
    ("GoogleUpdateTaskMachine", "chrome"),
    ("MicrosoftEdgeUpdateTask", "edge"),
];

const SQUIRREL_FINGERPRINTS: &[(&str, &str)] = &[
    ("slack", "slack"),
    ("Discord", "discord"),
    ("GitHubDesktop", "github-desktop"),
];

#[derive(Debug, Serialize)]
pub struct Finding {
    pub mechanism: String,
    pub detail: String,
    /// Recipe that silences it, if the catalog has one.
    pub recipe: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct Report {
    pub apps_total: usize,
    pub findings: Vec<Finding>,
    /// Update mechanisms we saw but cannot map to any recipe. Honesty section.
    pub unknown: Vec<Finding>,
}

pub fn build_report(probe: &dyn SystemProbe, catalog: &[RecipeFile]) -> Result<Report, String> {
    let facts = probe.facts()?;
    let has_recipe = |id: &str| catalog.iter().any(|r| r.recipe.id == id);
    let mut findings = Vec::new();
    let mut unknown = Vec::new();

    for (name, display) in &facts.services {
        if let Some((_, rid)) = SERVICE_FINGERPRINTS
            .iter()
            .find(|(pat, _)| name.to_lowercase().contains(&pat.to_lowercase()))
        {
            let f = Finding {
                mechanism: "updater service".into(),
                detail: format!("{name} ({display})"),
                recipe: has_recipe(rid).then(|| rid.to_string()),
            };
            if f.recipe.is_some() { findings.push(f) } else { unknown.push(f) }
        } else if name.to_lowercase().contains("update") {
            unknown.push(Finding {
                mechanism: "updater service (unrecognized)".into(),
                detail: format!("{name} ({display})"),
                recipe: None,
            });
        }
    }

    for task in &facts.tasks {
        if let Some((_, rid)) =
            TASK_FINGERPRINTS.iter().find(|(pat, _)| task.contains(pat))
        {
            let f = Finding {
                mechanism: "updater scheduled task".into(),
                detail: task.clone(),
                recipe: has_recipe(rid).then(|| rid.to_string()),
            };
            if f.recipe.is_some() { findings.push(f) } else { unknown.push(f) }
        } else if task.to_lowercase().contains("update") {
            unknown.push(Finding {
                mechanism: "updater scheduled task (unrecognized)".into(),
                detail: task.clone(),
                recipe: None,
            });
        }
    }

    for dir in &facts.squirrel_dirs {
        if let Some((_, rid)) = SQUIRREL_FINGERPRINTS
            .iter()
            .find(|(pat, _)| dir.to_lowercase().contains(&pat.to_lowercase()))
        {
            let f = Finding {
                mechanism: "Squirrel self-updater".into(),
                detail: format!("{dir}\\Update.exe"),
                recipe: has_recipe(rid).then(|| rid.to_string()),
            };
            if f.recipe.is_some() { findings.push(f) } else { unknown.push(f) }
        } else {
            unknown.push(Finding {
                mechanism: "Squirrel self-updater (unrecognized app)".into(),
                detail: format!("{dir}\\Update.exe"),
                recipe: None,
            });
        }
    }

    // Apps matched by recipe detect rules that surfaced no mechanism above
    // still get a finding — the recipe knows machinery the probe can't see
    // (in-app updaters like VS Code's or Spotify's).
    for r in catalog {
        let already = findings.iter().any(|f| f.recipe.as_deref() == Some(&r.recipe.id));
        if already {
            continue;
        }
        let installed = facts.apps.iter().any(|(name, _)| {
            r.detect
                .uninstall_names
                .iter()
                .any(|pat| name.to_lowercase().contains(&pat.to_lowercase()))
        });
        if installed {
            findings.push(Finding {
                mechanism: "in-app updater".into(),
                detail: r.recipe.name.clone(),
                recipe: Some(r.recipe.id.clone()),
            });
        }
    }

    Ok(Report { apps_total: facts.apps.len(), findings, unknown })
}
