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
`a3aa89dda79e8102d965e6dd1f3db332c3884aae`, using `rustc 1.95.0` on an Apple
M5 Pro (`arm64-apple-darwin`). The commit contains latest `main` through
`bfeeff6e6`, explicit frontend/peer-provider composition, request-keyed
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
| DOM backend product            |       42.480-42.658 us |         653 |         131,821 |      16 |                   9 |               9 |
| production SFC compiled module |       58.582-60.663 us |         830 |         169,187 |      23 |                  11 |              11 |
| DOM + SSR + Vapor products     |       46.151-46.869 us |         731 |         130,877 |      20 |                  11 |              11 |
| Patina document lint           |       38.042-38.419 us |         525 |         171,205 |      13 |                   7 |               7 |
| Canon SFC typecheck            |       34.645-35.001 us |         501 |         104,675 |      15 |                   7 |               7 |
| combined lint + typecheck      |       40.682-40.806 us |         572 |         176,621 |      19 |                   8 |               8 |
| two-source cross-file analysis |       79.754-80.067 us |         931 |         173,743 |      45 |                  25 |              13 |

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
