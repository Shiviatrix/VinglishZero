# Verification Corpus

This directory is the stable entry point for Vinglish Zero's golden validation
corpus. The canonical executable examples remain in `examples/<language>/` so
that documentation, benchmarks, and validation exercise the same source files
without maintaining divergent copies.

`verification/expected/semantic-corpus.json` records the expected primary
intent and minimum confidence for each core pattern. `vz validate` loads those
expectations, lowers each canonical example through its adapter, and compares
the deterministic result.

Python, Java, C, and Vinglish transport are currently executable corpus
frontends. Rust, JavaScript, and TypeScript directories document their stable
adapter extension points; those frontends currently return a deterministic
unavailable-frontend diagnostic and are tested as such.
