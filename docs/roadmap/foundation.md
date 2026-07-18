# Foundation Roadmap

The first version of Vinglish Zero should build trust in the architecture
before it builds capability.

## Phase 1

- freeze the workspace layout
- keep the Semantic IR stable enough for adapters to target it
- define engine module boundaries
- keep the CLI surface explicit

## Phase 2

- add the first adapter implementation
- add graph import and export paths
- add diagnostic rendering
- add semantic queries over the IR

## Phase 3

- add language-specific adapter coverage
- add reasoning workflows for explanation and repair
- add documentation and test generation surfaces
- harden compatibility and versioning

The key rule is to avoid premature coupling. New behavior should land behind
clear boundaries so the repository can remain maintainable for years.
