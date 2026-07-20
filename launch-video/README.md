# Vinglish Zero Build Week Film

This is the three-minute Build Week launch film for Vinglish Zero. It combines
your recorded on-camera narration with original Manim sequences and Remotion
editing. Every product claim in the film maps to an existing command, fixture,
or design document in the repository.

## The Story

The film makes one argument, in this order:

1. Equivalent behavior can be expressed in very different source syntaxes.
2. Adapters lower those forms into one language-independent Semantic Graph.
3. Facts, evidence, hypotheses, and constraints produce an inspectable intent
   report and ordered semantic pipeline.
4. That reasoning can reframe a compiler diagnostic in terms of semantic role.
5. Identical graphs produce byte-stable reports, verified across the supported
   Python, Java, C, and Vinglish transport corpus.
6. Codex and GPT-5.6 accelerated implementation; the shipped analysis remains
   deterministic symbolic reasoning rather than model inference.
7. The same reports make semantic query possible.

The canonical speaker script is [public/narration.txt](public/narration.txt).
It is written for a natural three-minute delivery, not for synthetic narration.

## Presenter Footage

Record one clean, horizontal 4K (3840×2160) clip around three minutes long. Look
into the lens and leave generous headroom. The composition crops it into a
large opening and Codex segment plus compact proof points; it never uses a
generic talking-head frame just to fill space.

Place the recording at `public/presenter.mp4`, then render it with:

```bash
npm run render:presenter
```

`presenter.mp4` is intentionally ignored by Git. Without this optional prop,
the launch film still renders as a voiceover-led product film.

## Render

Prerequisites: Node 20+, a local Remotion install, Manim Community Edition, and
FFmpeg (installed by Manim on most development machines). macOS also enables a
local `say` preview voice; it is not used for the final submission.

```bash
npm install
npm run typecheck
npm run render:manim
npm run render:preview # 1920×1080 review export
npm run render:4k      # 3840×2160 final export
npm run render:presenter # 3840×2160 final export with presenter.mp4
```

`npm run assets` renders four original Manim clips, copies the repository logo,
and generates a local scratch narration track. It is automatically invoked by
all render commands. Final output is 3840×2160, 30 fps, H.264, and exactly
179 seconds, keeping the rendered upload safely below the Build Week
three-minute limit. `render:preview` is deliberately a 1920×1080 review file;
the final and presenter render commands preserve the full 4K composition.

For the final cut, replace the generated `public/audio/narration-*.wav` files
with mastered recordings matching the segment boundaries in
[public/narration.txt](public/narration.txt). Presenter footage is deliberately
muted in the composition, so extract or record its audio separately before
replacing those tracks. Upload
[public/narration.srt](public/narration.srt) to YouTube for captions.

## Visual Rules

- No background grids, persistent connection lines, dashboard chrome, or
  unexplained side text.
- Backgrounds are plain matte black. Pastel color identifies a concrete stage;
  the proof visuals remain deliberately sparse.
- Manim carries the conceptual transformations; Remotion handles pace, camera
  footage, product code, sound, and final assembly.
- A frame should prove exactly one thing before it transitions.
- The on-camera segments make the work personal; the full-screen proof
  sequences give the technical claims room to land.

See [PRODUCTION.md](PRODUCTION.md) for the locked edit plan, source-of-truth
references, and final-recording checklist.
