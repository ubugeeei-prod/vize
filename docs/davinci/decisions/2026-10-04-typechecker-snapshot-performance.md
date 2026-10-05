# Typechecker source snapshot performance (2026-10-04)

Tracking: [#7698](https://github.com/ubugeeei-prod/vize/issues/7698).

The maintainer's target is 10x faster type checking before Vue Fes Japan,
with authored diagnostic fidelity retained. The full-command baseline is
`da66dc241c` from [Check Benchmark Gate 37169051230](https://github.com/ubugeeei-prod/vize/actions/runs/37169051230):
500 unique SFCs, 2,050,350 bytes, `blacksmith-32vcpu-ubuntu-2404`, identical
`ci-opt` release/thin-LTO/16-codegen-unit settings, two warmups and seven
fresh-process samples. The artifact pins both executable hashes and runtime
versions and requires script, template-prop, template-event, component-prop
and full-corpus diagnostic plants before timing.

Vize max cold startup is 423.1 ms and warmed median is 425.5 ms; 1T median is
1.34 s. The max-lane 10x target is therefore 42.55 ms under this same protocol.
Warm means warmed filesystem/process launches, not reuse of a running type
checker session. These are Vize-before/Vize-after targets; cross-engine
comparisons do not establish this goal.

## First slice: immutable dependency module facts

Scoped type worlds currently parse the same shared dependency module for
every root publication, although `TypeSourceSnapshot` already captures
dependency bytes and resolutions once per revision. Cache only unresolved
dependency module facts within that existing immutable snapshot. Clone those
facts before resolving module targets in each world. The current caller's
in-memory root, including combined normal/setup scripts, always parses anew.
New source snapshots refresh disk changes and editor overlays. Existing
module bounds, cycles, syntax selection and compatibility collection stay
unchanged; no level, pipeline stage or public API is added.

This slice benefits production consumers that reuse one snapshot across
public type-world queries, such as editor publications. The current batch CLI
creates a separate snapshot per SFC, so this slice alone does not establish a
full-command speedup.

The paired Actions probe uses identical public API source at exact common
ancestor/head SHAs and a common fixture tree. It measures cold one-host and
warmed 128-host publications in one/four threads for local-only, 32-module
fanout, deep barrels and Vue/TSX dependencies. Every timing pair must have the
same hash of all canonical module facts and the same host/module counts.
Two untimed process warmups precede nine alternating pairs. Raw samples,
source fixtures, exact SHAs and complete fact signatures are archived.

[Paired run 37170041452](https://github.com/ubugeeei-prod/vize/actions/runs/37170041452)
passed at base `da66dc241c` and head `bf09fd2838`; all 12 workload fact
signatures matched. Warmed 128-host medians were:

| Workload     | Threads |      Base |      Head | Speedup |
| ------------ | ------: | --------: | --------: | ------: |
| Fanout       |       1 | 177.05 ms |  79.12 ms |   2.24x |
| Deep barrels |       1 | 148.99 ms |  41.54 ms |   3.59x |
| Vue/TSX      |       1 | 255.74 ms | 132.74 ms |   1.93x |
| Fanout       |       4 |  63.66 ms |  44.34 ms |   1.44x |
| Deep barrels |       4 |  39.30 ms |  21.43 ms |   1.83x |
| Vue/TSX      |       4 | 109.03 ms | 108.31 ms |   1.01x |

Cold one-host dependency cases cost 0.26–1.07 ms more because the first
publication retains and clones dependency facts (fanout 1.61→1.87 ms,
barrels 1.69→2.76 ms, Vue/TSX 2.22→3.12 ms). Local-only timing differences
were under 0.04 ms. This is a measured tradeoff for snapshot reuse; real batch
adoption must demonstrate its full-command benefit. No 10x claim follows.

## Remaining work

- Adopt shared snapshots in full batch checking with both public compatibility
  collectors and their bounds unchanged. Since `389fd87223`, the private merge
  always selects scoped-world props; its flat-map fallback is unreachable.
  Removing that dead collection avoids any change to public collector caps.
- Measure first-use lock contention and unique-module workloads before
  broadening shared snapshot lifetime.
- Capture full-command paired before/after phases for the pinned generated
  corpus and import-heavy real projects, then persistent warm no-op, leaf and
  shared dependency edits.
- Keep #7698 open until the full-command 10x target is demonstrated. A
  stage-only gain or skipped correctness lane does not complete it.
