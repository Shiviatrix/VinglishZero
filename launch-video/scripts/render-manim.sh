#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SOURCE="$ROOT/manim/semantic_story.py"
OUTPUT="$ROOT/public/manim"
MEDIA="$ROOT/.manim-media"

if ! command -v manim >/dev/null 2>&1; then
  echo "Manim is required to render the launch-film animations. Install it with 'pipx install manim' or see https://www.manim.community/." >&2
  exit 1
fi

mkdir -p "$OUTPUT" "$MEDIA"

for scene in SemanticLift ReasoningProof DiagnosticReframe DeterminismProof; do
  manim -qh -r 3840,2160 --format=mp4 --media_dir "$MEDIA" "$SOURCE" "$scene"
  clip=$(find "$MEDIA/videos" -type f -name "$scene.mp4" -print -quit)
  if [[ -z "$clip" ]]; then
    echo "Manim did not produce $scene.mp4." >&2
    exit 1
  fi
  cp "$clip" "$OUTPUT/$scene.mp4"
done

echo "Rendered Manim launch-film clips to public/manim/."
