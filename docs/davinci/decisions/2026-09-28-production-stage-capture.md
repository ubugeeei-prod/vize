# Production stage capture

Decision for #6832, 2026-09-28.

The existing `ladder_run` is an independent L1 → L2 → L3 execution. DOM
production emission takes an optimized L1 → L2 → L4 route, while the accepted
SSR and Vapor bridges execute L1 → L2 → L3 → L4. Pages from `ladder_run` cannot
be presented as the stages that emitted a product module.

L0 owns a target-neutral `CaptureSink` contract and an owned `StageCapture`
sidecar. The ordinary compile uses `NoCapture`; page, timing, remark and
outcome builders are lazy, so they do no dump work or clock reading there.
The ordinary DOM host retains its original `try_emit_l2` and `emit_l2` bodies,
and the L1→L2 emitter retains its original parse, lower, transform and emit
body. A compile-time `CaptureSink::RECORDING` switch sends `NoCapture` through
those bodies and an observing sink through the same-run page path. This was
required by the protected instruction gate: candidate `8f364d` against the
green predecessor `cf8086` changed only the six full DOM compile probes
(+162, +779, +772, +1251, +23 and +2121 instructions); the DOM transform,
codegen and other product probes stayed unchanged. The captured generic body
changed LLVM's optimization of the ordinary DOM emitter despite lazy page
builders. Restoring the L1→L2 body alone left only the small (+28) and wide
(+48) probes above their ceilings; Callgrind showed the remaining ordinary
host calls still passed through the generic captured wrappers. No instruction
ceiling is raised. After restoring the host's direct ordinary calls, a
fresh-main diagnostic `f707e07c9` left four full DOM compile probes exactly
three instructions above their ceilings; all other 96 passed. Callgrind
showed identical ordinary host and L1→L2 emitter calls, with the remaining
three instructions at the benchmark closure's call to the generic DOM root
with a `NoCapture` argument. The ordinary DOM root now keeps its original
signature and direct compile body; the opt-in captured root is a sibling.
The next exact-candidate instruction run must verify all 100 probes before
the PR is updated.
Each product records only boundaries it actually executed. Provisional pages
are committed only after final product selection, including DOM source-map
parity. A compatibility map mismatch returns the compatibility module and
discards native pages; selection accounting reports that final lane. A legacy
selection, no-template result or rejected compile likewise discards pages. The sidecar
records the target, effective options and explicit outcome so an empty feed
cannot be mistaken for a native run with no changes.
`timings_observed` and `remarks_observed` are false until a producer observes
the entire respective channel; an empty unobserved channel is unavailable,
not a claim that zero events occurred.

DOM records its actual surface parse, lowering, resulting fact tables and render
emission. These facts describe the current transitional DOM emitter; static
and hoist decisions remain L3 target ownership. The production emitter folds
preserving facts rather than executing the separate ladder's pass plan, and
does not lower L3. SSR and Vapor record
the L3 artifact that their accepted backends consume. L4 pages represent the
documented backend emission boundary; later SFC script assembly is a distinct
host step. The current SFC adapter must carry the capture from the compile
that produced its returned module, including explicit fallback status. It
must not infer selection from global profiler counters or rerun a ladder.
Compatibility parser work that produces the existing public AST/diagnostics is
outside these native module-producing pages and remains separately reported.

This adds no replacement route for legacy compiler behavior. #6880 still
gates any such replacement. The CLI and playground consume this sidecar in
the next #6832 change; their old independent ladder feed is not production
evidence. `vize_l0::dump::capture` moves with the L0 substrate in #6833;
native SFC block splitting and the full tokenizer/dialect moves retain their
own issue boundaries.
