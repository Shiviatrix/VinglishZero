# Architecture Overview

Vinglish Zero is organized around one rule: all language-specific knowledge is
confined to adapters. Everything downstream from Semantic IR is language
agnostic.

```mermaid
flowchart TD
  Source["Source code or HIR"] --> Adapter["Language adapter"]
  Adapter --> IR["Semantic IR"]
  IR --> Engine["Semantic engine"]
  Engine --> Diagnostics["Diagnostics"]
  Engine --> Reasoning["Reasoning"]
  Reasoning --> CLI["CLI / export surfaces"]
```

## Boundary Rules

- Adapters translate syntax or HIR into shared semantic concepts.
- Semantic IR carries meaning, relationships, and provenance, not compiler
  implementation details.
- The engine consumes only Semantic IR.
- Diagnostics describe semantic findings, not parser failures or lowering
  internals.
- Reasoning composes engine capabilities and stays IR-centric.

## Repository Rule

This repository may live inside the parent Vinglish checkout, but it is not part
of the parent workspace and must remain an independent git repository.
