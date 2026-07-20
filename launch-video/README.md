# Vinglish Zero Launch Film

Production-ready Remotion film for the Build Week submission.

## Render

```bash
npm install
npm run render
```

The render is exactly 180 seconds at 1920×1080 and 30 fps. `npm run assets`
generates local ambient, interaction, and macOS speech-synthesis tracks. If the
`say` command is unavailable, the render still succeeds without narration audio;
the timestamped `public/narration.srt` remains the canonical narration cue sheet.

```bash
npm run dev
npm run typecheck
npm run render:preview
```

The default composition deliberately renders without captions to preserve the
minimal launch-film visual language. Captions are shipped as `public/narration.srt`
for accessibility upload on YouTube.

See `PRODUCTION.md` for the locked timeline, narration, sound-cue, and factual
source notes.
