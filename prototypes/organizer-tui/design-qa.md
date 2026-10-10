# Step navigator design check

2026-10-10. Result: passed for the native terminal prototype.

## Reference and capture

- Selected reference: `/Users/michellerojas/.codex/generated_images/01a11448-1877-7670-9e66-13068358757f/exec-36597161-451e-4847-868d-0ff4ca020af4.png` (1487 × 1058).
- Actual capture: [recordings/step-navigator.png](recordings/step-navigator.png) (1490 × 1058).
- Terminal: 136 columns × 41 rows; q3, teaching step 2, Steps focus.
- Capture method: running Ratatui binary in a pseudo-terminal, recorded as asciicast and rendered with agg at font size 18. The reference and actual capture were inspected together.

## Comparison

The left step rail and stacked source/narration panes preserve the selected
layout. Dark background, light text, amber selection, and inverse source highlight
are present. Focus also has a text label, and selected/highlighted rows have `>`
markers. Source, narration, and the keyboard footer fit without horizontal scroll.
The authored teaching text is preserved. No raster assets are used by the app.

Terminal adaptations: typography follows terminal cells; the focused pane has
an amber border instead of boxing the selected row. The source highlight covers
the code text. Reveal is anchored at the bottom of the rail. Text is denser than
the mockup, and pane titles use native borders. The header says PROTOTYPE.
“Pause here” replaces “Pause and discuss” so speaking is optional.

The initial capture inherited NO_COLOR and omitted the focus colors. Removing
that variable for the recording fixed the capture; the app still respects it.

## Interaction checks

The recorded session opens the question, selects a step without opening it,
jumps with Enter, cycles focus through all three panes, advances to the last
teaching step, explicitly reveals, scrolls, opens/closes help, and exits cleanly.
Tests additionally cover the reveal boundary, Escape navigation, narrow layouts,
candidate changes, and unchanged bank files. Below 80 columns only the focused
pane is shown. This check does not establish projector fit or event readiness.
