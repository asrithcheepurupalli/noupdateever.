//! stasis — the update firewall.
//!
//! Five functions, forever: scan, freeze/thaw, snapshot/diff, gate, rollback
//! (SPEC §4). M1 ships scan, freeze, thaw, recipes. The CLI is hand-rolled:
//! an argument parser is not worth a dependency we keep for eternity.

mod action;
mod engine;
mod inventory;
mod journal;
mod recipe;
mod util;
#[cfg(windows)]
mod windows_backend;

use engine::{LocalExecutor, PlannedAction};
use inventory::SystemProbe;
use std::path::PathBuf;
use std::process::ExitCode;

const USAGE: &str = "\
stasis — the update firewall. Nothing changes until you say so.

USAGE:
  stasis scan     [--fixture <file>] [--json]
  stasis freeze   <recipe-id>... | --all  [--dry-run]
  stasis thaw     [--dry-run]
  stasis recipes  [--dir <path>]
  stasis version

COMMANDS:
  scan      Report every mechanism this machine has for changing itself,
            and whether the catalog knows how to silence it.
  freeze    Apply recipes: stop the named apps from updating themselves.
            Everything done is journaled and exactly reversible.
  thaw      Reverse every outstanding change, newest first.
  recipes   List and validate the recipe catalog (embedded + --dir).
  version   Print version. This binary never updates itself.
";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let result = match args.first().map(String::as_str) {
        Some("scan") => cmd_scan(&args[1..]),
        Some("freeze") => cmd_freeze(&args[1..]),
        Some("thaw") => cmd_thaw(&args[1..]),
        Some("recipes") => cmd_recipes(&args[1..]),
        Some("version") => {
            println!("stasis {} (sealed; this binary never updates itself)", env!("CARGO_PKG_VERSION"));
            Ok(())
        }
        Some("help") | None => {
            print!("{USAGE}");
            Ok(())
        }
        Some(other) => Err(format!("unknown command `{other}`\n\n{USAGE}")),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("stasis: {e}");
            ExitCode::FAILURE
        }
    }
}

fn flag(args: &[String], name: &str) -> bool {
    args.iter().any(|a| a == name)
}

fn opt_value(args: &[String], name: &str) -> Option<String> {
    args.iter().position(|a| a == name).and_then(|i| args.get(i + 1).cloned())
}

/// State lives in one folder (SPEC §8): %ProgramData%\Stasis on Windows,
/// ./stasis-data during development elsewhere.
fn data_dir() -> PathBuf {
    if cfg!(windows) {
        util::env_lookup("ProgramData")
            .map(|p| PathBuf::from(p).join("Stasis"))
            .unwrap_or_else(|| PathBuf::from("C:\\ProgramData\\Stasis"))
    } else {
        PathBuf::from("stasis-data")
    }
}

fn journal_path() -> PathBuf {
    data_dir().join("journal.jsonl")
}

fn probe_from_args(args: &[String]) -> Result<Box<dyn SystemProbe>, String> {
    if let Some(fixture) = opt_value(args, "--fixture") {
        return Ok(Box::new(inventory::FixtureProbe::from_file(std::path::Path::new(
            &fixture,
        ))?));
    }
    #[cfg(windows)]
    {
        Ok(Box::new(windows_backend::WindowsProbe))
    }
    #[cfg(not(windows))]
    {
        Err("live scanning needs Windows; on this platform pass --fixture <facts.json> \
             (see FORMATS.md for the fixture shape)"
            .into())
    }
}

fn cmd_scan(args: &[String]) -> Result<(), String> {
    let catalog = recipe::load_catalog(None)?;
    let probe = probe_from_args(args)?;
    let report = inventory::build_report(probe.as_ref(), &catalog)?;

    if flag(args, "--json") {
        println!("{}", serde_json::to_string_pretty(&report).unwrap());
        return Ok(());
    }

    println!("Change Surface Report");
    println!("=====================");
    println!("Installed applications: {}", report.apps_total);
    println!();
    if report.findings.is_empty() {
        println!("No known update mechanisms found.");
    } else {
        println!("Update mechanisms the catalog can silence:");
        for f in &report.findings {
            println!(
                "  [{}]  {} — {}",
                f.recipe.as_deref().unwrap_or("-"),
                f.mechanism,
                f.detail
            );
        }
    }
    if !report.unknown.is_empty() {
        println!();
        println!("Honesty section — mechanisms with NO recipe (these can still change your machine):");
        for f in &report.unknown {
            println!("  [??]  {} — {}", f.mechanism, f.detail);
        }
    }
    println!();
    println!("Freeze with: stasis freeze <recipe-id>...   (or --all)");
    Ok(())
}

fn backend() -> Box<dyn engine::SystemBackend> {
    #[cfg(windows)]
    {
        Box::new(windows_backend::WindowsBackend)
    }
    #[cfg(not(windows))]
    {
        Box::new(engine::UnsupportedBackend)
    }
}

fn cmd_freeze(args: &[String]) -> Result<(), String> {
    let dry_run = flag(args, "--dry-run");
    let catalog = recipe::load_catalog(None)?;
    let ids: Vec<&str> = args
        .iter()
        .filter(|a| !a.starts_with("--"))
        .map(String::as_str)
        .collect();

    let selected: Vec<&recipe::RecipeFile> = if flag(args, "--all") {
        catalog.iter().collect()
    } else if ids.is_empty() {
        return Err("say which apps to freeze: stasis freeze <recipe-id>... (or --all)\n\
                    known ids: stasis recipes"
            .into());
    } else {
        let mut sel = Vec::new();
        for id in &ids {
            match catalog.iter().find(|r| r.recipe.id == *id) {
                Some(r) => sel.push(r),
                None => return Err(format!("no recipe with id `{id}` (see: stasis recipes)")),
            }
        }
        sel
    };

    for r in &selected {
        if r.recipe.signature == "unsigned" {
            eprintln!(
                "note: recipe `{}` is unsigned (signing lands in M4; dev builds accept this)",
                r.recipe.id
            );
        }
    }

    let plan = engine::plan(&selected, &|n| util::env_lookup(n))?;
    println!(
        "Plan: {} action(s) across {} recipe(s){}",
        plan.len(),
        selected.len(),
        if dry_run { " — DRY RUN, nothing will change" } else { "" }
    );
    for pa in &plan {
        println!("  [{}] {}", pa.recipe_id, pa.action.describe());
    }
    if dry_run {
        return Ok(());
    }
    if !cfg!(windows) {
        return Err(
            "this platform has no live executor; use --dry-run to inspect the plan".into(),
        );
    }

    execute_plan(&plan)
}

fn execute_plan(plan: &[PlannedAction]) -> Result<(), String> {
    let mut be = backend();
    let mut exec = LocalExecutor { backend: be.as_mut() };
    let session = format!("freeze-{}", util::now_unix());
    let jpath = journal_path();
    let mut ok = 0usize;
    let mut failed = 0usize;

    for pa in plan {
        match exec.apply(pa) {
            Ok(applied) => {
                for a in applied {
                    if !a.note.is_empty() {
                        println!("  = [{}] {} ({})", pa.recipe_id, pa.action.describe(), a.note);
                    } else {
                        println!("  ✓ [{}] {}", pa.recipe_id, pa.action.describe());
                    }
                    journal::append(
                        &jpath,
                        &journal::Entry {
                            ts_unix: util::now_unix(),
                            session: session.clone(),
                            op: journal::Op::Apply,
                            recipe_id: pa.recipe_id.clone(),
                            action: pa.action.clone(),
                            undo: a.undo,
                            applied: true,
                            note: a.note,
                        },
                    )?;
                    ok += 1;
                }
            }
            Err(e) => {
                println!("  ✗ [{}] {} — {e}", pa.recipe_id, pa.action.describe());
                journal::append(
                    &jpath,
                    &journal::Entry {
                        ts_unix: util::now_unix(),
                        session: session.clone(),
                        op: journal::Op::Apply,
                        recipe_id: pa.recipe_id.clone(),
                        action: pa.action.clone(),
                        undo: action::Undo::Nothing,
                        applied: false,
                        note: e,
                    },
                )?;
                failed += 1;
            }
        }
    }
    println!();
    println!("Frozen: {ok} action(s) applied, {failed} failed. Journal: {}", jpath.display());
    if failed > 0 {
        Err("some actions failed — the report above is the honest state".into())
    } else {
        Ok(())
    }
}

fn cmd_thaw(args: &[String]) -> Result<(), String> {
    let dry_run = flag(args, "--dry-run");
    let jpath = journal_path();
    let entries = journal::read_all(&jpath)?;
    let live = journal::outstanding(&entries);
    if live.is_empty() {
        println!("Nothing to thaw — no outstanding changes in the journal.");
        return Ok(());
    }
    println!(
        "Thaw: {} outstanding change(s){}",
        live.len(),
        if dry_run { " — DRY RUN" } else { "" }
    );
    for e in &live {
        println!("  [{}] undo: {}", e.recipe_id, e.action.describe());
    }
    if dry_run {
        return Ok(());
    }

    let mut be = backend();
    let mut exec = LocalExecutor { backend: be.as_mut() };
    let session = format!("thaw-{}", util::now_unix());
    let mut failed = 0usize;
    for e in &live {
        match exec.revert(&e.undo) {
            Ok(()) => {
                println!("  ✓ reverted: {}", e.action.describe());
                journal::append(
                    &jpath,
                    &journal::Entry {
                        ts_unix: util::now_unix(),
                        session: session.clone(),
                        op: journal::Op::Revert,
                        recipe_id: e.recipe_id.clone(),
                        action: e.action.clone(),
                        undo: e.undo.clone(),
                        applied: true,
                        note: String::new(),
                    },
                )?;
            }
            Err(err) => {
                println!("  ✗ could not revert {} — {err}", e.action.describe());
                failed += 1;
            }
        }
    }
    if failed > 0 {
        Err(format!("{failed} change(s) could not be reverted; journal keeps them outstanding"))
    } else {
        println!("Machine restored to its pre-Stasis state.");
        Ok(())
    }
}

fn cmd_recipes(args: &[String]) -> Result<(), String> {
    let dir = opt_value(args, "--dir").map(PathBuf::from);
    let catalog = recipe::load_catalog(dir.as_deref())?;
    println!("Recipe catalog — format v{}, {} recipe(s):", recipe::FORMAT_VERSION, catalog.len());
    for r in &catalog {
        println!(
            "  {:<20} {:<28} rev {:<3} {:>2} action(s)  [{}]",
            r.recipe.id,
            r.recipe.name,
            r.recipe.revision,
            r.actions.len(),
            if r.recipe.signature == "unsigned" { "unsigned" } else { "signed" },
        );
    }
    Ok(())
}
