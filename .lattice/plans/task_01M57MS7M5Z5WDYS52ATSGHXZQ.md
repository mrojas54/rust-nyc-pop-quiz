# PQ-78: Home's trace step buttons have no .btn base rule; the browser draws their edges

Found while building PQ-43 (PR #91, merged at 8098174). web/shared/trace.js renders the take-it-home step buttons as <button class="btn">, and web/home/home.css:64 sizes them (.walk .trace-nav .btn), but nothing in home's stylesheet chain (shared/tokens.css, shared/components.css, home/home.css) defines a .btn base rule: only web/host/host.css and web/buzzer/buzzer.css do. So on home those buttons get the browser's default button look, including its edge, which a11y.test.js does not check. PQ-43's owner ruling (2026-10-18) holds .btn edges at 3:1 (SC 1.4.11): --text-muted for neutral, --accent-hover for primary. Decide whether home's trace buttons get the same .btn rule (as host/buzzer carry it) and hold their edge in ac84_home; re-find line numbers first. Acceptance: just a11y and just test-web green; the edge check fails under its mutation; no copy or token value changes; AC-40, AC-33 hold.

# Plan (delegator, 2026-10-30)

**Stylesheet chain today.** `web/home/index.html:7-9` loads `shared/tokens.css`, `shared/components.css`, `home/home.css`. The only `.btn` rule in that chain is `web/home/home.css:64` (`.walk .trace-nav .btn`, sizing only). `web/shared/trace.js:133-137` renders the buttons with inline `min-height:34px;padding:4px 12px`. So the browser draws the edge, fill and font. The prototype page carries `.btn` through `prototypes/_shared/tokens.css:230-240`, so the base rule restores the designed look.

**The rule.** Add to `web/home/home.css`, scoped like the buzzer's (`web/buzzer/buzzer.css:29-38`): `.home-page .btn`, `:hover`, `.btn-primary`, `.btn-primary:hover`, `:disabled`. Edge: `--text-muted` neutral, `--accent-hover` primary (PQ-43 ruling). home.css only loads on home, and `.home-page` is on its `<body>`, so nothing leaks. Focus stays with the existing `.home-page :focus-visible` (home.css:90), so no `.btn:focus-visible` is added. trace.js's inline padding and 34px min-height still win over the base rule, and home.css:64 still lifts the height to 44px; the trace buttons keep today's size.

**The a11y cell.** In `ac84_home`, at step 1 (the back button is enabled, the forward one primary): `edge('home', html, '.btn:not(.btn-primary)', 'border', parentBg, ...)` and `edge('home', html, '.btn-primary', 'border-color', parentBg, ...)`, same as ac84_host. RED first: the cell fails today because no border is declared.

**Mutations (each bounded, each reverted).** (1) remove the base rule; (2) neutral edge to `--border-default`; (3) primary edge to `--accent-primary`. Each must fail `ac84_home`.

**What will look different.** Home's step buttons change from the browser default to the white mono button with a slate edge; forward becomes amber with a deep-amber edge and dark text; disabled ones halve in opacity. No size, copy or token change. If it fights `.walk .trace-nav .btn` or DESIGN.md in the rendered page, halt with screenshots (ruling 5).
