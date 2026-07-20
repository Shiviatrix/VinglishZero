# Production Notes

## Master

- Runtime: 179 seconds at 3840×2160, 30 fps, safely below the upload limit.
- Composition: `VinglishZeroLaunch`.
- 4K final render: `npm run render:4k`.
- 4K final render with presenter: `npm run render:presenter`.
- Captions: upload `public/narration.srt` with the YouTube video.
- The supplied macOS voice is only a timing reference. Record the narration
  yourself and replace the generated `public/audio/narration-*.wav` files.
- Presenter footage is muted by design. Use its matching recording as the
  source for the final narration tracks; this avoids a double-audio render.

## Visual Direction

The visual system uses a plain matte-black ground, saffron as the primary
accent, warm-grey surfaces, and occasional pastel proof accents. There are no
gradients, glass panels, texture fields, or background ornamentation. Every
animated mark must explain a real system boundary or a real reasoning step.
Manim handles the explanatory transformations, so the animation has the
deliberate, mathematical feel of a technical essay rather than a dashboard
demo. Remotion keeps your face, the real code, and the actual CLI vocabulary in
the edit.

Avoid generic tech-film habits: floating code rain, exaggerated neon, terminal
wallpaper, decorative gradient light, faded logo watermarks, and a permanent
presenter thumbnail. Keep each proof visual sparse and self-contained.

## Locked Timeline

| Time | Speaker beat | Visual | Evidence source |
| --- | --- | --- | --- |
| 00:00–00:15 | Four versions of one behavior | You on camera; direct opening question | `examples/*/compositions.*` |
| 00:15–00:29 | Syntax is not the stable part | Minimal Python/Java/C/Vinglish code comparison | shipped examples |
| 00:29–00:52 | Shared Semantic Graph | Manim convergence and `Filter → Map → Reduce` | adapter boundary, composition report |
| 00:52–01:18 | Deterministic reasoning | Manim facts → evidence → active/rejected hypotheses | `crates/reasoning`, live `vz explain` output |
| 01:18–01:44 | Semantic diagnostics | Manim reframe of type mismatch | `tests/fixtures/type-mismatch.json`, `vz diagnose` |
| 01:44–02:06 | Reproducibility and boundary | Manim repeated report; compact presenter | deterministic tests, `vz verify` |
| 02:06–02:31 | How Codex/GPT-5.6 helped | You on camera; concrete engineering artifacts | Rust workspace and verification corpus |
| 02:31–02:49 | Semantic query | Real CLI syntax and report-shaped result | `vz query` design and CLI |
| 02:49–02:59 | Close | Logo, repository URL, compact presenter | shipped SVG and repository remote |

## Factual Guardrails

- Say “Python, Java, C, and Vinglish transport” when describing the verified
  cross-language corpus. Do not imply the unavailable registry placeholders are
  supported frontends.
- Say “Semantic Graph” only as the current public product term. The Rust crate
  is named `semantic-ir`; the film does not claim a different architecture.
- “Byte-stable” describes repeated reasoning reports for identical input,
  demonstrated by the deterministic test suite.
- The `type mismatch` segment uses the actual `TYPE_MISMATCH` fixture and the
  produced `accumulator_type_conflict` interpretation. Do not replace it with a
  simulated error.
- The Codex/GPT-5.6 segment explains implementation assistance, not inference
  at runtime. Vinglish Zero's reasoning path is deterministic and contains no
  model, embedding, or remote-service call.

## Recording Direction

- Start conversationally, as if inviting the viewer to look at a small code
  puzzle with you. Do not announce the project like a pitch deck.
- Pause after “the behavior is the same,” “not an AI guess,” and “byte-stable.”
  Those are the three ideas the audience should retain.
- In the Codex segment, speak plainly and specifically. The credibility comes
  from the boundary: Codex accelerated the build; deterministic rules make the
  final product conclusion inspectable.
- Deliver the final line softly and leave half a second of quiet before the
  music resolves.

## Source Assets

- Official mark: `../vinglish-zero.svg`, copied by `scripts/generate-audio.mjs`.
- Manim source: `manim/semantic_story.py`.
- Rendered Manim clips: `public/manim/` (generated, not committed).
- Narration script: `public/narration.txt`.
- Accessibility captions: `public/narration.srt`.
- Presenter footage: `public/presenter.mp4` (local and ignored).
