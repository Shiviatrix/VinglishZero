# Source Adapter Registry

`vz` accepts a source path and delegates language selection to an
extension-indexed `AdapterRegistry`.

```text
source path
  -> AdapterRegistry
  -> SourceAdapter
  -> SemanticGraph
  -> existing semantic engine / reasoning / diagnostics
```

Every frontend implements `SourceAdapter`:

```rust
trait SourceAdapter: SemanticAdapter {
    fn extensions(&self) -> &[&str];
    fn language(&self) -> &'static str;
    fn semantic_graph(&self, source: &Path) -> Result<SemanticGraph, SourceAdapterError>;
}
```

The registry rejects duplicate extensions, so selection is deterministic and
never depends on registration order. The CLI contains no extension matching or
language frontend behavior.

## Current Frontends

| Extensions | Adapter | Status | Frontend boundary |
| --- | --- | --- | --- |
| `.py`, `.pyw` | Python | Operational | Embedded CPython `ast` module |
| `.ving`, `.json` | Vinglish | Operational | External `vng --emit-ir`, then versioned JSON import |
| `.rs` | Rust | Registered extension point | Requires rust-analyzer or stable Rust frontend integration |
| `.js`, `.mjs`, `.cjs` | JavaScript | Registered extension point | Requires TypeScript compiler API integration |
| `.ts`, `.tsx` | TypeScript | Registered extension point | Requires TypeScript compiler API integration |
| `.java` | Java | Operational | Java Compiler API (`javac`, `com.sun.source`) |
| `.go` | Go | Registered extension point | Requires `go/parser` and `go/ast` integration |
| `.c` | C | Operational | Clang JSON AST (`clang -Xclang -ast-dump=json`) |
| `.cc`, `.cpp`, `.cxx`, `.hpp`, `.hxx` | C++ | Registered extension point | Requires an official C++ frontend integration |
| `.cs` | C# | Registered extension point | Requires Roslyn integration |
| `.kt`, `.kts` | Kotlin | Registered extension point | Requires Kotlin compiler frontend integration |
| `.swift` | Swift | Registered extension point | Requires SwiftSyntax or official Swift parser integration |

Registered extension points deliberately return `frontend unavailable` until a
real official frontend lowers the language into `SemanticGraph`. They do not
use incomplete handwritten parsers.

## Vinglish Boundary

The Vinglish source adapter invokes the installed compiler only as an external
process. It captures `vng --emit-ir <source>` stdout in memory and imports the
compiler-owned versioned JSON document. No Vinglish crate, HIR type, or parser
is linked into Vinglish Zero. `VZ_VINGLISH_COMPILER` overrides the executable.

## Adding a Frontend

Implement `SourceAdapter` in an adapter crate, use the language's official
frontend to produce a `SemanticGraph`, and register it in the CLI composition
module. The semantic engine, reasoning engine, diagnostics, hypotheses, and
Semantic IR do not change.
