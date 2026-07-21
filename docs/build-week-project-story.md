# Vinglish Zero: Build Week Project Story

## Inspiration

Programming languages make the same idea look unrelated. A filter-and-reduce
written in Python, Java, C, or Vinglish may share almost no surface syntax,
even though a developer looking at each implementation would recognize the
same purpose immediately.

That gap was the starting point for Vinglish Zero. I wanted to build a tool
that reasons about what code *does*, rather than what it happens to look like.
The motivating question was simple: can we preserve the parts of program
understanding that are inspectable and repeatable, instead of asking a model to
guess?

Vinglish Zero is my answer: a deterministic, language-agnostic semantic
reasoning engine for source code. It is not a compiler, another programming
language, or an LLM wrapper. It receives code through language adapters,
lowers it into a shared Semantic IR, and derives semantic intent through
explicit rules.

## What I Built

The project turns source code into a common semantic representation and then
evaluates a fixed reasoning pipeline:

```text
Source code
  -> language adapter
  -> Semantic IR / Semantic Graph
  -> facts
  -> evidence
  -> hypotheses and constraints
  -> intent report, diagnostics, and semantic query metadata
```

The key boundary is the Semantic IR. After lowering, the reasoning engine does
not know whether a graph originated in Python, Java, C, or Vinglish. It sees
language-neutral concepts such as functions, loops, conditionals, calls,
assignments, mutation, return values, and collection traversal.

From those concepts, Zero derives facts and evidence, evaluates competing
hypotheses, eliminates hypotheses whose required constraints are absent, and
produces a structured `IntentReport`. The current corpus validates 32 semantic
patterns, including accumulators, filters, mappers, reducers, searches,
histograms, grouping, partitioning, composition pipelines, and collection
operations.

The design makes the core claim easy to inspect:

\[
\text{IntentReport} = f(\text{SemanticGraph})
\]

where \(f\) is deterministic. The same semantic graph produces the same facts,
evidence, rejected hypotheses, confidence values, and final report on every
run. There is no model inference, embedding lookup, external API call, or
probabilistic scoring in the runtime path.

Zero also includes semantic diagnostics. A compiler may report a type mismatch;
Zero can connect that issue to a demonstrated semantic role, such as an
accumulator within a reduction, and return structured suggestions grounded in
the report's evidence. Its query command searches intent reports and semantic
pipelines rather than source text, so a query such as `filter THEN reduce` is a
query over the cached semantic state.

## How I Built It

I built Vinglish Zero as a Rust workspace with deliberately narrow boundaries:

- adapters own language-frontend integration and only lower into Semantic IR;
- the semantic engine consumes only the shared IR;
- reasoning separates fact extraction, evidence providers, hypothesis
  registration, constraints, and confidence calculation;
- diagnostics consume compiler-neutral diagnostics plus intent reports;
- the query and incremental-cache layers consume deterministic reports rather
  than source syntax.

The Vinglish integration is especially intentional. Vinglish Zero does not
depend on compiler crates or expose compiler HIR. The compiler exports a
versioned JSON transport model; Zero imports that stable contract and lowers it
into its own IR. That lets the compiler and semantic engine evolve separately.

For the other supported frontends, Zero uses official language tooling rather
than a handwritten parser: CPython AST for Python, the Java Compiler API for
Java, and Clang tooling for C. Cross-language verification compares their final
semantic reports rather than their syntax trees. The current benchmark corpus
contains 92 cross-language samples, while the verification suite checks that
equivalent programs preserve their intended classification across the supported
frontends.

Codex and GPT-5.6 were used as engineering collaborators, not as a runtime
semantic oracle. They accelerated repository exploration, implementation,
testing, documentation, and iteration on adapter and verification boundaries.
The important architectural decision was to keep that assistance outside the
product's conclusions: every result emitted by Zero is still traceable to
versioned IR, explicit evidence, and deterministic code.

## Challenges

The hardest part was resisting the easiest-looking solution. It would have been
straightforward to ask an LLM what a function appears to do, but that would make
the system difficult to reproduce, compare, or trust in a tooling workflow.
Instead, the engine needed a small enough semantic vocabulary to work across
languages while retaining enough structure to distinguish similar patterns.

That created several practical challenges:

- **Language independence.** Adapters must normalize differences without
  leaking parser or compiler-specific concepts into the engine.
- **Deterministic ambiguity.** Real functions can plausibly look like several
  patterns. Zero keeps competing hypotheses, applies explicit eliminations, and
  exposes supporting and rejected evidence instead of hiding the decision.
- **Stable integration.** The Vinglish compiler and Zero needed a durable
  boundary, so the transport contract is versioned and deliberately separate
  from internal HIR.
- **Verification.** A claim of language agnosticism is weak without tests. The
  project therefore verifies equivalent intent reports across representative
  Python, Java, C, and Vinglish inputs.
- **Performance without obscurity.** Incremental caching, compact semantic
  blobs, and deterministic query metadata improve reuse, but the stored state
  must remain inspectable and versioned.

## What I Learned

I learned that semantic tooling gets more credible when its uncertainty is
modeled as structure rather than hidden behind fluent prose. A rejected
hypothesis is useful information. A confidence value means more when it is the
reproducible result of rule evaluation rather than an opaque prediction.

I also learned that a shared representation is only useful when its boundaries
are defended. The compiler does parsing; adapters do lowering; the engine does
reasoning. Keeping those responsibilities separate made it possible to add
frontends without adding language branches to the reasoning core.

Finally, I learned that AI-assisted development and deterministic software are
not opposites. GPT-5.6 and Codex made the engineering loop faster. The product
itself remains deliberately inspectable: users can ask why a classification was
made and follow the answer through graph, facts, evidence, constraints, and
report.

## Why It Matters

Developers increasingly work across languages, repositories, and abstractions.
Syntax search is still useful, but it cannot answer the question that matters
most during maintenance: *where is the code that performs this behavior?*

Vinglish Zero explores a different foundation for that question. It treats
semantic intent as a first-class, deterministic artifact that can be verified,
diagnosed, cached, and queried across language boundaries.

Code should not only be searchable by what it looks like. It should be
searchable by what it does.
