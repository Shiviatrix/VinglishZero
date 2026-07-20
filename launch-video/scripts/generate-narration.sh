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
  "Take one function and write it four times: Python, Java, C, and Vinglish. The braces change. The types move. But the behavior is the same: keep the values that matter, transform them, combine the result."
  "Vinglish is a language I built. Zero is deliberately separate from it: the compiler exports versioned J S O N, and Zero sees only the meaning on the other side."
  "The project is built on a simple idea: programming languages express syntax differently; they express intent similarly. Each adapter lowers a language through its own frontend into a shared Semantic Graph. After that point, the engine does not know which language wrote it."
  "And it is deliberately not an A I guess. It extracts facts: loops, branches, returns, calls, mutations. Independent providers turn those facts into evidence. Deterministic constraints activate compatible hypotheses and eliminate the rest. For this function, the report is a real, ordered pipeline: Filter, Mapper, Reducer."
  "That structure also changes what an error can mean. Here, the compiler says only type mismatch. Zero sees an accumulator: a value updated through a loop and returned as a number. So it can identify an accumulator type conflict, and give fixes that are tied to that role."
  "Run it twice and the report is byte stable. Same graph, same evidence, same answer. The verifier exercises 32 semantic patterns across Python, Java, C, and Vinglish transport. Vinglish itself remains decoupled: the compiler and Zero meet only at a versioned J S O N boundary."
  "I used Codex with G P T 5.6 as an engineering partner to build the workspace around that idea: the Rust crates, adapters, corpus, verification, and launch tooling. But the key decision was mine: the final conclusion should be inspectable code, not a model response. You can trace every result back through rules and evidence."
  "That makes a different kind of developer tool possible. Instead of searching for filenames or keywords, ask for a meaning: filter then reduce. The query matches functions through their stored semantic reports, without reparsing source code."
  "Vinglish Zero. Different syntax. Same meaning. And every conclusion comes with its trail."
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
