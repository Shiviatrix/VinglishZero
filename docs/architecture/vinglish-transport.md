# Vinglish Transport Boundary

The Vinglish adapter receives the compiler-owned `vinglish.semantic-export`
JSON contract, currently version `1`. It does not receive Vinglish HIR and has
no Cargo dependency on any Vinglish compiler crate.

```text
Vinglish compiler -- versioned JSON over stdout/file --> Vinglish Zero adapter
                                                        -> Semantic IR
                                                        -> semantic engine
```

The compiler owns the schema specification in the parent repository at
`docs/architecture/semantic-export.md`. This document explains the consumer
side of that contract.

## Adapter Boundary

The adapter performs exactly three operations:

1. Deserialize the versioned document.
2. Reject an unknown format or schema version.
3. Map supported transport concepts into language-agnostic Semantic IR nodes.

It does not read Vinglish source, invoke compiler passes, resolve names, infer
types, or explain code. Those are compiler and engine responsibilities.

## Source Adapter Convenience

The Vinglish `SourceAdapter` owns the narrow transport-acquisition boundary.
When the adapter receives a `.ving` file, it launches the installed compiler
as:

```bash
vng --emit-ir path/to/file.ving
```

The adapter captures stdout in memory and sends it to the exact same JSON
importer used for a `.json` file. The compiler executable defaults to `vng`;
users can set `VZ_VINGLISH_COMPILER` to an explicit executable path. Stderr and
the compiler exit code are retained in deterministic CLI errors.

This convenience does not create a library coupling: Zero invokes an external
process and consumes only its versioned JSON output. It has no compiler Cargo
dependency and does not parse `.ving` source.

## Mapping in Version 1

| Transport concept | Semantic IR concept |
| --- | --- |
| Program | Program |
| Module | Module |
| Function / Parameter | Function / Param |
| Variable | Variable |
| Assignment | Assignment |
| Mutation | Assignment with a semantic operation |
| Call | Call |
| Return | Return |
| While loop | Loop::While |
| Conditional | Conditional |
| Type description | TypeConcept |

Unsupported transport expressions become an extension node. This preserves the
source location while keeping version 1 narrow and preventing language syntax
from entering the shared IR.

## Why the Engine Is Language Agnostic

`vz-semantic-engine` accepts `SemanticGraph` only. Its explanation pass walks
functions, loops, assignments, calls, conditionals, and returns without
checking an adapter name, source format, or Vinglish-specific type. The same
graph shape can later be produced by other adapters without changing the
engine.

## Usage

The recommended workflow accepts source directly:

```bash
vz explain examples/fibonacci.ving
vz diagnose compiler-diagnostic.json examples/fibonacci.ving
```

The diagnostic command needs both a portable compiler diagnostic and program
input. For automation, first export from the compiler and then use the
transport document directly:

```bash
vng --emit-ir examples/fibonacci.ving > fibonacci.vinglish-export.json
cd VinglishZero
cargo run -p vz-cli -- explain ../fibonacci.vinglish-export.json
```

The JSON file is the only artifact passed between projects.
