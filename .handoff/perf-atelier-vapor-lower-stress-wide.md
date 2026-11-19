# Handoff: perf/atelier-vapor-lower-stress-wide (PR #13344, closed; 2026-11-19)

Remove this `.handoff/` file before anything from this branch merges.

PR #13344 is closed, not merged, because no shape passes every bar yet. The
branch head before this note is ab16d515b2 (v5). The SSR half now lives on
`perf/atelier-ssr-slot-name`; see its own handoff.

## Measurement setup

- Scratch driver: branch `perf/pb-driver`, workflow file `level-instruction-counts.yml`.
  - Dispatch: `gh workflow run level-instruction-counts.yml --ref perf/pb-driver -f release=true -f roles='["name=<full sha>",...]'`.
  - Each role builds ci-opt exactly like pr-performance-observation, plus an optional release (fat LTO) probe.
  - Each role then runs `instruction-counts.mjs --collect` (stages, formatter, sfc-parsing).
- Diff the artifacts with the scratchpad tools `sdiff.py` and `cmp.sh` (path in the session report).
- Base: 9c7d4cba55. Its Rust sources are identical to later mains up to dc783944a3. Base numbers are in run 44468615200.

## Vapor (`write_attributes` length mask)

The target row is `atelier_vapor_lower_stress-wide`: 103,460 on main. Every mask shape drops it by 4.7–5.8%. But every shape also shifts the `vize_atelier_vapor` ci-opt codegen-unit partition (thin LTO, 16 CGUs). The shift moves unrelated generate rows up.

| Variant (commit on 9c7d4) | lower_stress-wide | Rows up |
| --- | ---: | --- |
| va5 = v5, mask from a props scan (356dd54d12) | −4,931 | generate/compile_stress-wide +300, generate_large +44, lower_large +56, compile_large +100 |
| va9, mask helper `#[inline(never)]` (cf5f2ae941) | −4,826 | same, plus lower_large +95, compile_large +139 |
| va10, plain helper (32eaa58ee9) | −4,931 | same as va5 |
| va11, mask from the bound-name set (58c9409943) | −6,006 | generate/compile_stress-wide +300, generate_large +44 (3 rows) |
| va16, `fold` (b14910367b) | −5,969 | 13 rows up, up to +2.3% |

- **x86 partition probe.** `xmono.sh` builds `cargo rustc --target x86_64-unknown-linux-gnu -Zprint-mono-items`, using a downloaded rust-std sysroot and `/usr/bin/true` as linker. `cgu2.py` compares root-item groupings.
  - va5, va9, va10, va11, va12, va13, va15 and va17 all produce the same shifted partition (4 CGUs split vs main): `generate::operations::generate_operation` separates from its module mates, which costs about +3 instructions per call.
  - va16, va18, va19, va21 and va22 produce a second, worse partition.
  - No shape tried reproduces main's partition.
- **Next ideas:**
  - Find which callee `generate_operation` stops reaching locally. Use a callgrind function diff of `atelier_vapor_generate_stress-wide`, base vs va11, then make that callee partition-independent (e.g. `#[inline]`, so it is instantiated in every CGU).
  - Alternatively, move the probe change to a module whose size estimate does not reorder the merge.
- Output byte-identity was verified for v1 only. Redo it for the final shape.

## Release-size rule (maintainer, 2026-11-19)

Size is judged on the shipped release profile (fat LTO, 1 CGU), which must not grow. A ci-opt-only shift is not a regression. va16's release is +512 B; the other vapor variants were not release-probed.
