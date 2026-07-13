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
`cce266d6178fdfd0b603dd8d874ece512aa8864f`, using `rustc 1.95.0` on an Apple
M5 Pro (`arm64-apple-darwin`). The commit contains latest `main` through
`588c1e15d`, explicit `SourceKind` arbitration, parse-once script facts,
request-keyed multi-source planning, immutable snapshots, and cached provider
observations. Rendering requests the narrow SFC template-binding product rather
than a full Croquis document. Canon requests Croquis plus source-shaped Relief
and Module dependencies; Flow remains independent and is not fabricated as a
type-check dependency. Command:

```sh
cargo bench -p vize --bench artifact_graph -- \
  --warm-up-time 1 --measurement-time 3 --sample-size 100
```

| Recipe                         | Observed time interval | Allocations | Peak live bytes | Queries | Provider executions | Cached products |
| ------------------------------ | ---------------------: | ----------: | --------------: | ------: | ------------------: | --------------: |
| DOM backend product            |       33.061-33.128 us |         577 |         100,207 |      15 |                   8 |               8 |
| production SFC compiled module |       49.357-49.450 us |         809 |         144,729 |      23 |                  11 |              11 |
| DOM + SSR + Vapor products     |       36.304-36.468 us |         661 |         101,687 |      19 |                  10 |              10 |
| Patina document lint           |       39.009-39.084 us |         556 |         170,508 |      13 |                   7 |               7 |
| Canon SFC typecheck            |       35.495-35.668 us |         532 |         104,218 |      15 |                   7 |               7 |
| combined lint + typecheck      |       42.018-42.287 us |         604 |         176,372 |      19 |                   8 |               8 |
| two-source cross-file analysis |       81.828-82.097 us |         984 |         171,638 |      45 |                  25 |              13 |

Allocation counts and peak live bytes are deltas observed by the benchmark's
tracking global allocator; peak bytes are not process RSS. Timing is a
host-local canary signal, not a cross-machine speed claim or a claim that one
recipe is faster than another. Query, execution, cache, and allocation counts
are the deterministic structural measurements.

The DOM-product case measures a graph-native backend root. The production SFC
case additionally assembles the public compiled-module product. The
multi-backend case proves DOM, SSR, and Vapor share one frontend/Rendu closure
instead of reparsing the source for each target. Relative to the preceding
canary baseline, the DOM and multi-backend closures each remove one query,
provider execution, and cache entry by avoiding the full semantic document.

The combined tool case adds only one provider execution and one cached product
over either single-tool case. Its Patina and Canon roots share source-shaped
descriptor, Relief, Module, and Croquis work; neither requests Flow or Rendu.
The two-source case requests the full cross-file analyzer, not the lightweight
project index. Query-count assertions and zero-execution assertions for
unrequested products are enforced by the artifact-graph integration tests.
