//! Recipe loading and validation.
//!
//! A recipe is one app's update machinery and how to silence it, as signed
//! TOML. The binary ships with the starter catalog burned in (SPEC §5); more
//! recipes load from a directory. Format v1 is documented in FORMATS.md and
//! is sealed — parsing is strict and unknown `kind`s are rejected.

use crate::action::Action;
use serde::Deserialize;
use std::fs;
use std::path::Path;

pub const FORMAT_VERSION: u32 = 1;

#[derive(Debug, Deserialize)]
pub struct RecipeFile {
    pub recipe: Meta,
    #[serde(default)]
    pub detect: Detect,
    #[serde(default, rename = "action")]
    pub actions: Vec<Action>,
}

#[derive(Debug, Deserialize)]
pub struct Meta {
    pub id: String,
    pub name: String,
    /// Format fields not yet consumed by the engine (vendor, notes, detect
    /// paths) are still parsed strictly — the format is sealed even where the
    /// M1 code doesn't read it yet.
    #[serde(default)]
    #[allow(dead_code)]
    pub vendor: String,
    pub format: u32,
    pub revision: u32,
    /// Ed25519 over the canonical body. Signing lands in M4 (SPEC §10);
    /// until then recipes carry the literal string "unsigned" and the CLI
    /// says so out loud. The field is required now so the format never moves.
    pub signature: String,
    #[serde(default)]
    #[allow(dead_code)]
    pub notes: String,
}

#[derive(Debug, Default, Deserialize)]
pub struct Detect {
    /// Substring matches against installed-app display names.
    #[serde(default)]
    pub uninstall_names: Vec<String>,
    /// Existence checks, %VAR%-expandable.
    #[serde(default)]
    #[allow(dead_code)]
    pub paths: Vec<String>,
}

pub fn parse(source: &str, origin: &str) -> Result<RecipeFile, String> {
    let r: RecipeFile =
        toml::from_str(source).map_err(|e| format!("{origin}: {e}"))?;
    validate(&r).map_err(|e| format!("{origin}: {e}"))?;
    Ok(r)
}

fn validate(r: &RecipeFile) -> Result<(), String> {
    if r.recipe.format != FORMAT_VERSION {
        return Err(format!(
            "recipe format {} is not supported (this build understands format {})",
            r.recipe.format, FORMAT_VERSION
        ));
    }
    let id = &r.recipe.id;
    if id.is_empty()
        || !id.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
    {
        return Err(format!("recipe id `{id}` must be lowercase-kebab ascii"));
    }
    if r.actions.is_empty() {
        return Err(format!("recipe `{id}` has no actions"));
    }
    for a in &r.actions {
        a.validate().map_err(|e| format!("recipe `{id}`: {e}"))?;
    }
    Ok(())
}

/// The starter catalog, burned into the binary (SPEC §5).
pub const EMBEDDED: &[(&str, &str)] = &[
    ("chrome", include_str!("../recipes/chrome.toml")),
    ("edge", include_str!("../recipes/edge.toml")),
    ("firefox", include_str!("../recipes/firefox.toml")),
    ("vscode", include_str!("../recipes/vscode.toml")),
    ("slack", include_str!("../recipes/slack.toml")),
    ("discord", include_str!("../recipes/discord.toml")),
    ("spotify", include_str!("../recipes/spotify.toml")),
    ("zoom", include_str!("../recipes/zoom.toml")),
    ("notepad-plus-plus", include_str!("../recipes/notepad-plus-plus.toml")),
    ("github-desktop", include_str!("../recipes/github-desktop.toml")),
];

pub fn load_embedded() -> Result<Vec<RecipeFile>, String> {
    let mut out = Vec::new();
    for (name, src) in EMBEDDED {
        let r = parse(src, &format!("embedded:{name}"))?;
        if &r.recipe.id != name {
            return Err(format!(
                "embedded recipe `{name}` declares mismatched id `{}`",
                r.recipe.id
            ));
        }
        out.push(r);
    }
    Ok(out)
}

/// Load *.toml recipes from a directory (the user-extendable catalog).
/// A directory recipe with the same id overrides the embedded one — that is
/// how catalog updates supersede burned-in knowledge without touching the binary.
pub fn load_catalog(extra_dir: Option<&Path>) -> Result<Vec<RecipeFile>, String> {
    let mut recipes = load_embedded()?;
    if let Some(dir) = extra_dir {
        let entries = fs::read_dir(dir)
            .map_err(|e| format!("cannot read recipe dir {}: {e}", dir.display()))?;
        let mut names: Vec<_> = entries
            .filter_map(|e| e.ok().map(|e| e.path()))
            .filter(|p| p.extension().is_some_and(|x| x == "toml"))
            .collect();
        names.sort();
        for path in names {
            let src = fs::read_to_string(&path)
                .map_err(|e| format!("cannot read {}: {e}", path.display()))?;
            let r = parse(&src, &path.display().to_string())?;
            recipes.retain(|existing| existing.recipe.id != r.recipe.id);
            recipes.push(r);
        }
    }
    recipes.sort_by(|a, b| a.recipe.id.cmp(&b.recipe.id));
    Ok(recipes)
}
