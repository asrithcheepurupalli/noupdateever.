# Round 2 — Non-Obvious Ideas (evidence-backed)

*2026-07-15. Second brainstorm pass: deeper, researched, deliberately not simple.*

## 0. Two findings that change everything from Round 1

**Finding 1 — The ONCE model failed commercially.** 37signals' pay-once
Campfire ($299, self-hosted) recouped its investment and then stalled; they
gave up and open-sourced Campfire, Writebook, and Fizzy. So "take a SaaS
category, sell it pay-once" — my Round-1 recommendation B — is *weaker than
it looked*. The best-resourced, best-distributed team in indie software
tried exactly that and pivoted away. Lesson: a philosophical promise alone
doesn't clear the purchase bar.

**Finding 2 — People already pay serious money for stasis itself.**
- Microsoft charges $61 → $122 → $244 per device per year (doubling
  annually) for Windows 10 Extended Security Updates — i.e., for the right
  to NOT update.
- Faronics Deep Freeze has sold "reboot-to-restore = nothing ever changes"
  to schools, kiosks, and POS fleets for two decades.
- Factories buy *used Windows XP machines* on secondary markets and
  virtualize ancient PCs to keep CNC software running, because the upgrade
  path is "replace the whole machine."
- In FDA-regulated pharma, every vendor software update historically
  triggered a 4–8 week revalidation cycle — updates are a literal
  compliance cost line item.

**The synthesis, and the fix for ONCE's revenue problem:** don't sell
software that doesn't update — **sell stasis itself as the product.** The
software stays sealed forever (the ethos holds), but the *guarantee* is
what's monetized: warranties, coverage, protocol packs, validation
packages. Insurance-shaped revenue on top of finished software. Microsoft's
ESU is proof this prints money; we'd be the company that does it as a
promise-keeper instead of a hostage-taker.

**The derivation method** (how these ideas were found, reusable):
1. Follow money *already being paid* for stasis (ESU, Deep Freeze, used XP
   boxes, extended-support contracts).
2. Look where change is a quantified cost — compliance revalidation,
   production downtime, bricked hardware — not a mere annoyance.
3. Invert the product: sell *control over change*, not absence of change.
4. Let the recurring revenue attach to the guarantee, never to the software.

---

## 1. The Machine Time Capsule (industrial stasis appliance)

Factories run irreplaceable machines controlled by Windows XP/2000 PCs.
When that PC dies, a $500K CNC machine is scrap — so shops hoard used XP
towers today. Product: a hardened appliance that swallows the legacy PC
whole — disk-imaged into a VM with the machine-specific I/O (serial, PCI,
parallel-port dongles) passed through, network-isolated by construction,
with a written 25-year support commitment and a spare-appliance program.

- **Who pays / why now**: machine downtime costs thousands per hour;
  Windows 10's end-of-support is pushing a fresh wave of stranded
  industrial PCs right now.
- **Why unique**: existing answers are consultants and DIY virtualization;
  nobody sells this as a sealed *product with a longevity warranty*.
- **Revenue**: appliance + annual "stasis warranty" (spares, disaster
  recovery, re-imaging). Recurring revenue without touching the software.
- **Moat**: the library of weird hardware passthrough recipes compounds.
- **Risk**: hardware business; field diversity of legacy I/O is a beast.

## 2. Validated-Once: sealed software for regulated industries

In GxP/pharma/labs, the vendor's update cadence is the customer's
compliance bill. Product line: data-capture / electronic lab notebook /
batch-record software that is **sealed by design and ships with the full
validation package included**. Sales pitch is one sentence: "The only
vendor whose software will never trigger a revalidation."

- **Who pays / why now**: QA time is expensive and scarce; even under the
  FDA's newer risk-based CSA guidance, *zero change* is the cheapest
  possible thing to assure.
- **Why unique**: every incumbent (Veeva, MasterControl…) is cloud SaaS
  with continuous updates — they structurally cannot make this promise.
  Our constraint is their impossibility. That's a real moat.
- **Revenue**: high-ticket licenses + paid validation documentation +
  optional signed security-only patch coverage (each patch ships with its
  own delta-validation pack — we monetize the paperwork, not the code).
- **Risk**: slow enterprise sales; domain expertise needed; CSA guidance
  is gradually shrinking (but not eliminating) the pain.

## 3. The De-Cloud Box: a retirement home for orphaned smart homes

Belkin killed the entire Wemo line in January 2026 — parents lost baby
monitor access overnight. Neato vacuums, Logitech POP, Nest Secure, Devolo:
the brick-parade is now an annual event. Product: a sealed consumer
appliance that *adopts* cloud-orphaned devices — takes over local control
via local APIs / Matter / reverse-engineered protocols — and works offline
forever. The box itself never updates; device support ships as **protocol
packs**: data, not code, installed only by user choice.

- **Who pays / why now**: every shutdown mints thousands of angry owners
  holding paperweights; press writes the story for us every time.
- **Why unique**: Home Assistant is the hobbyist answer and is famous for
  its "avalanche of updates always breaks something." Hubitat is closest
  but still update-driven. Nobody sells *sealed appliance + your devices
  will outlive their manufacturers* to normal people.
- **Revenue**: hardware margin + paid protocol packs (new dead-brand
  rescues) + optional local-only monitoring add-ons. Recurring without
  touching the box's behavior.
- **Risk**: hardware + consumer support load + reverse-engineering gray
  zones. The boldest, most narrative-rich swing on this list.

## 4. Acqui-Freeze: the permanent home for finished software (holdco)

Constellation Software became a ~30%/yr compounding machine by buying 1000+
niche software companies and *never selling*. Nobody has run that playbook
with **stasis as the covenant**: acquire small, beloved, essentially-done
tools from aging or burned-out indie developers, and publicly, contractually
promise: never redesigned, never subscription-ified, never killed, formats
documented forever.

- **Who pays**: existing users keep buying licenses; upgrades were already
  rare. We buy at indie-tool multiples, hold forever.
- **Why unique**: it's a *business-model* innovation. The brand promise
  compounds on both sides — users trust the mark, and founders who love
  their users *want* to sell to us instead of to a private-equity
  strip-miner. Deal flow becomes the moat.
- **Why now**: a generation of solo devs behind 15-year-old beloved tools
  is hitting retirement with no succession plan.
- **Risk**: needs capital and M&A muscle; start with literally one tiny
  beloved tool as proof of covenant.

## 5. The Update Firewall: a control plane for change itself

Deep Freeze, ESU, and the periodic fleet-wide-bad-update disaster all point
at the same product: software whose job is to **stop other software from
changing**. Pin every app version across a machine or fleet, block/defer
auto-updates (Microsoft just removed the Store opt-out — power users are
furious), snapshot-and-rollback, and a "release window" policy: change
happens only when you open the gate, staged and reversible. Dev mode:
"freeze my machine until launch week."

- **Who pays / why now**: IT fleets (kiosk/POS/labs — Deep Freeze's market,
  modernized), dev teams pre-release, and anyone burned by a pushed update.
- **Why unique**: update *managers* exist everywhere; an update *firewall*
  — allowlist-by-default, change as a gated event — inverts the category.
- **Fit**: monetizes hatred of updates directly, sells to devs (our
  distribution), and the tool itself must be sealed — we eat our dogfood in
  public. Revenue: per-seat/per-endpoint; the coverage catalog (freeze
  recipes per app/OS) is data, so recurring revenue never touches behavior.
- **Risk**: OS vendors fight back (APIs shift under us — ironic); needs
  per-platform depth. Start Windows-fleet-first where Deep Freeze proved
  the buyer exists.

## 6. The Century Archive: a vault that carries its own reader

Preservation research is unanimous: the only digital archive that survives
decades is a *maintained* one — and no family maintains one. Product: an
archive format where **the reader travels inside the archive** — a single
self-contained HTML file (plain JS, zero dependencies, formal paper spec in
the box) that any browser in 2080 can open — plus a sealed writer app that
only admits eternal formats (JPEG/TIFF/PDF/FLAC/MP4). Pay once. Zero
servers, zero network surface: it *cannot* rot from our side.

- **Who pays**: estates, parents, genealogists — the photo-book /
  safe-deposit-box budget. Death-tech adjacency (digital wills).
- **Why unique**: every competitor is a cloud subscription — i.e., a
  promise to bill your heirs. Ours is the only architecture where the
  company dying doesn't matter. That's the whole pitch.
- **Risk**: consumer marketing is hard; purchase is emotional, not urgent.

---

## 7. How they stack up

| Idea | Pain is $$ | Uniqueness | Buildable small | Dev distribution fits | Recurring rev w/o updates |
|---|---|---|---|---|---|
| 1 Machine Time Capsule | ★★★ | ★★★ | ★★ | ★ | ★★★ (warranty) |
| 2 Validated-Once | ★★★ | ★★★ | ★★ | ★ | ★★★ (validation packs) |
| 3 De-Cloud Box | ★★ | ★★★ | ★ | ★★ | ★★ (protocol packs) |
| 4 Acqui-Freeze | ★★ | ★★★ | ★ (capital) | ★★ | ★★★ (portfolio) |
| 5 Update Firewall | ★★★ | ★★ | ★★★ | ★★★ | ★★★ (coverage catalog) |
| 6 Century Archive | ★ | ★★★ | ★★★ | ★ | ★ (pay once) |

**Current read:** Idea 5 (Update Firewall) is the wedge — it sells the
*hatred of updates* directly, to an audience we can reach, at a scope a
small team can finish, with Deep Freeze and ESU as existence proofs of the
buyer. Idea 4 (Acqui-Freeze) is the endgame — the brand "a permanent home
for finished software" is what the company grows up to be, and ideas 1/2/3
are acquisition-or-expansion territory once revenue exists. Idea 3 is the
high-variance consumer swing if we ever want a hardware story.

Round-1's monitoring-tool idea isn't dead, but post-ONCE evidence it needs
the insurance-shaped revenue attached ("stasis warranty") to be a business
rather than a manifesto demo.

## 8. Sources

- 37signals ONCE outcome: https://world.hey.com/dhh/once-again-3e99f755
- Wemo/Neato/POP/Nest brickings: https://www.howtogeek.com/smart-home-brands-that-bricked-products/
- FDA CSV revalidation burden (4–8 wks/update) and CSA shift: https://www.pharmanow.live/leadership/fda-computer-software-assurance-csa-pharma-guide
- Windows 10 ESU pricing ($61/$122/$244 per device): https://learn.microsoft.com/en-us/windows/whats-new/extended-security-updates
- Deep Freeze reboot-to-restore for kiosks/POS: https://www.faronics.com/technology/reboot-to-restore
- Factories on XP / used-PC secondary market: https://www.shopfloorautomations.com/overcoming-the-risks-of-outdated-windows-based-cnc-machines/
- Waves subscription backlash and reversal: https://homemademusic.com/waves-plugins/
- Constellation Software buy-and-hold-forever model: https://quartr.com/insights/company-research/constellation-software-acquiring-and-empowering
- Home Assistant breaking-changes complaints: https://forums.homeseer.com/forum/general-home-automation/off-topic/1733522-has-anyone-else-found-home-assistant-to-be-confusing
- Forced-update backlash (MS Store opt-out removed): https://it.slashdot.org/story/25/08/19/1719236/windows-power-users-frustrated-as-microsoft-forces-automatic-app-updates
- Personal digital preservation / format longevity: https://www.digitalpreservation.gov/personalarchiving/
