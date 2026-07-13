---
title: Artifact graph cost baseline
description: Compiler-operation measurements for the typed Atlas artifact graph
---

# Artifact graph cost baseline

This baseline measures compiler operation, not generated-code runtime behavior.
Each case creates one Atlas compilation for the same SFC, requests only the
listed recipe roots, executes the dependency closure, and drops the result.

Measured on 2026-07-10 from the canary working tree based on
`80191eac4fc7cc0d5e19609515345e843ea62259`, using `rustc 1.95.0` on an Apple
M5 Pro (`aarch64-apple-darwin`). This run includes request-keyed multi-source
planning, immutable snapshots, project-provider registration, and cached
transitive provider observations. Canon consumes the descriptor, Relief syntax,
and Croquis semantics; Flow is an independent product and is not fabricated as
a type-check dependency. Command:

```sh
cargo bench -p vize --bench artifact_graph -- \
  --warm-up-time 1 --measurement-time 3 --sample-size 100
```

| Recipe                    | Observed time interval | Allocations | Peak live bytes | Queries | Provider executions | Cached products |
| ------------------------- | ---------------------: | ----------: | --------------: | ------: | ------------------: | --------------: |
| compiler-only (DOM)       |       22.251-23.004 us |         230 |          27,981 |       6 |                   5 |               5 |
| lint-only                 |       38.739-39.405 us |         247 |          57,397 |       7 |                   5 |               5 |
| typecheck-only            |       63.822-66.891 us |         366 |          62,185 |      10 |                   6 |               6 |
| combined lint + typecheck |       60.043-61.243 us |         382 |          63,897 |      12 |                   7 |               7 |

Allocation counts and peak live bytes are deltas observed by the benchmark's
tracking global allocator; peak bytes are not process RSS. Timing is a
host-local canary signal, not a cross-machine speed claim or a claim that one
recipe is faster than another. Query, execution, cache, and allocation counts
are the deterministic structural measurements.

The combined case executes the shared Croquis semantic product once. Its two
tool roots add one provider execution, one cached product, two queries, and
sixteen allocations over typecheck-only, rather than rebuilding the
syntax/semantic/Flow closure. Query-count assertions and zero-execution
assertions for unrequested products are also enforced by the artifact-graph
integration tests.
