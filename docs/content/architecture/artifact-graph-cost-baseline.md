---
title: Artifact graph cost baseline
description: Compiler-operation measurements for the typed Atlas artifact graph
---

# Artifact graph cost baseline

This baseline measures compiler operation, not generated-code runtime behavior.
Each case creates one Atlas compilation, requests only the listed recipe roots,
executes the dependency closure, and drops the result. Six cases use one SFC;
the cross-file case adds a second SFC to exercise a real multi-source closure.

Measured on 2026-07-11 at canary commit
`52272c67ce5a213cabfbaf30d841f791f3c5339e`, using `rustc 1.95.0` on an Apple
M5 Pro (`arm64-apple-darwin`). The commit contains latest `main` through
`588c1e15d`, explicit frontend/peer-provider composition, request-keyed
multi-source planning, immutable snapshots, and cached provider observations.
Canon requests the descriptor and Croquis document, plus Relief only when a
template exists and Module only when a script exists. Flow is independent and
is not fabricated as a type-check dependency. Command:

```sh
cargo bench -p vize --bench artifact_graph -- \
  --warm-up-time 1 --measurement-time 3 --sample-size 100
```

| Recipe                         | Observed time interval | Allocations | Peak live bytes | Queries | Provider executions | Cached products |
| ------------------------------ | ---------------------: | ----------: | --------------: | ------: | ------------------: | --------------: |
| DOM backend product            |       42.950-43.141 us |         660 |         132,156 |      16 |                   9 |               9 |
| production SFC compiled module |       57.482-57.633 us |         837 |         169,522 |      23 |                  11 |              11 |
| DOM + SSR + Vapor products     |       46.150-46.269 us |         738 |         131,212 |      20 |                  11 |              11 |
| Patina document lint           |       37.490-37.604 us |         532 |         171,540 |      13 |                   7 |               7 |
| Canon SFC typecheck            |       34.377-34.495 us |         508 |         105,010 |      15 |                   7 |               7 |
| combined lint + typecheck      |       40.584-40.755 us |         579 |         176,956 |      19 |                   8 |               8 |
| two-source cross-file analysis |       79.547-79.813 us |         938 |         174,150 |      45 |                  25 |              13 |

Allocation counts and peak live bytes are deltas observed by the benchmark's
tracking global allocator; peak bytes are not process RSS. Timing is a
host-local canary signal, not a cross-machine speed claim or a claim that one
recipe is faster than another. Query, execution, cache, and allocation counts
are the deterministic structural measurements.

The DOM-product case measures a graph-native backend root. The production SFC
case additionally assembles the public compiled-module product. The
multi-backend case proves DOM, SSR, and Vapor share one frontend/Rendu closure
instead of reparsing the source for each target.

The combined tool case adds only one provider execution and one cached product
over either single-tool case. Its Patina and Canon roots share source-shaped
descriptor, Relief, Module, and Croquis work; neither requests Flow or Rendu.
The two-source case requests the full cross-file analyzer, not the lightweight
project index. Query-count assertions and zero-execution assertions for
unrequested products are enforced by the artifact-graph integration tests.
