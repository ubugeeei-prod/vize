# Fresh mounted-runtime devtools observation

Issue: [#6863](https://github.com/ubugeeei-prod/vize/issues/6863).

The immutable bundle experiment retained all 162 fresh-process comparisons but
regressed wall time. On [the first controlled run](https://github.com/ubugeeei-prod/vize/actions/runs/36852084634),
development disabled/cold/warm runs took 90.757/94.753/94.323 seconds, and production
took 9.679/13.336/13.120 seconds. Complete input fingerprinting cost approximately
7.5 seconds per warm run, exceeding the 4.3–4.5 seconds of eliminated bundling.
The cache provider and adoption are rejected from the merge path; their published
heads and negative experiment remain available in
[Stack #7345](https://github.com/ubugeeei-prod/vize/pull/7344).

The unchanged [input-cost comparison](https://github.com/ubugeeei-prod/vize/actions/runs/36862831801)
also passed all 162 executions and confirmed the regression. Its development
disabled/cold/warm runs took 90.785/94.044/93.931 seconds; production took
9.358/12.793/12.100 seconds. Every cached execution covered 6577 actual inputs.

| Warm runtime | Inventory | Complete byte hashing | ABI      | Total fingerprinting |
| ------------ | --------- | --------------------- | -------- | -------------------- |
| Development  | 1.627 s   | 5.752 s               | 0.0365 s | 7.417 s              |
| Production   | 1.586 s   | 5.320 s               | 0.0360 s | 6.943 s              |

Uncached bundling totaled only 4.232/4.436 seconds. Hashing dominates the retained
identity cost; ABI work is negligible. Metadata-only fingerprint reuse is
rejected. Both [#7343](https://github.com/ubugeeei-prod/vize/pull/7343) and
[#7344](https://github.com/ubugeeei-prod/vize/pull/7344) are closed without merging,
and this independent replacement has no cache provider or adoption dependency.
The paired [#6863 rejection](https://github.com/ubugeeei-prod/vize/issues/6863#issuecomment-5931783710)
records the same measured decision.

## Decision

The pinned [Vue rc.9 devtools hook](https://github.com/vuejs/core/blob/v3.6.0-rc.9/packages/runtime-core/src/devtools.ts)
waits 3000 milliseconds for late devtools installation when a browser-shaped
environment has no hook. The pinned [Vapor app preparation](https://github.com/vuejs/core/blob/v3.6.0-rc.9/packages/runtime-vapor/src/apiCreateApp.ts)
selects this hook in development. Our fresh HappyDOM process meets that condition,
and Rust waits for its complete exit after reading the trace. The measured
development/production difference is consistent with 27 such waits; this is a
source-backed attribution to verify with an actual controlled comparison.

Install a fresh supported devtools observer before loading each development
runtime. The paired [#6863 observer decision](https://github.com/ubugeeei-prod/vize/issues/6863#issuecomment-5931848660)
records this same slice. It records lifecycle events and supports `emit`, `on`, `once`, `off`,
`appRecords` and `cleanupBuffer`. Every successful development trace must observe
its exact app initialization and unmount once and release all app records. Each
trace disposes its own observer; existing hooks and non-writable environments keep
their existing behavior. Production keeps its original devtools-disabled runtime.
The observer scope includes asynchronous runtime/render and app/scope setup. A
setup failure closes its DOM and disposes its hook before rethrowing the identical
original error, even if DOM cleanup also fails.

All runtime builds, processes, DOMs, modules, scheduler state, diagnostics, render
comparisons, transition callbacks, full snapshots and goldens remain fresh and
unchanged. There is no forced process exit, timer interception, shortened timeout,
Vue implementation patch, DEV flag change, or cached runtime state.

## Proof and remaining work

The dedicated Actions comparison runs the unchanged nine-scenario, three-backend
Transition test in development and production, with the observer disabled and
enabled at the same source. It retains all 108 distinct subprocesses, complete
parity logs, diagnostics, per-trace hashes, devtools lifecycle receipts and wall
time. Every complete trace hash must match the corresponding baseline; all
original native/retained/official goldens must pass. Production must continue to
emit no devtools lifecycle events. Observer-off mode exists for this controlled
baseline, rather than as a shorter required test path.

The five observer laws cover failed asynchronous setup and cleanup, event listeners and fresh ownership, missing or
wrong lifecycle evidence, production/baseline/external-hook behavior, and exact
descriptor restoration. Actual rc.9 timing and runtime semantics are validated
in Actions rather than against the older local oracle installation.
All existing mounted-runtime tests must also pass in the full protected suite.

The source and lifecycle laws need review before publication. The comparison must
show a measured net gain before entering the protected queue; full queue suites
and all existing required contexts remain mandatory. The overall two-minute PR
target remains unfinished even if this isolated mounted-runtime cost improves.
