//! Small shared helpers. `%VAR%` expansion takes a lookup function so the
//! planner is testable on any OS with a fixed variable map.

/// Expand Windows-style `%VAR%` references. Unknown variables are an error —
/// silently leaving `%LOCALAPPDATA%` literal in a path would make an action
/// "succeed" against a nonsense location, and we don't do quiet wrongness.
pub fn expand_vars(
    input: &str,
    lookup: &dyn Fn(&str) -> Option<String>,
) -> Result<String, String> {
    let mut out = String::with_capacity(input.len());
    let mut rest = input;
    while let Some(start) = rest.find('%') {
        out.push_str(&rest[..start]);
        let after = &rest[start + 1..];
        match after.find('%') {
            Some(end) => {
                let name = &after[..end];
                if name.is_empty() {
                    out.push('%'); // literal "%%"
                } else {
                    match lookup(name) {
                        Some(v) => out.push_str(&v),
                        None => return Err(format!("undefined variable %{name}%")),
                    }
                }
                rest = &after[end + 1..];
            }
            None => return Err(format!("unterminated %VAR% in `{input}`")),
        }
    }
    out.push_str(rest);
    Ok(out)
}

/// Default lookup: the process environment, case-insensitively (Windows
/// convention; also makes dev on other platforms unsurprising).
pub fn env_lookup(name: &str) -> Option<String> {
    std::env::vars()
        .find(|(k, _)| k.eq_ignore_ascii_case(name))
        .map(|(_, v)| v)
}

pub fn now_unix() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::expand_vars;
    use std::collections::HashMap;

    fn map(pairs: &[(&str, &str)]) -> HashMap<String, String> {
        pairs.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect()
    }

    #[test]
    fn expands_known_vars() {
        let m = map(&[("LOCALAPPDATA", "C:\\Users\\x\\AppData\\Local")]);
        let f = |n: &str| m.get(n).cloned();
        assert_eq!(
            expand_vars("%LOCALAPPDATA%\\slack\\Update.exe", &f).unwrap(),
            "C:\\Users\\x\\AppData\\Local\\slack\\Update.exe"
        );
    }

    #[test]
    fn unknown_var_is_error() {
        let f = |_: &str| None;
        assert!(expand_vars("%NOPE%\\x", &f).is_err());
    }

    #[test]
    fn double_percent_is_literal() {
        let f = |_: &str| None;
        assert_eq!(expand_vars("100%%", &f).unwrap(), "100%");
    }
}
