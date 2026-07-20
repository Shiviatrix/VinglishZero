#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUT="$ROOT/public/audio"
mkdir -p "$OUT"

if ! command -v say >/dev/null 2>&1 || ! command -v afconvert >/dev/null 2>&1; then
  echo "macOS say/afconvert not available; retaining generated silent narration tracks."
  exit 0
fi

segments=(
  "Vinglish Zero. Deterministic semantic reasoning for source code."
  "Now look at the same behavior in four languages. Different syntax. Different frontends. The same underlying intent."
  "Vinglish Zero lowers each one into a shared semantic representation. The code disappears. The meaning remains."
  "Filter. Map. Reduce. One ordered semantic pipeline, recovered without a model, embeddings, or a remote service."
  "Here is the reasoning path. Source becomes Semantic I R. I R becomes facts. Facts produce evidence. Evidence activates compatible hypotheses and eliminates the rest. The result is not a guess. It is an inspectable conclusion."
  "The same evidence can interpret a failure. A type mismatch inside an accumulator is not just an error string. It conflicts with the inferred accumulator contract, so the diagnostic explains the semantic role that broke."
  "On a larger function, multiple patterns remain separate and ordered. A filter can feed a mapper, which feeds a reducer. The engine preserves the composition instead of flattening it into a label."
  "Run the same analysis twice. The report is byte stable. Same input. Same evidence. Same conclusion."
  "Under the surface: isolated adapters, a typed semantic graph, deterministic scheduling, verification fixtures, benchmarks, reproducible reports."
  "Different syntax. Same meaning. Deterministic reasoning you can inspect."
)

for index in "${!segments[@]}"; do
  number=$(printf '%02d' "$((index + 1))")
  aiff="$OUT/narration-$number.aiff"
  wav="$OUT/narration-$number.wav"
  say -v Samantha -r 140 -o "$aiff" "${segments[$index]}"
  afconvert -f WAVE -d LEI16@22050 -c 1 "$aiff" "$wav" >/dev/null
  rm -f "$aiff"
done

echo "Generated local narration tracks with macOS speech synthesis."
