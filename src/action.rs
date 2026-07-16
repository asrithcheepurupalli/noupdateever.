//! The sealed action allowlist — recipe format v1.
//!
//! Recipes are data, not code: every recipe compiles down to a list of these
//! seven declarative actions and nothing else. A malicious or buggy recipe
//! cannot do anything this enum cannot express. Adding a variant is a format
//! revision (FORMATS.md) and a product decision, not a convenience.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Action {
    /// Set a Windows service's start type to Disabled.
    DisableService { service: String },

    /// Disable a scheduled task. Google/Edge updater tasks carry
    /// machine-specific suffixes, hence prefix matching.
    DisableScheduledTask {
        task: String,
        #[serde(rename = "match", default)]
        match_mode: MatchMode,
    },

    /// Set a registry value (vendor-documented update policies live here).
    SetRegistryValue {
        hive: Hive,
        key: String,
        value: String,
        #[serde(rename = "type")]
        value_type: RegType,
        data: RegData,
    },

    /// Set one top-level key in a JSON config file (e.g. Discord's
    /// SKIP_HOST_UPDATE). Format v1 supports single-level pointers only.
    SetJsonValue {
        file: String,
        pointer: String,
        value: serde_json::Value,
    },

    /// Move an updater executable aside and leave an inert marker in its
    /// place. Exactly reversible via the journal.
    StubExecutable { path: String },

    /// Occupy a path with a read-only file so an updater cannot recreate its
    /// working directory there (the Spotify technique).
    BlockPath { path: String },

    /// Windows Firewall outbound block for one program.
    FirewallBlockProgram { program: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MatchMode {
    #[default]
    Exact,
    Prefix,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Hive {
    HKLM,
    HKCU,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RegType {
    Dword,
    String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RegData {
    Dword(u32),
    String(String),
}

impl Action {
    /// One-line human description, used by scan/dry-run output and the journal.
    pub fn describe(&self) -> String {
        match self {
            Action::DisableService { service } => {
                format!("disable service `{service}`")
            }
            Action::DisableScheduledTask { task, match_mode } => match match_mode {
                MatchMode::Exact => format!("disable scheduled task `{task}`"),
                MatchMode::Prefix => format!("disable scheduled tasks starting with `{task}`"),
            },
            Action::SetRegistryValue { hive, key, value, data, .. } => {
                let d = match data {
                    RegData::Dword(n) => n.to_string(),
                    RegData::String(s) => format!("\"{s}\""),
                };
                format!("set registry {hive:?}\\{key}\\{value} = {d}")
            }
            Action::SetJsonValue { file, pointer, value } => {
                format!("set JSON {file} {pointer} = {value}")
            }
            Action::StubExecutable { path } => format!("stub executable {path}"),
            Action::BlockPath { path } => format!("block path {path}"),
            Action::FirewallBlockProgram { program } => {
                format!("firewall: block outbound for {program}")
            }
        }
    }

    /// Structural validation beyond what serde enforces.
    pub fn validate(&self) -> Result<(), String> {
        match self {
            Action::DisableService { service } if service.trim().is_empty() => {
                Err("disable_service: empty service name".into())
            }
            Action::DisableScheduledTask { task, .. } if task.trim().is_empty() => {
                Err("disable_scheduled_task: empty task name".into())
            }
            Action::SetRegistryValue { key, value, value_type, data, .. } => {
                if key.trim().is_empty() || value.trim().is_empty() {
                    return Err("set_registry_value: empty key or value name".into());
                }
                match (value_type, data) {
                    (RegType::Dword, RegData::Dword(_)) => Ok(()),
                    (RegType::String, RegData::String(_)) => Ok(()),
                    _ => Err("set_registry_value: `type` does not match `data`".into()),
                }
            }
            Action::SetJsonValue { file, pointer, .. } => {
                if file.trim().is_empty() {
                    return Err("set_json_value: empty file".into());
                }
                // Format v1: single-level pointer only, e.g. "/SKIP_HOST_UPDATE".
                if !pointer.starts_with('/') || pointer.len() < 2 || pointer[1..].contains('/') {
                    return Err(format!(
                        "set_json_value: pointer `{pointer}` must be a single-level JSON pointer like /KEY"
                    ));
                }
                Ok(())
            }
            Action::StubExecutable { path } | Action::BlockPath { path }
                if path.trim().is_empty() =>
            {
                Err("stub_executable/block_path: empty path".into())
            }
            Action::FirewallBlockProgram { program } if program.trim().is_empty() => {
                Err("firewall_block_program: empty program".into())
            }
            _ => Ok(()),
        }
    }
}

/// The undo record captured when an action is applied. Every apply MUST
/// produce one; `stasis thaw` replays them in reverse. This is covenant
/// material: everything Stasis does to a machine is exactly reversible.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Undo {
    ServiceStart { service: String, prior_start: u32 },
    TaskWasEnabled { task: String },
    RegistryValue {
        hive: Hive,
        key: String,
        value: String,
        prior: Option<RegData>,
        key_existed: bool,
    },
    JsonValue {
        file: String,
        pointer: String,
        prior: Option<serde_json::Value>,
        file_existed: bool,
    },
    MovedExecutable { original: String, moved_to: String },
    CreatedBlocker { path: String },
    FirewallRule { rule: String },
    /// Dry runs and no-ops (e.g. task already disabled) record this.
    Nothing,
}
