# Production stage capture

Decision for #6832, 2026-09-28.

The existing `ladder_run` is an independent L1 → L2 → L3 execution. DOM
production emission takes an optimized L1 → L2 → L4 route, while the accepted
SSR and Vapor bridges execute L1 → L2 → L3 → L4. Pages from `ladder_run` cannot
be presented as the stages that emitted a product module.

L0 owns a target-neutral `CaptureSink` contract and an owned `StageCapture`
sidecar. The ordinary compile uses `NoCapture`; page, timing, remark and
outcome builders are lazy, so they do no dump work or clock reading there.
Each product records only boundaries it actually executed. Provisional pages
are committed when its native emitter returns the selected module; a legacy
selection, no-template result or rejected compile discards them. The sidecar
records the target, effective options and explicit outcome so an empty feed
cannot be mistaken for a native run with no changes.

DOM records its actual surface parse, lowering, fact transform and render
emission. Its production emitter folds preserving facts rather than executing
the separate ladder's pass plan, and does not lower L3. SSR and Vapor record
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
