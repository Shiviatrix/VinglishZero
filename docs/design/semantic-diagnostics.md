# Semantic Diagnostic Pipeline

Semantic diagnostics combine two portable inputs without reading source code:

```text
CompilerDiagnostic + IntentReport
  -> DiagnosticRuleRegistry
  -> SemanticDiagnosticEngine
  -> SuggestedFix templates
  -> SemanticDiagnostic
```

## Inputs

`CompilerDiagnostic` is language-neutral: code, severity, category, optional
source location, compiler message, and optional function name. An adapter for
any language can export this shape.

`IntentReport` comes from the deterministic reasoning engine. The diagnostics
crate accepts it through serialization and reads a private projection of only
the stable report fields it needs. This prevents a dependency cycle while the
CLI still passes the actual report produced by `ReasoningEngine`.

## Matching and Fixes

Rules match a diagnostic category and hypothesis identifier. Current rules
include `accumulator + type_mismatch`, `average + missing_division`, and
`resource_manager + move_after_use`. Rules select interpretation and fix
templates; they never inspect compiler implementation details.

Suggestions contain enum-based title, explanation, rationale, deterministic
integer confidence, and the supporting reasoning evidence. They do not contain
generated prose or AI output.

## CLI

```bash
vz diagnose compiler-diagnostic.json vinglish-export.json
```

The command imports the Semantic IR transport, runs deterministic reasoning,
matches the supplied diagnostic, and writes a JSON `SemanticDiagnostic`.
