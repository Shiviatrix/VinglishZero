# Production Notes

## Master

- Runtime: 179.9 seconds, leaving encoder-padding headroom inside the 2:55–3:00 requirement.
- Format: 1920×1080, 30 fps, H.264, CRF 17.
- Composition: `VinglishZeroLaunch`.
- Render: `npm run render`.

## Visual Identity

- Ground: deep charcoal drafting field with sandstone construction rules.
- Surfaces: opaque warm-ivory annotation plates with hard printed offsets.
- Semantic accents: peacock teal, marigold saffron, emerald, vermilion, and muted sandstone.
- Motion language: measured assembly, drawn connectors, and physical registration rather than glow, glass, or particle effects.
- Logo: the shipped `vinglish-zero.svg` is copied into the render public directory by the asset step. The ending lets the semantic graph disappear into a traced construction of the official mark before revealing the SVG itself.

## Timeline

| Time | Beat | Source of truth |
| --- | --- | --- |
| 00:00–00:08 | Logo-led launch title resolving into the Python hook | official SVG and `examples/python/compositions.py` |
| 00:08–00:30 | Python, Java, C, Vinglish syntax | shipped composition examples |
| 00:30–00:55 | Shared semantic pipeline | verified composition pipeline |
| 00:55–01:35 | Facts, evidence, activated/rejected hypotheses | reasoning model and fixtures |
| 01:35–01:55 | Type mismatch interpretation | `vz diagnose` fixture output |
| 01:55–02:15 | Filter → mapper → reducer composition | composition fixture |
| 02:15–02:30 | Byte-stable report demonstration | repeated `vz diagnose` SHA-256 |
| 02:30–02:45 | Engineering depth | repository structure and verification surface |
| 02:45–02:59.9 | Graph pull-back and repository URL | Git remote |

## Narration and Captions

- Narration script: `public/narration.txt`.
- YouTube caption upload: `public/narration.srt`.
- Local preview voice: macOS `Samantha`, generated at 140 words per minute.
- Replace only the generated `public/audio/narration-*.wav` files for a studio voiceover; timing remains unchanged.

## Sound Design

- `ambient.wav`: low, deterministic generative room tone.
- `click.wav`: language/front-end and repeat-run arrivals.
- `rise.wav`: semantic convergence and final pull-back.
- `pulse.wav`: hypothesis and composition activation.
- `error.wav`: diagnostic transition.

All cues are synthesized locally by `scripts/generate-audio.mjs`; no third-party audio asset is required.

## Integrity

The determinism frame uses the actual SHA-256 prefix `f81f…2b97`, obtained by running the shipped `vz diagnose` command twice against `tests/fixtures/type-mismatch.json` and `tests/fixtures/accumulate-v1.json`. The verified remote shown in the final frame is `github.com/Shiviatrix/VinglishZero`.
