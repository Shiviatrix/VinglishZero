# Examples

These source examples are parsed by their registered frontend and lowered to
the shared Semantic IR. They are not copied into the reasoning engine.

`collection_patterns.py`, `CollectionPatterns.java`,
`collection_patterns.c`, and `collection-patterns.ving` provide aligned
examples for the deterministic collection-pattern corpus. The Vinglish
verification path consumes the equivalent compiler-owned JSON fixture at
`tests/fixtures/collection-patterns-v1.json`, preserving the transport-only
boundary between Vinglish and Vinglish Zero.

`compositions.py`, `CompositionPatterns.java`, `compositions.c`, and
`composition-patterns.ving` demonstrate an ordered `Filter -> Map -> Reduce`
pipeline. The matching transport fixture is `tests/fixtures/compositions-v1.json`.
