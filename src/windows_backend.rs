//! The Windows implementation: the real probe and the real system backend.
//! Compiled only on Windows; everything here goes through either winreg or
//! the OS's own administration tools (sc, schtasks, netsh) — boring,
//! documented, decades-stable interfaces. That is a feature.
#![cfg(windows)]

use crate::action::{Action, Hive, RegData, RegType, Undo};
use crate::engine::{Applied, SystemBackend};
use crate::inventory::{MachineFacts, SystemProbe};
use std::process::Command;
use winreg::enums::*;
use winreg::RegKey;

fn run(cmd: &str, args: &[&str]) -> Result<String, String> {
    let out = Command::new(cmd)
        .args(args)
        .output()
        .map_err(|e| format!("{cmd}: {e}"))?;
    if !out.status.success() {
        return Err(format!(
            "{cmd} {}: {}",
            args.join(" "),
            String::from_utf8_lossy(&out.stderr).trim()
        ));
    }
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}

fn hive_key(hive: Hive) -> RegKey {
    match hive {
        Hive::HKLM => RegKey::predef(HKEY_LOCAL_MACHINE),
        Hive::HKCU => RegKey::predef(HKEY_CURRENT_USER),
    }
}

pub struct WindowsBackend;

impl SystemBackend for WindowsBackend {
    fn disable_service(&mut self, service: &str) -> Result<Applied, String> {
        let services = RegKey::predef(HKEY_LOCAL_MACHINE)
            .open_subkey(format!("SYSTEM\\CurrentControlSet\\Services\\{service}"))
            .map_err(|_| format!("service `{service}` not found"))?;
        let prior_start: u32 = services
            .get_value("Start")
            .map_err(|e| format!("service `{service}`: cannot read start type: {e}"))?;
        if prior_start == 4 {
            return Ok(Applied {
                undo: Undo::Nothing,
                note: format!("service `{service}` already disabled"),
            });
        }
        run("sc.exe", &["config", service, "start=", "disabled"])?;
        run("sc.exe", &["stop", service]).ok(); // may not be running; fine
        Ok(Applied {
            undo: Undo::ServiceStart { service: service.into(), prior_start },
            note: String::new(),
        })
    }

    fn disable_scheduled_task(
        &mut self,
        task: &str,
        prefix: bool,
    ) -> Result<Vec<Applied>, String> {
        let matches: Vec<String> = if prefix {
            list_tasks()?
                .into_iter()
                .filter(|t| t.trim_start_matches('\\').starts_with(task.trim_start_matches('\\')))
                .collect()
        } else {
            vec![task.to_string()]
        };
        if matches.is_empty() {
            return Ok(vec![Applied {
                undo: Undo::Nothing,
                note: format!("no scheduled tasks match `{task}`"),
            }]);
        }
        let mut out = Vec::new();
        for t in matches {
            run("schtasks.exe", &["/change", "/tn", &t, "/disable"])?;
            out.push(Applied {
                undo: Undo::TaskWasEnabled { task: t },
                note: String::new(),
            });
        }
        Ok(out)
    }

    fn set_registry_value(&mut self, action: &Action) -> Result<Applied, String> {
        let Action::SetRegistryValue { hive, key, value, value_type, data } = action else {
            return Err("internal: wrong action for set_registry_value".into());
        };
        let root = hive_key(*hive);
        let key_existed = root.open_subkey(key).is_ok();
        let (subkey, _) = root
            .create_subkey(key)
            .map_err(|e| format!("registry {key}: {e}"))?;
        let prior = match value_type {
            RegType::Dword => subkey.get_value::<u32, _>(value).ok().map(RegData::Dword),
            RegType::String => subkey.get_value::<String, _>(value).ok().map(RegData::String),
        };
        match data {
            RegData::Dword(n) => subkey
                .set_value(value, n)
                .map_err(|e| format!("registry {key}\\{value}: {e}"))?,
            RegData::String(s) => subkey
                .set_value(value, s)
                .map_err(|e| format!("registry {key}\\{value}: {e}"))?,
        }
        Ok(Applied {
            undo: Undo::RegistryValue {
                hive: *hive,
                key: key.clone(),
                value: value.clone(),
                prior,
                key_existed,
            },
            note: String::new(),
        })
    }

    fn firewall_block_program(&mut self, program: &str) -> Result<Applied, String> {
        let rule = format!("Stasis: block updater ({program})");
        run(
            "netsh.exe",
            &[
                "advfirewall", "firewall", "add", "rule",
                &format!("name={rule}"),
                "dir=out", "action=block",
                &format!("program={program}"),
                "enable=yes",
            ],
        )?;
        Ok(Applied { undo: Undo::FirewallRule { rule }, note: String::new() })
    }

    fn revert_system(&mut self, undo: &Undo) -> Result<(), String> {
        match undo {
            Undo::ServiceStart { service, prior_start } => {
                let start = match prior_start {
                    2 => "auto",
                    3 => "demand",
                    4 => "disabled",
                    0 | 1 => return Err(format!("service `{service}`: refusing to touch a boot/system driver")),
                    _ => "demand",
                };
                run("sc.exe", &["config", service, "start=", start])?;
                Ok(())
            }
            Undo::TaskWasEnabled { task } => {
                run("schtasks.exe", &["/change", "/tn", task, "/enable"])?;
                Ok(())
            }
            Undo::RegistryValue { hive, key, value, prior, key_existed } => {
                let root = hive_key(*hive);
                match prior {
                    Some(RegData::Dword(n)) => {
                        let sub = root.open_subkey_with_flags(key, KEY_SET_VALUE)
                            .map_err(|e| format!("registry {key}: {e}"))?;
                        sub.set_value(value, n).map_err(|e| e.to_string())?;
                    }
                    Some(RegData::String(s)) => {
                        let sub = root.open_subkey_with_flags(key, KEY_SET_VALUE)
                            .map_err(|e| format!("registry {key}: {e}"))?;
                        sub.set_value(value, s).map_err(|e| e.to_string())?;
                    }
                    None => {
                        if *key_existed {
                            if let Ok(sub) = root.open_subkey_with_flags(key, KEY_SET_VALUE) {
                                sub.delete_value(value).ok();
                            }
                        } else {
                            // We created the whole key for this policy; remove it again.
                            root.delete_subkey_all(key).ok();
                        }
                    }
                }
                Ok(())
            }
            Undo::FirewallRule { rule } => {
                run(
                    "netsh.exe",
                    &["advfirewall", "firewall", "delete", "rule", &format!("name={rule}")],
                )?;
                Ok(())
            }
            _ => Err("internal: not a system undo".into()),
        }
    }
}

fn list_tasks() -> Result<Vec<String>, String> {
    let csv = run("schtasks.exe", &["/query", "/fo", "csv", "/nh"])?;
    Ok(csv
        .lines()
        .filter_map(|l| l.split('"').nth(1))
        .map(|s| s.to_string())
        .collect())
}

pub struct WindowsProbe;

impl SystemProbe for WindowsProbe {
    fn facts(&self) -> Result<MachineFacts, String> {
        let mut apps = Vec::new();
        for (root, path) in [
            (HKEY_LOCAL_MACHINE, "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Uninstall"),
            (HKEY_LOCAL_MACHINE, "SOFTWARE\\WOW6432Node\\Microsoft\\Windows\\CurrentVersion\\Uninstall"),
            (HKEY_CURRENT_USER, "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Uninstall"),
        ] {
            if let Ok(key) = RegKey::predef(root).open_subkey(path) {
                for name in key.enum_keys().flatten() {
                    if let Ok(sub) = key.open_subkey(&name) {
                        if let Ok(display) = sub.get_value::<String, _>("DisplayName") {
                            let version =
                                sub.get_value::<String, _>("DisplayVersion").unwrap_or_default();
                            apps.push((display, version));
                        }
                    }
                }
            }
        }

        let mut services = Vec::new();
        if let Ok(key) =
            RegKey::predef(HKEY_LOCAL_MACHINE).open_subkey("SYSTEM\\CurrentControlSet\\Services")
        {
            for name in key.enum_keys().flatten() {
                if let Ok(sub) = key.open_subkey(&name) {
                    let display = sub.get_value::<String, _>("DisplayName").unwrap_or_default();
                    services.push((name, display));
                }
            }
        }

        let tasks = list_tasks()?;

        let mut squirrel_dirs = Vec::new();
        if let Some(local) = crate::util::env_lookup("LOCALAPPDATA") {
            if let Ok(entries) = std::fs::read_dir(&local) {
                for e in entries.flatten() {
                    let p = e.path();
                    if p.is_dir() && p.join("Update.exe").exists() {
                        squirrel_dirs.push(p.display().to_string());
                    }
                }
            }
        }

        Ok(MachineFacts { apps, services, tasks, squirrel_dirs })
    }
}
