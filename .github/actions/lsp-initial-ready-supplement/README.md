# Focused foreground-readiness comparison

Status: private controller source. No run has been dispatched by this packet. Parent `/root/lsp_lead` owns final publication, one dispatch, transport and adoption.

## Question and fixed sources

Measure the incremental same-document foreground preparation notification, rather than attributing the earlier C2 scheduler improvement to this change.

- B: actual #11325 merge `6395f93131f6d4cdb4b429dece08d9b15bbbef4c`, already contains C2.
- C: qualified #11347 head `03ad1ca3e78fc97e8995882bd61942ce6fcbda23`.
- Current functional proof: `../ci-03ad/functional-current-03ad.json`, SHA256 `3bf6bb215f3e128f016b4e1282efc35809fcaee4cde05515c4b56b250429ee38`.
- Current ordinary Actions: all 15 workflows SUCCESS. Check 56 PASS / 13 SKIP. All 37 scheduler tests pass with zero ignored tests; native invalid/repair/recovery coverage is 129/129/129. Native LSP groups 77 and 32 pass without skips.
- B/C differ in exactly 13 complete paths listed in protocol.json. Cargo, lockfile, Rust toolchain, package manifests and provider selection are unchanged. The shipping effect is the original-scope receipt after strict same-URI native preparation; no new provider call, RPC or output filtering.

The existing controller source is historical H0499 `0499c7bcf70222a9f3b21c41ffbb72bf2af20fe7`. Its original 8 FAILED outcomes remain FAILED. The temporary H is a DIRECT CHILD of qualified C03ad, adding/retargeting only the finite existing .github controller files. Whole-byte old controller/helpers may be reused from immutable Git objects, but the H→C inverse must restore the exact C tree/modes. Build both separately pinned role refs/trees; do not transfer old H0499 source/provider/runtime custody. Parent freezes the actual H/parent/pathset and verifies remote source before the one POST.

## Exact sample and workload

One TRACE-OFF timing arm only, serial B1/C2/C3/B4. Each attempt opens one fresh huge4096 session, then one fresh real-Vben2 session. Total: 4 attempts, 8 fresh application sessions, 4 sessions per role, n=2 per fixture/role. No observer, 1/4/8-memory or tiny100/100 arm is repeated. No preparation query is issued before the measured pair.

Reuse the exact H0499 fixture generation and request schedule:

1. Initialize the fresh server with editor/native typecheck enabled. Keep actual child spawn receipt, constructor lower bound and initialize-return clocks distinct.
2. Open v1; arrival=0 means no artificial delay, not an invented equality between didOpen and the request-attempt/pre-send callback. Retain request-attempt/pre-send callback timestamps and open-to-attempt gaps. The following(id) callback is evaluated BEFORE this.send/write; it is a lower bound, not actual stdin-write completion or an OS event. The earlier pair sentNs is a pair-construction lower bound. Raw frame order is retained separately; native CPU and IO intervals remain UNKNOWN.
3. Send the same concurrent first native template MEMBER hover and completion on `totals.invoiceTotal`, initialized by an imported call from a closed support file. Preserve both complete responses, errors and elapsed times before assertions.
4. Receive the complete independently expected v1 diagnostic vector for every declared opened target, with actual notification-receipt clocks; do not substitute a later backlog-awaiter completion time.
5. Keep the already declared, untimed `completionItem/resolve` conservation request after both first responses and full v1 diagnostics. This is 8 planned resolves, not prior execution credit. Whole issued-item equality allows only the independently authored detail and Markdown fields to be added. A new resolve mismatch is retained as an actual failure, without a Markdown-oracle chase or automatic campaign retry. Initial opaque projection authority remains UNAVAILABLE.
6. Apply the exact v2 repair, changing `invoiceTotal` number to string and removing all deliberate errors. Send the same current hover/completion pair, require revision advance, exact complete v2 diagnostic vectors and actual collector-finished evidence. Retain every intermediate publication and refuse unknown/stale packet fields.
7. Close authored targets and shut down; all raw streams and complete framing, process exit, source/build/FIRST identities and RSS evidence are saved before verdict. Each failed case remains a failed observation; continue the fixed remaining attempts where source/image safety permits.

Planned client request roster across 8 sessions: initialize 8, first+repaired hover 16, first+repaired completion 16, untimed resolve 8, shutdown 8, total 56. Preserve and independently account for server-originated requests and all notifications separately. No hidden resolve/query is added.

Huge4096 preserves all 4096 authored typed member expressions, deliberate missingFirst and missingFinal at the first/final nodes, imported call probe and exact full repair. It checks 1 opened SFC plus its closed support dependency. Vben is fixed gitlink `748b182e440863711dc9624924e92440f5ddc4d3`, the same 2 authored target SFCs and exact patches/support. Inventory total workspace files separately from the 2 opened/checked targets; no eager diagnostic scan of closed workspace files.

Across the campaign there are 24 declared target/version diagnostic vectors: 4 attempts × (1 huge + 2 Vben targets) × 2 versions. There are 16 hover responses and 16 completion responses. All fields and opaque nested payloads are retained, with no normalized or learned golden.

## Source-derived hover and unchanged resolve

Apply only the already frozen hover-only separator proposal: Markdown native hover has exactly ONE newline between closing code fence and plain JSDoc. This follows pinned native `hover.go` (ref `2bd066d87f5bafd315be9f40889d0a60b9e58e0b`, blob `da4f11883c0514d35bea4a50d9c7b52c7d6ec21e`, SHA256 `376e51239fac43eb8e9fe92389c1e99d428fa9f414030b4141170c2d5c55271a`) and unchanged product pass-through/range bodies. It is not a trim/normalization operation. Completion-resolve Markdown remains TWO newlines as independently authored in the existing sibling control. Type-only hover, wrong content/type/range/method/request, missing or extra fields still refuse. The 104/104 held hover replay and 104/104 completion replay are historical offline evidence, not new-run acceptance.

## Existing controller deltas required

Keep the same `lsp-current-baseline.yml` workflow_dispatch path and closed source_sha contract, temporary evidence ref, existing staging, same shipping-adapter/readiness-parity helper bodies, same source snapshots and failure-first persistence. Do not modify PR product source for measurement.

- source-custody.ts: exact new 13-path B→C roster and complete role-specific SHA/blob/mode rows from protocol.json. Existing BOUND_PATHS has 50 rows; 49 remain byte-exact. `ide/context.rs` is the sole deliberate changed bound row and must be authenticated by its complete reviewed before/after row, not ignored. The historical B51/C2 SOURCE_PINS/ancestry must not be used to qualify B6395/C03.
- controller.ts: pin B/C above and actual temporary H ancestry/pathset; closed plan contains only four timing attempts. Keep exact default-release build/tool/provider/FIRST custody. No libtest build/retest in the measurement arm.
- workload.ts + source-custody checkOutcome: closed timing arrivals `[0]`, exactly two ordered fixture rows and two complete raw captures per attempt. Existing mode parser need not gain a new mode or flag. Remove observer/memory execution from the fixed controller plan, rather than dynamically skipping failed rows.
- fixtures.ts: only source-proven hover separator in hoverPacket; keep markdown()/resolve TWO and all source/input/diagnostic bytes unchanged. Do not change native-queries.ts completion schema, resource sampler, input count, or production.
- controls/custody-controls: exact plan/count/source/missing/foreign/extra/trace/nonzero-arrival refusal, full hover negative mutants, complete raw-capture checks. All unchanged historical controls remain intact; parent records meaningful current pure/strict/YAML results.
- README/workflow staging: publish exact fixed sample and limits. Hydrate only the pinned Vben gitlink. The helper/control path inventory is closed; preserve all existing framed raw and source-custody checks.

All existing files must remain formatted and ≤500 lines. If a cohesive extraction is unavoidable, enumerate it as a genuine new tooling path and retain body equality/source peer; no generic framework or new workload is authorized by this plan.

## Build, resource and cache law

Use exactly the accepted NORMAL recipe `cargo build --release -p vize --message-format=json-render-diagnostics` (metadata argument only), default opt3/native/no profiling. Two distinct fresh target directories, one per role. Preserve each FIRST CLI image/receipt immediately and restore its exact SHA before every attempt. Protect against a later library build changing the CLI; no library build is planned here. SDK compiler artifacts must contain all seven public `2.0.0-beta.4` crates, unchanged declaration source closure and no profiling features.

Keep selected provider `@typescript/typescript-linux-x64@7.0.2`, exact executable physical identity/ELF magic/full SHA, empty version stderr, version text, and before/after tool/provider identities. Do not infer provider identity from version alone. Existing source/lock/package checks remain exact.

Use the existing 32-vCPU Linux runner, CARGO_BUILD_JOBS=16, RAYON_NUM_THREADS=8, CARGO_INCREMENTAL=0, empty RUSTFLAGS/encoded flags, fixed Node options and ambient override refusal. Preserve actual affinity/cgroup/CPU/memory/cache facts; Rayon8 is not a whole native-process CPU limit. Fresh application processes are app-cold; OS page cache is UNKNOWN, no purge. No profiler or preparation trace in primary timing.

Original foreground safety/query limits remain root256MiB, tree512MiB, process4, request1000ms. Cheap RSS sampling stays 50ms with actual gaps/missed/failed samples retained; known-owned denial is incomplete/failure, not zero. Keep per-row root plus all descendants, PID birth and same-sample tree peaks; no RSS=PSS or PID=config inference. Fixed untimed post-completion identities may be retained. No timed PSS collection or new 8-context cap.

Keep raw stream16MiB, observation4MiB, events16MiB, source member32MiB, ZIP32MiB, expanded256MiB and 4096-member caps. No full target or ELF archive. Forecast: 2 default-release builds plus 8 sessions is smaller in measured session count than held0499's 38 sessions/29MB selected data; this is not a guaranteed linear byte/time forecast. Individual caps do not override aggregate caps. Retain the existing pre-build120GiB/role-build60GiB/attempt10GiB free-space safety checks and ≥32GiB/32CPU authority before work. The 180-minute workflow timeout remains a terminal safety bound, not an acceptance latency budget.

## Measurements and decision

For each fixture/role show BOTH samples and B/C/C/B order. Report per-request first/repaired hover/completion, spawn→initialize-ready, open→first exact native diagnostic receipt, and open→all declared v1 diagnostics. Keep full v2 repair/end-collector clocks, actual request-attempt offsets, response/error/cancellation/currentness facts and all raw RSS rows. With n=2, p50 is the lower sample and p95/p99 are the maximum; disclose n and avoid broad statistical claims. Show worst ratios/outliers and mixed-sign tradeoffs, including prior C2 warm tails/RSS negatives.

Primary question: does the same successful foreground preparation avoid the residual initial Open grace while retaining exact query, diagnostic, currentness, process and resource laws? A correct faster diagnostic alone is not proof of a native CPU overlap interval, whole workspace completion, memory improvement, 500ms across scales or 10x. Missing dedicated dropped/retry accounting stays UNKNOWN. Terminal source/image/guard failure blocks dependent dispatch or adoption; semantic failures retain all observations and do not authorize blind rerun or oracle relaxation. Parent/root make a bounded adoption/normal-Q/M decision from useful measured evidence; current Draft green status is not completion.
