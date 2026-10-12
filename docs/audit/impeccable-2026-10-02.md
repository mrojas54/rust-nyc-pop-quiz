# Impeccable technical audit — 2026-10-02

## Current-source recheck — 2026-10-11 (PR #70 documentation correction)

Rechecked against main `046779c8c14d46e3722331501e87622db5bb1235` and
PR head `6744a7a9e166893afb68dbfc092ae32c4e310e87`; their `web/` source is
identical. The October 2 audit below is retained as historical evidence.
Its 15/20 score, three P2 findings, test count and browser samples are not a
current certification or a current open-issue count.

| Historical finding | Current disposition | Evidence |
|---|---|---|
| Explanation heading hierarchy | Confirmed in source; application follow-up remains | `web/home/home.js` `beatsHtml` still emits h3 after the question h1 with no explanation h2 parent; later sections emit h2. Skipping alone is not proof of WCAG failure. |
| Large TTF delivery | Recommended optimization already implemented; old recommendation is superseded | `web/shared/fonts.css` prefers all four WOFF2 siblings and keeps TTF fallbacks. `pipeline/src/popquiz/fallback.py` removes TTF fallbacks and embeds WOFF2 only. File sizes match the October 3 report: 380,948 WOFF2 bytes versus 1,245,372 TTF bytes across all faces. See `docs/performance/2026-10-03-fonts.md`; those historical timings/tests were not rerun here. No current latency or Core Web Vitals conclusion follows. |
| Incomplete machine-readable typography | Corrected in this documentation patch | `DESIGN.md` now records 30px, distinct 28px mono/serif roles, 46px choices and other scoped surface sizes; the landing page’s scoped title and control sizes are included. Wall source remains measured, not a static ramp role. |

Additional imported wording corrected: shared trace dimming is .70 (AC-84 /
PQ-40), not .32; host phase button padding and voting disabled states are
exceptions to generic `.btn` examples; small metadata has no contrast exemption;
serif is also intentional for large room identities and voting letters.
The redundant `.impeccable/` config and token snapshot were removed during review.
Shared CSS remains the token source of truth; DESIGN.md describes its roles.

The restructuring retains the original phase table, voice, divergences and
venue hypotheses. New inventory sections are descriptive and subordinate to
those decisions and later accepted build amendments. Historical § references
point to `PHILOSOPHY.md`, not the new heading order. `PRODUCT.md` is rechecked
against October 11 source and linked from CLAUDE.md; it remains a descriptive
inventory, not a completion ledger.

This recheck inspects code and asset sizes; it does not refresh screen-reader,
layout, device, deployed performance or venue evidence. Repairing heading
markup belongs in a separate application change. No application files are
modified by this correction.

---

## Historical October 2 audit

## Implementation integrity verdict

**Pass, with isolated documentation gaps.** The implementation expresses the
projector-first product: phase-specific source colouring, anonymous counts,
private hints, a host-paced trace, and evidence provenance. The shared token
system and surface-specific accessibility companions are intentional.

No application code was changed. Findings below are recommendations, not fixes.

## Historical audit health score — 2026-10-02

**15/20 is the original score, retained without rescoring.** The October 11 source
recheck leaves one of the three P2 findings open; font delivery and typography
documentation are resolved. No new browser audit or current score is claimed.

Scores describe the bounded checks below, not full WCAG conformance or meetup readiness.

| Dimension | Score / 4 | Key finding |
|---|---:|---|
| Accessibility | 3 | Take-home heading structure skips from h1 to h3 |
| Performance | 3 | Brand fonts are relatively large TTF assets |
| Responsive design | 3 | Sampled phone states fit; full phase/zoom/device matrix remains untested |
| Theming | 3 | Strong shared tokens; component-scoped literals remain |
| Implementation integrity | 3 | Documented typography ramp omits intentional implementation sizes |
| **Total** | **15/20** | **Good — address weak dimensions** |

**Verified issues:** P0: 0; P1: 0; P2: 3; P3: 0.
No blocking defect was established in the sampled states.

## Evidence and limits

- Read PRODUCT.md, the binding design decisions, source styles/renderers,
  project run-state and the relevant phase/accessibility contracts.
- Ran `node --test 'web/test/*.test.js'`: **271 passed, 0 failed, 0 skipped**.
  These include contrast, keyboard/focus preservation, announcements, secrecy,
  touch-size CSS and reduced-motion checks. The custom test DOM does no layout.
- Ran the bundled Impeccable detector once over `web/`. Exit 2 indicates findings;
  it also reported an unresolved `/join/buzzer.css` stylesheet path. Its HTML
  route resolution is incomplete; no missing production stylesheet is inferred.
- Used the existing `web/test/a11y/browser.html` harness in the Codex in-app
  browser on a loopback server. Inspected buzzer join/live, host idle, home initial
  trace, and wall idle. The harness supplies room fixtures, not real Discord or
  server behavior. It is not deployed smoke testing.
- Requested a 320×640 viewport override. The browser reported **291 CSS px**
  inner width, so the measurements apply to that reported width, not a certified
  320px device. Sampled buzzer, host and home document scrollWidth equalled
  innerWidth. Buzzer letter targets measured about 267×56.7px; host Start was
  about 259×44px; home trace buttons were about 44×44px. Subpixel values near
  43.999px are rounding of the 44px CSS minimum, not a target-size finding.
- Inspected the saved host-reveal screenshot and compared its font, palette,
  borders and provenance treatment with current source. It is historical visual
  evidence, not a fresh capture of every phase.
- Did not test screen-reader speech, physical touch, text zoom/reflow, reduced
  motion at the OS setting, all phases at multiple widths, mobile browser engines,
  live performance timings, deployed cache headers, or projector venue legibility.
  The wall idle browser view does not establish source fit.

## Detailed findings

### [P2] Take-home explanation skips a heading level

**Location:** `web/home/home.js:110–121` (`beatsHtml`).
**Category:** Accessibility.

The browser heading tree starts with the question h1, followed by explanation
h3 headings; the next h2 is “Walk it yourself.” There is no h2 parent for the
explanation beats. Heading navigation presents an unclear hierarchy to readers
using assistive technology.

**Standard:** Heading hierarchy best practice supporting WCAG 1.3.1. A skipped
level alone does not establish a WCAG failure.

**Recommendation:** Give the explanation a meaningful h2 parent or use h2 for
its top-level beats, preserving their existing visual styling and copy.
**Suggested command:** `$impeccable harden`.

### [P2] Font delivery has avoidable transfer cost

**Location:** `web/shared/fonts.css:19–47`; `web/shared/fonts/`.
**Category:** Performance.

The regular Cascadia Mono TTF occupies about 620 KiB and its italic face about
464 KiB; the two Instrument Serif TTFs occupy about 72 KiB each. Faces are loaded
on demand, so this is available asset size, not a claim that every page downloads
all four. The large regular monospace face adds cold-load work on attendee phones
and increases the size of inlined offline artifacts. `font-display: swap` already
keeps text visible while fonts load.

**Standard:** No specific WCAG failure; no measured latency or Core Web Vitals
failure is claimed.

**Recommendation:** Evaluate locally generated WOFF2 versions of the same
licensed fonts. Preserve required Rust symbols, variable weight coverage,
self-hosting and offline operation. Compare bytes and measured font swap/refit
before replacing the TTF sources.
**Suggested command:** `$impeccable optimize`.

### [P2] Machine-readable type documentation understates the implementation

**Location:** `DESIGN.md` frontmatter typography; `web/shared/tokens.css`
(`text-3xl`); `web/buzzer/buzzer.css:52–69`; `web/wall/wall.css`.
**Category:** Implementation integrity / documentation.

The imported draft frontmatter listed 12, 13, 14, 24 and 36px roles but omits the existing
30px shared token and intentional surface roles such as the 28px room code and
46px voting letters. The detector therefore reports existing implementation as
off-ramp. New agents could incorrectly “fix” established layouts to satisfy the
incomplete documentation. This gap was introduced by the documentation pass;
it is not evidence that these components need restyling.

**Recommendation:** Document reused surface-specific roles and distinguish the
wall's measured type from static phone/read sizes. Preserve the current code and
binding prototype decisions; do not force all surfaces onto the body ramp.
**Suggested command:** `$impeccable document`.

## Detector findings reviewed in context

- **Side-tab accents:** not defects here. Solid machine edges, dashed human
  provenance, trace focus, and company counts are established signal channels.
  Colour is supplemented by text, shape or glyphs. Generic anti-pattern advice
  does not supersede this product's contract.
- **Font-size advisories:** many are legitimate surface-specific values or wall
  sizes. The documentation gap above is the actionable finding; these are not
  independent layout defects.
- **Reduced-motion kill rules:** host has a 0.01ms guard, but currently has no
  animation whose feedback would be lost. Phase changes and labels persist.
  No motion sensitivity defect is established from that pattern alone.
- **Hard-coded colours:** shared syntax/provenance colours and literals matching
  existing house tokens warrant classification, not a new palette. Light-only v1
  is binding; lack of product dark mode is not a finding. The prototype dim-room
  proposal is not an implemented product theme to certify.

## Patterns and positive findings

The main gap is between broad machine checks and intentional surface decisions.
Keep component and phase context in future audits instead of normalizing every
literal or accented border mechanically.

Maintain native named controls, focus restoration after repaint, a separate
polite announcement region, 44px control minimums, meaningful selected states,
self-hosted font fallbacks, internal code scrolling on the reading surface,
source-fit measurement on the wall, and phase-gated syntax colour. The bounded
wall refit loop is required legibility work, not unbounded layout thrashing.

## Recommended actions — status rechecked 2026-10-11

1. **Open — [P2] `$impeccable harden`** — repair take-home heading semantics.
2. **Done — `$impeccable optimize`** — WOFF2 delivery and offline embedding are
   implemented; see `docs/performance/2026-10-03-fonts.md`. No fresh timing claim.
3. **Done — `$impeccable document`** — DESIGN.md records intentional typography
   roles, including the landing page and the wall's measured source type.
4. **Optional follow-up — `$impeccable polish`** — a bounded browser pass after
   the heading repair; this is a validation suggestion, not another open defect.

A new audit after application fixes can reassess the score; the historical 15/20
above is unchanged evidence from October 2.
