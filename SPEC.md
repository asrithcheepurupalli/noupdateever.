# STASIS — The Update Firewall
## The Sealed Specification, v1 — final

*Working product name: **Stasis** (alternates considered: Gate, Keep, Freeze,
Halt). This document is written in our style: it is the complete description
of everything v1 will ever be. When the software matches this document, the
product is finished. Features not in this document do not go in v1 — they go
in a future, separate product (v2), which buyers of v1 may freely ignore.*

---

## 1. One sentence

Stasis is a single sealed binary for Windows that stops every application on
your machine from updating itself, and makes change something that happens
only when you open the gate — deliberately, visibly, reversibly.

## 2. The covenant (printed in the box, legally part of the license)

1. Stasis never updates itself automatically. Ever.
2. Stasis never connects to the network except the two functions listed in
   §6, both user-initiated, both optional, both fully functional offline.
3. No telemetry, no analytics, no account, no login, no activation server.
   The license key is verified offline (signed key file).
4. No feature of v1 will ever be removed, moved, renamed, or paywalled.
5. Your data (config, logs, snapshots) lives in open, documented formats
   (§8) in one folder you can copy, back up, and read with a text editor.
6. Security fixes, if ever needed, ship as signed, opt-in patches that
   change nothing but the flaw, each with a one-page diff description.
7. If the company dies, a pre-committed dead-man process open-sources the
   recipe-signing key escrow so the catalog keeps working. Your binary
   already works forever without us.

## 3. Who it is for

- **Developers** who want their machine frozen through a release week,
  a demo, a conference talk, or a long investigation.
- **Professionals** (audio, CAD, trading, medical, lab) whose toolchain
  breaking costs real money and who are one forced update away from rage.
- **Operators of single-purpose PCs** — kiosk, POS, machine controller,
  studio rig — who today buy Deep Freeze or just unplug the network.

Explicitly NOT for (v1): IT fleets needing central management. That is the
Fleet Console — a *separate future product*, not a v1 feature.

## 4. The complete feature list

These five features are the entire product. Nothing else will be added.

### F1 — Inventory ("what can change on this machine?")
Scans the machine and produces the **Change Surface Report**: every
installed application, its version, and every mechanism it has for changing
itself — updater services (gupdate, edgeupdate…), scheduled tasks
(GoogleUpdateTaskMachine…), Squirrel/Electron `Update.exe` layouts, Store
apps, winget/msix sources, updater registry keys, and apps whose updaters
Stasis does not recognize (flagged honestly as "unknown updater").

### F2 — Freeze ("nothing changes until I say so")
One master switch, per-app override switches. Freezing an app applies its
**recipe** (§5): disabling updater services and scheduled tasks, setting
vendor-documented update policies (e.g. Chrome `UpdateDefault=0`), renaming
or stubbing updater executables where policy doesn't exist, and adding
Windows Firewall rules for known updater endpoints. Everything Freeze does
is recorded and exactly reversible (F5).

**OS policy, stated honestly:** Stasis defers Windows *feature* and
*quality* updates using Microsoft's own supported policies, but by default
does NOT block OS security updates. A machine silently missing security
patches is the one broken promise we refuse to ship. Users may override
this (air-gapped/kiosk machines); the override is explicit, per-machine,
and prints the warning once — then never nags again (covenant §4: we do
not nag).

### F3 — Snapshot ("what did my machine look like when it worked?")
On demand and automatically before every gate session: records the full
version manifest (every app + version + file hashes of executables),
creates a Windows restore point, and archives installer caches where
present. Snapshots are plain files (§8), diffable: `stasis diff` shows
exactly what changed between any two points in time.

### F4 — The Gate ("change as a ceremony, not an ambush")
The only sanctioned way anything updates. Opening the gate: takes a
snapshot (F3), shows the pending-updates list with versions and sizes,
lets the user pick, applies only what was picked, re-runs inventory,
shows the diff, closes. If something broke, F5 rolls back to the
pre-gate snapshot. The gate can be scheduled ("first Monday, monthly")
but never opens itself.

### F5 — Rollback ("undo the mistake")
Reverts a gate session: restores the pre-gate restore point, reinstalls
prior versions from archived installers where available, and reports —
honestly, per-app — what could and could not be reverted (some vendors
make downgrade impossible; we say so instead of pretending).

Interfaces: a CLI (`stasis scan|freeze|thaw|snapshot|diff|gate|rollback`)
and a minimal local GUI presenting the same five functions. The GUI is a
thin skin over the CLI; parity is a spec requirement.

## 5. Recipes — where the churn goes to die

The binary is sealed forever. The world is not. Everything about the
outside world lives in **recipes**: signed, versioned, human-readable TOML
files describing one app's update machinery and how to silence it. The
binary ships with a starter catalog burned in (top ~50 apps: browsers,
Electron apps, IDEs, launchers). New and changed recipes come from the
subscription catalog (§6) or are written by hand — the format is publicly
documented and users can author, edit, and share their own. Recipes are
data: declarative allowlisted actions only (set key, disable task, stub
exe, firewall rule), no scripting, so a malicious recipe can't do anything
the engine itself can't.

This is the architectural answer to "how can an update blocker survive a
changing world without updating?" — the engine's *behavior* is eternal;
its *knowledge* is replaceable paper.

## 6. The only two network functions (both optional, user-initiated)

1. **Catalog fetch**: download new signed recipes. Also fully usable
   offline — recipes are files; carry them in on a USB stick.
2. **Patch check**: fetch the (almost always empty) list of signed
   security-only patches for the binary. Never automatic.

## 7. Business model

- **Stasis Personal — $79 once.** One human, all their machines. Includes
  the burned-in catalog forever.
- **Stasis Workstation — $199/seat once.** Commercial use.
- **Catalog subscription — $39/year, optional.** New/updated recipes as
  the world churns. Lapsing loses nothing you already have; the binary and
  every recipe you ever downloaded work forever. This is the
  insurance-shaped recurring revenue: it pays for the *knowledge*, never
  for the software's behavior.
- **v2, if it ever exists, is a separate product with a separate price.**
  Nothing about your v1 changes when v2 ships. (Fleet Console likewise.)

## 8. Technical commitments

- One static binary, no installer-managed dependencies. Rust. No runtime,
  no framework churn.
- State: one folder, `%ProgramData%\Stasis\` — TOML config, JSONL logs,
  snapshot manifests as JSON, recipes as TOML. All formats documented in
  `FORMATS.md`, which ships in the box.
- GUI: native Win32/WinUI-minimal, not Electron. (An update firewall built
  on Electron would be a joke at our own expense.)
- Signed everything: binary, patches, recipes (Ed25519; public keys
  printed in the manual, key escrow per covenant §7).
- Windows 10 and 11 at spec-freeze. New Windows versions are handled by
  recipes where possible; if an OS change genuinely breaks the engine,
  that is v2 territory — v1 keeps working on the OS versions it promised.

## 9. What v1 will never do (anti-features, as binding as features)

No cloud. No accounts. No AI. No fleet management. No macOS/Linux (that's
a future separate product, same covenant). No blocking of OS security
updates by default. No nagging, toasts, or upsells inside the product. No
"news". No branding refreshes. The v1 you buy on day one is pixel-for-pixel
the v1 running in 2036.

## 10. Definition of done → build plan

The product is finished when the five features pass the acceptance list
below on Win 10 + 11; then it is sealed, and work shifts to recipes only.

- **M1 — Engine core**: inventory scanner + recipe interpreter + the
  starter catalog for 10 apps. Acceptance: Chrome, VS Code, Slack, Spotify,
  Discord provably stop self-updating; `stasis scan` report is complete
  and honest about unknowns.
- **M2 — Freeze/Thaw + reversibility**: every action journaled and
  reversible; thaw restores machine to pre-Stasis state bit-for-bit.
- **M3 — Snapshot/Diff/Gate/Rollback**: full ceremony works end-to-end;
  kill-the-power-mid-gate recovery test passes.
- **M4 — GUI + license file + docs**: FORMATS.md, the manual, the printed
  covenant. Catalog grows to ~50 recipes.
- **M5 — Seal**: external security review of the engine and recipe
  interpreter; sign; ship. Post-seal, only recipes and (if ever) security
  patches leave the building.

## 11. Decisions still open (the only ones)

1. Final name + trademark search (Stasis vs. alternates).
2. Catalog subscription price point ($29 vs $39 — pick after 20 customer
   conversations).
3. Whether Personal includes commercial use for sole proprietors.
4. The dead-man escrow mechanism (legal review needed).

*Everything not listed in §11 is decided. That is the point.*
