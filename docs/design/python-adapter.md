# Python Adapter

The Python adapter is the first non-Vinglish producer of Semantic IR. It is a
translation boundary, not a reasoning or diagnostic subsystem.

## Boundary

```text
Python source -> Python standard ast -> private normalized DTO -> Semantic IR
```

The adapter embeds the installed Python runtime in-process through PyO3 and
calls `ast.parse`. It does not spawn a Python process, reimplement Python
parsing, or use a third-party parsing library. The normalized DTO is private to
this crate. A future parser replaces only the producer of that DTO; the
Semantic IR lowering and every downstream crate remain unchanged.

## Supported Mapping

| Python AST | Semantic IR |
| --- | --- |
| `Module` | `Program` and `Module` |
| `FunctionDef` | `Function` and `Parameter` |
| `Assign`, `AnnAssign` | first local `Variable`, later `Assignment` |
| `AugAssign` | `Assignment` with a mutation operation |
| expression `Call` | `Call` |
| `Return` | `Return` |
| `If` | `Conditional` |
| `For name in iterable` | `Loop::ForEach` |
| `While` | `Loop::While` |
| binary operations and comparisons | `BinaryOp` |
| unary operations | `UnaryOp` |
| list, tuple, and set literals | `Collection` |

Unsupported statements are omitted from the resulting graph. Unsupported
expressions are represented by a generic extension node, allowing supported
surrounding constructs to continue lowering deterministically.

## Language-Agnostic Proof

The adapter depends on `vz-semantic-ir`, but the reasoning engine does not
depend on this adapter, Python, or any parser API. The cross-language test in
the adapter compares a Python accumulator with the equivalent Vinglish export
fixture and asserts the same active `accumulator` hypothesis and confidence.
All six examples in `examples/python` are analyzed through the existing
reasoning engine without any Python-specific engine behavior.

The adapter test suite also passes a Python-derived `IntentReport` and a
portable diagnostic model to the existing semantic diagnostics crate. The
result is the same structured accumulator type-conflict interpretation used by
other adapters; diagnostics receives no Python AST or source text.
