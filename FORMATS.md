# FORMATS.md — Stasis on-disk formats, v1

*This document ships in the box (SPEC §8). Everything Stasis writes is in an
open format described here, readable with a text editor. Format v1 is sealed:
fields are never removed or repurposed; a future format bump is a new,
separately documented version, and every shipped binary keeps reading v1
forever.*

## 1. The data folder

All Stasis state lives in one folder you can copy, back up, and inspect:

- Windows: `%ProgramData%\Stasis\`
- Development builds on other platforms: `./stasis-data/`

Contents: `journal.jsonl` (§3), `recipes/` (user-added recipes, §2),
`snapshots/` (reserved for M3; format will be added to this document before
that milestone ships).

## 2. Recipe format (TOML)

A recipe describes one application's update machinery and how to silence it.
Recipes are **data, not code**: they compile down to the seven action kinds
in §2.2 and nothing else. Anyone may write one; the catalog is not a priesthood.

```toml
[recipe]
id = "chrome"            # lowercase-kebab ascii, unique
name = "Google Chrome"   # display name
vendor = "Google"        # optional
format = 1               # this document
revision = 1             # bump on every published change to this recipe
signature = "unsigned"   # Ed25519 (base64) over the canonical body; see §2.3
notes = ""               # optional, human-facing

[detect]                          # optional; how scan maps machines to recipes
uninstall_names = ["Google Chrome"]  # substring match on installed-app names
paths = ["%ProgramFiles%\\Google\\Chrome\\Application\\chrome.exe"]

[[action]]                        # one or more
kind = "disable_service"
service = "gupdate"
```

Paths and programs may use Windows-style `%VAR%` environment references.
Unknown variables fail the plan before anything is executed — no action ever
runs against a half-resolved path.

### 2.2 The seven action kinds (sealed allowlist)

| kind | fields | does |
|---|---|---|
| `disable_service` | `service` | Sets a Windows service start type to Disabled and stops it |
| `disable_scheduled_task` | `task`, `match` (`exact`\|`prefix`) | Disables matching scheduled task(s) |
| `set_registry_value` | `hive` (`HKLM`\|`HKCU`), `key`, `value`, `type` (`dword`\|`string`), `data` | Sets a registry value (vendor update policies) |
| `set_json_value` | `file`, `pointer` (single-level, e.g. `/KEY`), `value` | Sets one top-level key in a JSON config file |
| `stub_executable` | `path` | Moves the executable to `<path>.stasis-original`, leaves an inert marker |
| `block_path` | `path` | Creates a read-only file at the path so an updater cannot use it |
| `firewall_block_program` | `program` | Adds a Windows Firewall outbound block rule for the program |

A recipe containing any other `kind` is rejected at parse time. Adding a kind
is a format revision and a product decision, never a convenience.

### 2.3 Signatures

`signature` is Ed25519 over the recipe body with the signature field set to
the empty string, base64-encoded. Signing infrastructure ships in M4; until
then recipes carry the literal `"unsigned"` and the CLI announces it. The
catalog public keys are printed in the manual; the escrow arrangement for
them is covenant §7 material.

## 3. Journal format (JSONL)

`journal.jsonl` is append-only; one JSON object per line. Nothing Stasis does
to a machine is off the books, and every applied entry carries the undo data
to reverse it exactly. `stasis thaw` replays outstanding undos newest-first.

```json
{"ts_unix":1752600000,"session":"freeze-1752600000","op":"apply",
 "recipe_id":"chrome","action":{"kind":"disable_service","service":"gupdate"},
 "undo":{"kind":"service_start","service":"gupdate","prior_start":2},
 "applied":true,"note":""}
```

- `op`: `"apply"` or `"revert"`.
- `applied: false` records an action that failed and changed nothing.
- `undo.kind` values: `service_start`, `task_was_enabled`, `registry_value`,
  `json_value`, `moved_executable`, `created_blocker`, `firewall_rule`,
  `nothing`.

## 4. Scan fixture format (JSON)

`stasis scan --fixture <file>` reads machine facts from JSON instead of
probing live — used by tests, development on non-Windows platforms, and
support ("send me your fixture"). Shape:

```json
{
  "apps": [["Google Chrome", "126.0.6478.127"]],
  "services": [["gupdate", "Google Update Service (gupdate)"]],
  "tasks": ["\\GoogleUpdateTaskMachineCore{GUID}"],
  "squirrel_dirs": ["C:\\Users\\me\\AppData\\Local\\slack"]
}
```
