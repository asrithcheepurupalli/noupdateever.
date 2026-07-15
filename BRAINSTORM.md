# No Update Ever — Product Brainstorm

*Working notes, 2026-07-15. Nothing here is final; this is the thinking so far.*

## 1. Sharpening the premise

The raw idea: devs (and normal people) are exhausted by continuous updates,
forced restarts, downtime, SaaS churn, and software that stops working when
the internet does. We build products that reject that. Local-first as an
option, and "no updates ever" as the business identity.

**Important refinement:** the thing people actually hate is not updates —
it's *change they didn't choose*. Nobody is angry that a bug got fixed.
They're angry that:

- the UI reshuffled overnight and muscle memory broke
- a feature they relied on was removed or paywalled
- an update was forced, with a restart, at the worst moment
- the tool stopped working because a server was shut down or an API deprecated
- their files are hostage to a subscription

So the honest, defensible promise is:

> **"Your software never changes out from under you."**
>
> No auto-updates. No forced anything. No feature ever removed. No UI ever
> reshuffled. No login required. No telemetry. Your data in open formats on
> your disk. Works offline forever. When you buy vN, vN is *finished* — it
> will behave identically in 10 years.

"No updates ever," taken literally, has one genuine problem: **security**.
Software with a network surface that never gets patched is irresponsible and
will be used against us in marketing. Two clean answers, use both:

1. **Design for zero network surface wherever possible.** A local tool that
   never opens a socket has almost nothing to patch. This pushes us toward
   genuinely local-first architecture — which is our vision anyway.
2. **Security-only patches, opt-in, that change nothing else.** Signed,
   documented, one-line changelog, never bundled with features. This keeps
   the promise in spirit: the *behavior* never changes.

So the brand is really the **"finished software"** promise. That phrase has
an existing movement behind it we can ride.

## 2. Prior art (proof this sells)

- **37signals ONCE** (Campfire, Writebook): pay once (~$299), get the code,
  run it yourself, no subscription. Validated that devs/teams will pay
  upfront for un-SaaS'd software.
- **SQLite**: publicly promises support through 2050. Its stability *is* its
  brand. Most-deployed software on earth.
- **Sublime Text / Pinboard / WinRAR-era boxed software**: pay once, works
  forever, paid major versions when *you* decide to upgrade.
- **Obsidian**: local-first files-on-disk was a core reason it beat Notion
  for a large dev audience.
- **Local-first software** (Ink & Switch essay) and the permacomputing /
  malleable-software crowd: an existing ideology and audience that will
  evangelize for free.

The model that makes revenue work without subscriptions: **paid major
versions as separate products.** v1 is finished and eternal. v2, years
later, is a new product you may buy or ignore. Plus an expanding *catalog*
under one trusted brand — the brand promise is the moat, each product is a
SKU.

## 3. Candidate products

Constraint we should respect: our unfair advantage is **distribution to
devs**. First product should be dev-adjacent. Also: the promise itself
forces engineering discipline — boring tech, tiny dependency tree, single
static binary, SQLite/flat files, no Electron-and-npm churn. That discipline
should be visible in the product; it *is* the marketing.

### A. "Time capsule" build tool — *seal your project forever*
A CLI that snapshots a project's entire world — toolchain versions, deps,
system libs — into a hermetic archive that builds and runs identically in
10 years. "Open a 3-year-old client project and it just builds."

- **Pain**: extremely real (every agency, every legacy codebase, compliance/
  archival requirements). Nix solves it but is famously hostile; a friendly
  wrapper is a product.
- **Fit with promise**: perfect — the product's whole job is defeating churn.
- **Risk**: technically hard to do across ecosystems; start with 1–2 (e.g.
  Node + Python) or it's a tarpit.
- **Money**: per-seat or per-seal; agencies and enterprises pay for this.

### B. ONCE-style self-hosted tool for teams — pay once, single binary
Pick one SaaS category with maximum subscription fatigue and ship the
sealed, self-hosted, pay-once version. Best candidates:
- **Uptime/status monitoring** (Datadog/Pingdom pricing rage is constant)
- **Web analytics** (Plausible et al. are still subscriptions)
- **Internal wiki / docs**

One static binary + SQLite, runs on a $5 VPS or a laptop, $299 once.

- **Pain**: real and financial — easy ROI story ("stop paying $99/mo").
- **Fit**: strong; 37signals already educated the market on the model.
- **Risk**: crowded categories; we win on the *promise*, not features.

### C. Finished finance tool for freelancers — invoicing/bookkeeping
Nowhere is "please stop changing my software" stronger than money tools
(every QuickBooks redesign produces refugees). Local file, open format,
your invoices work in 2040.

- **Pain**: high; willingness to pay: high; churn-hatred: maximum.
- **Risk**: not our audience (less dev distribution advantage), and tax
  rules change yearly — which fights the no-updates promise. Ship the
  *engine* eternal, treat tax tables as user-editable data, not code.

### D. Local-first sync framework for other devs
CRDT-based engine so others build no-server apps.
- **Risk**: very crowded (Automerge, Yjs, ElectricSQL, PowerSync, Zero) and
  infrastructure is the *hardest* thing to never update. **Pass for now.**

## 4. Recommendation

**Lead with B (pay-once self-hosted tool), and hold A (time capsule) as the
second act.**

Reasoning: B is buildable by a small team in months, the business model is
pre-validated by ONCE, the audience is exactly who we can reach, and it
lets us *demonstrate* the manifesto quickly — a single 20MB binary with no
auto-updater is a better argument than any landing page. A is the bigger,
more defensible idea but is a research project; do it once the brand exists
and revenue flows.

Within B, my pick is **uptime/status monitoring**: clear recurring cost to
displace, near-zero feature treadmill (ping things, alert, show a page —
finishable!), naturally small network surface, and devs buy it with their
own card.

## 5. Hard questions to nail before writing code

1. **Security policy in writing, day one.** "Sealed behavior, signed
   security-only patches, opt-in" — publish the policy as part of the
   manifesto or critics will define it for us.
2. **Platform rot.** OSes change under us (macOS especially). Mitigation:
   boring substrates — static binaries, POSIX, SQLite, standards-based web
   UI served locally; no Electron, no framework-of-the-week.
3. **Revenue shape.** Pay once + paid major versions + growing catalog.
   Model the numbers: no MRR means launch spikes; catalog cadence matters.
4. **Scope discipline.** "Finished" means we must *spec completely upfront*
   (everything upfront, clear planning). Every feature request post-launch
   is answered with "v2, someday, optional." That's the brand working, not
   a failure.
5. **The manifesto is the first product.** Write and publish it before the
   tool: the promise, the security policy, the format guarantees. It's our
   Local-First/ONCE moment and it costs a weekend.

## 6. Open threads

- Name/brand: "No Update Ever" is already a strong manifesto title.
- License question for ONCE-style: source-visible? (37signals ships code.)
- Which monitoring feature set is "complete"? Draft the *finished* spec.
- Validate: interview 10 devs paying for uptime/analytics SaaS today.
