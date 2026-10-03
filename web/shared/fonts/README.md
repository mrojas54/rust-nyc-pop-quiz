# Vendored fonts

Self-hosted, no font CDN — D-14, `SPEC.md` §15. Three criteria depend on there
being no font request at runtime: AC-77 (a segment runs with outbound network
blocked), AC-102 (the static fallback makes zero network requests), and the
wall's type model, which measures type it must actually have.

Declared in `../fonts.css`. Referenced through `--font-mono` and
`--font-heading` in `../tokens.css`.

## What is here, and where each file came from

| File | Family | Source | Version | SHA-256 |
|---|---|---|---|---|
| `CascadiaMono-VariableFont_wght.ttf` | Cascadia Mono, upright, variable `wght` 200–700 | **the Rust NYC Design System**, `project/fonts/` | artifact version `1789942372-baaa` | `85c0d075ccb628edef86d48fa9ad9ce0ff69b93e83123ea42461ea62151244ce` |
| `CascadiaMono-Italic-VariableFont_wght.ttf` | Cascadia Mono, italic, variable `wght` 200–700 | **the Rust NYC Design System**, `project/fonts/` | artifact version `1789942372-baaa` | `522c2c5f31c7256e49720ca702f8c2f2a363e12e7c26cca3e778f1c56bd109ee` |
| `OFL-CascadiaMono.txt` | — | the design system, `project/assets/fonts/OFL.txt` | artifact version `1789942372-baaa` | `7e52740dad2c7064a313bf3994e63d3edac66ef113872b5fff6a811bb64c16b6` |
| `InstrumentSerif-Regular.ttf` | Instrument Serif, upright | `github.com/google/fonts`, `ofl/instrumentserif/` | commit `0b58fb370093f9a9f4ff785d94405710b79de67c` (2026-03-03) | `498efd461f6ddfcb7a111bf9a565709d2085d48201d501ead960d93e84ffbb88` |
| `InstrumentSerif-Italic.ttf` | Instrument Serif, italic | `github.com/google/fonts`, `ofl/instrumentserif/` | commit `0b58fb370093f9a9f4ff785d94405710b79de67c` (2026-03-03) | `08939b8bdf534afec24ae0ef5e03f948940cd9a8fe08e7fecbad040e62327385` |
| `OFL-InstrumentSerif.txt` | — | `github.com/google/fonts`, `ofl/instrumentserif/OFL.txt` | commit `0b58fb370093f9a9f4ff785d94405710b79de67c` | `129ed7618959716959f2941fdd5b49e0ad6e6c1d78726761786a00253d865521` |

Both families are SIL Open Font License 1.1. Both licence files ship beside the
faces, which is what the OFL requires of a redistribution.

- Cascadia Mono: © 2019–present Microsoft Corporation, Reserved Font Name
  *Cascadia Code*.
- Instrument Serif: © 2022 The Instrument Serif Project Authors
  (`github.com/Instrument/instrument-serif`).

## Why the two sources differ

`SPEC.md` §15 says both faces "ship in `web/shared/fonts/` from the design
system's `assets/fonts/`". **That is true of Cascadia Mono and not of Instrument
Serif.**

The design system (`claude.ai/artifact/2k3zt9WWz6XUZRjkLDWvfi`, 123 files)
publishes both Cascadia Mono faces and their OFL, and carries the `@font-face`
rules this build's `fonts.css` is ported from. It publishes **no Instrument
Serif file at all** — `--font-heading` names the family, and the prototype got
the actual face from the Google Fonts CDN `@import` that D-14 forbids.

So Cascadia Mono comes from the design system exactly as §15 says, and
Instrument Serif comes from its upstream OFL release, pinned to a commit. The
gap in §15 is reported to the Orchestrator as a contract defect (Lattice PQ-2);
it is not a licence or quality difference, and `google/fonts` is the project's
own distribution channel.

## Re-fetching

Cascadia Mono, with the Artifact tool against the design system:

    read  https://claude.ai/artifact/2k3zt9WWz6XUZRjkLDWvfi
          paths: project/fonts/CascadiaMono-VariableFont_wght.ttf
                 project/fonts/CascadiaMono-Italic-VariableFont_wght.ttf
                 project/assets/fonts/OFL.txt

Instrument Serif, pinned so the bytes are reproducible:

    BASE=https://raw.githubusercontent.com/google/fonts/0b58fb370093f9a9f4ff785d94405710b79de67c/ofl/instrumentserif
    curl -sSfL -O "$BASE/InstrumentSerif-Regular.ttf"
    curl -sSfL -O "$BASE/InstrumentSerif-Italic.ttf"
    curl -sSfL -o OFL-InstrumentSerif.txt "$BASE/OFL.txt"

Verify against the table before committing:

    shasum -a 256 *.ttf *.txt

## Compressed browser delivery

Each TTF has a losslessly compressed `.woff2` sibling. `fonts.css` prefers
WOFF2 and retains TTF as a compatibility fallback. Both formats are served
locally by the room. The single-file offline fallback embeds only WOFF2, with
`data:font/woff2` MIME, so it does not carry duplicate fonts.

Original TTFs and licence files remain unchanged. No subsetting or weight
instancing is applied. Generate siblings with FontTools 4.60.1 and Brotli 1.1.0:

```python
from pathlib import Path
from fontTools.ttLib import TTFont

for source in Path("web/shared/fonts").glob("*.ttf"):
    font = TTFont(source)
    font.flavor = "woff2"
    font.save(source.with_suffix(".woff2"))
```

| File | SHA-256 |
|---|---|
| `CascadiaMono-VariableFont_wght.woff2` | `d4b4998ad47f83e9d744d077b13a08506ebada62002ce93498d599783ee5def6` |
| `CascadiaMono-Italic-VariableFont_wght.woff2` | `5fea6440656116f022cc02e52fad389009cf18143ba892eb6ff0e622e41ce124` |
| `InstrumentSerif-Regular.woff2` | `a85235850e4bccf3f6a9f3d686eee1ec160cb388377ed83aa2a12c9e36be59f3` |
| `InstrumentSerif-Italic.woff2` | `ddd5635a58d34da302cb5ef468fecc7cc12bd25320b3a8b251349433ac4954d5` |

Measurements and validation: [font performance report](../../../docs/performance/2026-10-03-fonts.md).
