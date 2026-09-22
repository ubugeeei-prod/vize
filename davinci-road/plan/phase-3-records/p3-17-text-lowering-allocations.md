# P3-17 — Text lowering allocation ratchet (2026-09-23)

The final-child leaf path avoids building and discarding a temporary
`TextPart` vector. The existing `s1_to_s2_lower_vfor_three_aliases` probe
measures the resulting reduction with its fixture and stage window unchanged:

| Metric                            | Previous budget | Measured budget |
| --------------------------------- | --------------: | --------------: |
| Allocation calls, Linux and macOS |              13 |              12 |
| Peak live allocation bytes, Linux |            1671 |            1511 |
| Peak live allocation bytes, macOS |            1655 |            1495 |

Linux evidence comes from [Check run 35762295465](https://github.com/ubugeeei-prod/vize/actions/runs/35762295465/job/106863895489),
head `2f7a8674a0c8482b23b3d916b68f3e7159062fd0`, artifact
`davinci-allocation-bench-reports` (ID `10710412914`). The required allocation
gate ran `cargo bench -p vize_s1_to_s2 --bench davinci_storage -- --quick`
with the release profile. All five other compact-storage reports passed their
existing budgets; only this beneficial exact-budget change failed.

macOS arm64 evidence was measured independently at
`fce0e21564a3fbad508c9f97a6e710d4b0b89d38`, using the isolated target built for
the text-lowering change:

```sh
cargo bench --profile ci-opt -p vize_s1_to_s2 --bench davinci_storage \
  --target-dir /tmp/vize-dom-emission-target \
  -- s1_to_s2_lower_vfor_three_aliases --quick
```

Both reports use harness version `0.425.1`. The macOS report records
`platform: macos`, `allocs: 12`, and `alloc_bytes_peak: 1495`; these are measured
values, not a Linux-derived estimate. The platform difference remains 16 bytes.

The exact allocation gates and their assertion tests now ratchet down to
these values. Wall time remains report-only, and no tolerance or production
selector changes. This allocation evidence does not close production DOM
admission: the earlier full-SFC reach check still reports 294 byte-exact
projection comparisons with zero divergences and zero production selections.
