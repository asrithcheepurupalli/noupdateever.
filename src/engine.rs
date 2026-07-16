//! Planning and execution.
//!
//! `plan()` turns recipes into resolved, human-auditable planned actions.
//! `LocalExecutor` applies them: file-based actions (stub, block, JSON) are
//! implemented portably right here and are exercised by tests on any OS;
//! system actions (services, tasks, registry, firewall) go through the
//! `SystemBackend` trait, whose only real implementation is Windows. Every
//! successful apply yields an `Undo` and is journaled by the caller.

use crate::action::{Action, Undo};
use crate::recipe::RecipeFile;
use crate::util::expand_vars;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct PlannedAction {
    pub recipe_id: String,
    /// The action with %VAR%s resolved — exactly what will be done.
    pub action: Action,
}

/// Resolve every %VAR% up front so the whole plan is auditable before
/// anything runs. A recipe whose variables don't resolve fails at plan time,
/// not halfway through an apply.
pub fn plan(
    recipes: &[&RecipeFile],
    lookup: &dyn Fn(&str) -> Option<String>,
) -> Result<Vec<PlannedAction>, String> {
    let mut out = Vec::new();
    for r in recipes {
        for a in &r.actions {
            let resolved = resolve_action(a, lookup)
                .map_err(|e| format!("recipe `{}`: {e}", r.recipe.id))?;
            out.push(PlannedAction { recipe_id: r.recipe.id.clone(), action: resolved });
        }
    }
    Ok(out)
}

fn resolve_action(
    a: &Action,
    lookup: &dyn Fn(&str) -> Option<String>,
) -> Result<Action, String> {
    let mut a = a.clone();
    match &mut a {
        Action::SetJsonValue { file, .. } => *file = expand_vars(file, lookup)?,
        Action::StubExecutable { path } | Action::BlockPath { path } => {
            *path = expand_vars(path, lookup)?
        }
        Action::FirewallBlockProgram { program } => *program = expand_vars(program, lookup)?,
        Action::DisableService { .. }
        | Action::DisableScheduledTask { .. }
        | Action::SetRegistryValue { .. } => {}
    }
    Ok(a)
}

/// Result of applying one planned action.
pub struct Applied {
    pub undo: Undo,
    pub note: String,
}

/// Windows-only machinery, abstracted so the engine compiles and tests everywhere.
pub trait SystemBackend {
    fn disable_service(&mut self, service: &str) -> Result<Applied, String>;
    fn disable_scheduled_task(
        &mut self,
        task: &str,
        prefix: bool,
    ) -> Result<Vec<Applied>, String>;
    fn set_registry_value(&mut self, action: &Action) -> Result<Applied, String>;
    fn firewall_block_program(&mut self, program: &str) -> Result<Applied, String>;
    fn revert_system(&mut self, undo: &Undo) -> Result<(), String>;
}

/// Backend for platforms without the Windows machinery. File-based actions
/// still work (they're portable); system actions refuse honestly.
/// (Unused when compiled for Windows itself — that build has the real one.)
#[allow(dead_code)]
pub struct UnsupportedBackend;

impl SystemBackend for UnsupportedBackend {
    fn disable_service(&mut self, _: &str) -> Result<Applied, String> {
        Err("services are a Windows mechanism; not available on this platform".into())
    }
    fn disable_scheduled_task(&mut self, _: &str, _: bool) -> Result<Vec<Applied>, String> {
        Err("scheduled tasks are a Windows mechanism; not available on this platform".into())
    }
    fn set_registry_value(&mut self, _: &Action) -> Result<Applied, String> {
        Err("the registry is a Windows mechanism; not available on this platform".into())
    }
    fn firewall_block_program(&mut self, _: &str) -> Result<Applied, String> {
        Err("Windows Firewall is not available on this platform".into())
    }
    fn revert_system(&mut self, _: &Undo) -> Result<(), String> {
        Err("system undo is not available on this platform".into())
    }
}

pub struct LocalExecutor<'a> {
    pub backend: &'a mut dyn SystemBackend,
}

pub const STUB_SUFFIX: &str = ".stasis-original";
pub const STUB_MARKER: &str = "This executable was set aside by Stasis, the update firewall.\n\
The original is next to this file with the extension `.stasis-original`.\n\
Run `stasis thaw` to restore it exactly.\n";

impl<'a> LocalExecutor<'a> {
    /// Apply one action. May yield several undos (prefix task matches).
    pub fn apply(&mut self, pa: &PlannedAction) -> Result<Vec<Applied>, String> {
        match &pa.action {
            Action::DisableService { service } => {
                Ok(vec![self.backend.disable_service(service)?])
            }
            Action::DisableScheduledTask { task, match_mode } => self
                .backend
                .disable_scheduled_task(task, *match_mode == crate::action::MatchMode::Prefix),
            Action::SetRegistryValue { .. } => {
                Ok(vec![self.backend.set_registry_value(&pa.action)?])
            }
            Action::FirewallBlockProgram { program } => {
                Ok(vec![self.backend.firewall_block_program(program)?])
            }
            Action::SetJsonValue { file, pointer, value } => {
                Ok(vec![apply_json(file, pointer, value)?])
            }
            Action::StubExecutable { path } => Ok(vec![apply_stub(path)?]),
            Action::BlockPath { path } => Ok(vec![apply_block(path)?]),
        }
    }

    pub fn revert(&mut self, undo: &Undo) -> Result<(), String> {
        match undo {
            Undo::JsonValue { file, pointer, prior, file_existed } => {
                revert_json(file, pointer, prior.as_ref(), *file_existed)
            }
            Undo::MovedExecutable { original, moved_to } => {
                revert_stub(original, moved_to)
            }
            Undo::CreatedBlocker { path } => revert_block(path),
            Undo::Nothing => Ok(()),
            Undo::ServiceStart { .. }
            | Undo::TaskWasEnabled { .. }
            | Undo::RegistryValue { .. }
            | Undo::FirewallRule { .. } => self.backend.revert_system(undo),
        }
    }
}

// ---- portable file-based actions -----------------------------------------

fn apply_json(file: &str, pointer: &str, value: &serde_json::Value) -> Result<Applied, String> {
    let key = &pointer[1..]; // validated single-level pointer "/KEY"
    let path = Path::new(file);
    let file_existed = path.exists();
    let mut doc: serde_json::Value = if file_existed {
        let src = fs::read_to_string(path).map_err(|e| format!("read {file}: {e}"))?;
        serde_json::from_str(&src).map_err(|e| format!("{file} is not valid JSON: {e}"))?
    } else {
        serde_json::Value::Object(Default::default())
    };
    let obj = doc
        .as_object_mut()
        .ok_or_else(|| format!("{file}: top level is not a JSON object"))?;
    let prior = obj.get(key).cloned();
    if prior.as_ref() == Some(value) {
        return Ok(Applied {
            undo: Undo::Nothing,
            note: format!("{file} {pointer} already set"),
        });
    }
    obj.insert(key.to_string(), value.clone());
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("create {}: {e}", parent.display()))?;
    }
    fs::write(path, serde_json::to_string_pretty(&doc).unwrap())
        .map_err(|e| format!("write {file}: {e}"))?;
    Ok(Applied {
        undo: Undo::JsonValue {
            file: file.to_string(),
            pointer: pointer.to_string(),
            prior,
            file_existed,
        },
        note: String::new(),
    })
}

fn revert_json(
    file: &str,
    pointer: &str,
    prior: Option<&serde_json::Value>,
    file_existed: bool,
) -> Result<(), String> {
    let path = Path::new(file);
    if !file_existed && prior.is_none() {
        // We created the file solely to hold this key; remove it again.
        if path.exists() {
            fs::remove_file(path).map_err(|e| format!("remove {file}: {e}"))?;
        }
        return Ok(());
    }
    let src = fs::read_to_string(path).map_err(|e| format!("read {file}: {e}"))?;
    let mut doc: serde_json::Value =
        serde_json::from_str(&src).map_err(|e| format!("{file}: {e}"))?;
    let obj = doc
        .as_object_mut()
        .ok_or_else(|| format!("{file}: top level is not a JSON object"))?;
    let key = &pointer[1..];
    match prior {
        Some(v) => {
            obj.insert(key.to_string(), v.clone());
        }
        None => {
            obj.remove(key);
        }
    }
    fs::write(path, serde_json::to_string_pretty(&doc).unwrap())
        .map_err(|e| format!("write {file}: {e}"))?;
    Ok(())
}

fn apply_stub(path_s: &str) -> Result<Applied, String> {
    let path = Path::new(path_s);
    if !path.exists() {
        return Ok(Applied {
            undo: Undo::Nothing,
            note: format!("{path_s} not present; nothing to stub"),
        });
    }
    let moved_to = format!("{path_s}{STUB_SUFFIX}");
    if Path::new(&moved_to).exists() {
        return Ok(Applied {
            undo: Undo::Nothing,
            note: format!("{path_s} appears already stubbed"),
        });
    }
    fs::rename(path, &moved_to).map_err(|e| format!("move {path_s}: {e}"))?;
    fs::write(path, STUB_MARKER).map_err(|e| format!("write marker {path_s}: {e}"))?;
    Ok(Applied {
        undo: Undo::MovedExecutable { original: path_s.to_string(), moved_to },
        note: String::new(),
    })
}

fn revert_stub(original: &str, moved_to: &str) -> Result<(), String> {
    let orig = Path::new(original);
    if orig.exists() {
        let content = fs::read(orig).map_err(|e| format!("read {original}: {e}"))?;
        if content != STUB_MARKER.as_bytes() {
            return Err(format!(
                "{original} is not the Stasis marker (the app may have replaced it); \
                 original preserved at {moved_to} — restore manually"
            ));
        }
        fs::remove_file(orig).map_err(|e| format!("remove marker {original}: {e}"))?;
    }
    fs::rename(moved_to, orig).map_err(|e| format!("restore {original}: {e}"))?;
    Ok(())
}

fn apply_block(path_s: &str) -> Result<Applied, String> {
    let path = Path::new(path_s);
    if path.is_dir() {
        return Err(format!(
            "{path_s} exists as a directory; refusing to block (freeze the app's updater \
             or remove the directory first)"
        ));
    }
    if path.exists() {
        return Ok(Applied {
            undo: Undo::Nothing,
            note: format!("{path_s} already occupied"),
        });
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("create {}: {e}", parent.display()))?;
    }
    fs::write(path, b"blocked by Stasis; run `stasis thaw` to remove\n")
        .map_err(|e| format!("write {path_s}: {e}"))?;
    let mut perms = fs::metadata(path).map_err(|e| e.to_string())?.permissions();
    perms.set_readonly(true);
    fs::set_permissions(path, perms).map_err(|e| format!("set read-only {path_s}: {e}"))?;
    Ok(Applied {
        undo: Undo::CreatedBlocker { path: path_s.to_string() },
        note: String::new(),
    })
}

fn revert_block(path_s: &str) -> Result<(), String> {
    let path = Path::new(path_s);
    if !path.exists() {
        return Ok(());
    }
    let mut perms = fs::metadata(path).map_err(|e| e.to_string())?.permissions();
    #[allow(clippy::permissions_set_readonly_false)]
    perms.set_readonly(false);
    fs::set_permissions(path, perms).map_err(|e| e.to_string())?;
    fs::remove_file(path).map_err(|e| format!("remove {path_s}: {e}"))?;
    Ok(())
}
