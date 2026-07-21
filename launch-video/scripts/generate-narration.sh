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
  "What if you could query a ten-million-line codebase by meaning, not keywords? What if you could derive a function's intent across four languages, deterministically, without AI hallucination? That's the problem I set out to solve."
  "Take one behavior and write it four ways. The human intent is the same: keep what matters, transform it, and combine the result. Syntax is noise. To find the signal, I built Zero."
  "Zero is a language-agnostic semantic reasoning engine. Each adapter lowers source into a universal Semantic Graph. Once code enters Zero, the engine has no language-specific types or syntax. It only sees the intent."
  "Zero does not use AI at runtime. It extracts hard facts: loops, branches, mutations. Fixed rule constraints eliminate incompatible hypotheses in microseconds. For this function, Zero creates an inspectable proof: Filter, Mapper, Reducer."
  "When you understand intent, errors change completely. A compiler says type mismatch. But Zero knows this variable is an accumulator inside a reduction loop. It pinpoints the broken role and provides a one-to-one fix tied to the developer's intent."
  "For identical input, the report is byte stable: same graph, same evidence, same answer. Zero verifies 32 semantic patterns across four language transports. In the current corpus, the deterministic reasoning pass itself averages under one millisecond per sample."
  "I used Codex and G P T 5.6 as engineering partners: roughly twelve thousand seven hundred lines of Rust, four adapters, and an eighty-test verification suite in seven days. AI accelerated construction, but every conclusion is inspectable, rules-based code."
  "Because Zero caches intent, you can search repositories by meaning. Ask the C L I for filter then reduce, and it returns semantic matches from cached reports without reparsing source."
  "Code should not be searched by what it looks like. It should be searched by what it does. This is Zero."
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
