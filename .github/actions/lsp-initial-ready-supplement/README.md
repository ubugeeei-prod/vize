# Prepared diagnostics: foreground and retained context supplement

This private source-release experiment follows the completed tiny comparison41319724375. It evaluates scheduling QoS and retained resource costs before adoption of PR11325. It does not rerun or replace the original100/100 six-operation experiment, its positive initial-publication result, its warm tail/RSS regressions or any earlier failure.

## Fixed source and executable identity

- B: `51c31c6bfee4d0b6587b644c2aab39c6f3e83a1f`.
- C2: `b52148128716418a08aa33b6d6e1c6e33961db56`, through reviewed C1 `b5aa51bf91e4642b16657a5deeaca311f8b5f250`.
- The new workflow commit has accepted H3 `7c74351cf94507b42221cf52e7037c21a5dc7a0d` as its only parent. H3's source parent is C2. Only this closed supplement directory and the existing `lsp-current-baseline.yml` change.
- Two distinct fresh Cargo targets build `cargo build --release -p vize`; JSON message output is metadata only. Cargo's actual optimized/default/native features and all seven published SDK4 crate artifacts are checked. Profiling features are refused. The first release executable is preserved, hashed, staged and checked before and after every workload. No Rust library executable is presented as that CLI.
- Reused H3 shipping and readiness helpers remain byte-identical. SDK10 declarations, Cargo/lock/source inputs, JavaScript locks, actual provider7.0.2, selected tools, platform and capacity are retained. No native image is uploaded.

## Eight outer outcomes, one physical Linux runner

1. Four trace-OFF timing attempts in B/C/C/B order. Each attempt has six fresh sessions: `huge4096` and `vben2`, each with scheduled first foreground arrivals0/150/650ms after its open sequence. Actual send/receive times and observed source-event positions are retained; the target offset is not a claim that native work entered at that instant.
2. Two separate trace-ON observer attempts, B then C. Both fixtures select `beforePrep` and `duringPrep`; the third Huge row selects `apiWait`, while Vben explicitly selects `collectorWait` because its retained current source route has no API-read marker. The latter observes only the selected worker interval and supplies no API/native CPU attribution. These source-log/caller API-wait observations cannot prove native CPU entry or preemption. Missing, already-closed or same-chunk intervals remain unmatched failures; no retry selects a favorable overlap.
3. Two trace-ON resource/ownership attempts, B then C. One server visits1,4,8 distinct workspace/configuration inputs, closes their documents, then reopens all eight. Fixed milestone PSS and executable/owner facts are untimed. Eight-context latency is observer-affected and is not ordinary shipping latency.

Actual child spawn-event receipt, initialize return, didOpen attempts, notification receive events and foreground request/response clocks remain separate. Constructor/tool-probe start is an additional lower bound, not the spawn instant. First-valid and all-declared-target publication clocks must be derived from matching notification receive events; a later waiter can consume a backlog, so waiter/qualification completion is not substituted for publication time.

The closed schedule has38 sessions:24 timing,12 observer and2 memory. Resolve runs only in the36 timing/observer cases; memory has no resolve and uses version3 reopen. If all planned paths complete, the public protocol has244 semantic requests plus76 initialize/shutdown requests. These are not counts of internal native RPCs.

All eight outcomes are saved, including failures. A semantic failure does not select away other planned attempts. An immutable source/provider/executable custody failure stops dependent work. There is one workflow dispatch, with no retry or retuning for a favorable outcome.

## Fixtures and complete requested-target semantics

`huge4096` has4096 authored flat ledger rows and a separate native invoice member expression. Its closed support file has a4096-property Ledger type. The first and last ledger rows intentionally reference missing members; the full version1 diagnostic vector must include both exact errors. Repair fixes both rows, switches the imported factory from a numeric invoice to a string invoice, and requires the exact empty repaired diagnostic vector plus the changed whole hover. Construction counts are not asserted native parser node counts.

`vben2` uses pinned Vben revision `748b182e440863711dc9624924e92440f5ddc4d3`. Only the offline leaves in web-antd and web-ele are replaced with a controlled imported invoice type probe and a genuine authored assignment error. Original and patched bytes are hashed. Actions hydrates only the exact reviewed Vben gitlink, checks its actual HEAD/clean state and retains its whole tracked-tree receipt. These are two declared/opened/checked targets within the real workspace; its complete file inventory is recorded separately. This does not claim all workspace files have been typechecked. Fixture dependency symlinks and materialized workspaces stay outside uploaded evidence.

Vben foreground hover/completion probe only the first web-antd target; both apps retain complete diagnostic qualification. No first-query latency is claimed for web-ele. The first hover and completion query the call-initialized template MEMBER `totals.invoiceTotal` from a CLOSED imported support file. This excludes local CSS, static-object and bare-identifier structural responses. The entire hover Markdown and authored UTF16 range are independently expected. The complete native completion item, wrapper and opaque data are retained. The public capture seam does not export its owned canonical projection, so initial opaque URI/position authority is explicitly UNAVAILABLE; no filesystem surrogate, wildcard path or learned golden is accepted as independent authority.

Exactly one declared, untimed `completionItem/resolve` is allowed per case AFTER both initial foreground responses and complete version1 diagnostics, BEFORE the version2 edit. It compares the complete issued item with only the independently authored detail/documentation enrichment. All opaque and unknown keys must remain unchanged. It supplies a conservation/type oracle, not timing credit or independent opaque projection authority. The version2 whole hover must change number to string; stale currentness or error responses remain failures. Pending publications, all raw frames and source worker completion are retained. There is no existing exact dropped-background-commit counter: that count remains UNKNOWN rather than a fabricated zero.

## Resource interpretation and caps

Cheap `/proc` RSS/stat sampling uses a nominal50ms interval, with actual gaps/coverage retained. Root, all descendants, process birth identities and legitimate/unknown executable kinds remain distinct. Sampled peaks are not guaranteed continuous true peaks. PSS and executable hashes occur only at fixed untimed milestones; no continuous smaps walk runs inside primary latency windows. Cgroup mount counters, when present, do not establish isolated process RSS and can include cache/kernel charges.

The foreground arm retains the existing Vben guards: root256MiB, process tree512MiB,4 processes and1000ms foreground queries. The separate8-context experiment has explicit safety ceilings of root2048MiB, tree8192MiB and32 processes; these are not replacements for the foreground budgets and do not establish memory improvement. All contexts and all owners remain observed, not merged by provider hash or evicted to meet a budget. Close uses an observed250ms quiet window, not an ACK, quiescence or process-exit theorem; reopen is version3.

Initial runner capacity is at least32 logical CPUs/32GiB physical memory/120GiB free disk, with60GiB before each isolated release build and10GiB before workload execution. The budget allows two retained build trees, two protected first executables and at most8GiB live memory for the context arm. Application processes/workspaces are fresh per timing case; cumulative memory residency is deliberate within its one session. OS page-cache coldness is UNKNOWN. No page-cache purge, GC flag, sleep-based product fix, provider replacement, eager closed-file scan or production cache is introduced.

## Raw evidence and refusal

The workload persists `outcome.json` and all original raw captures before verdicts. Timing/observer outcomes require exactly six unique fixture/arrival rows and six complete closed captures. Memory requires one capture and the ordered1/4/8/closed/reopened8 milestones. The outer controller independently checks source and FIRST image, build receipt, process exit, stream sizes/SHA and actual complete JSON-RPC frame bytes. Missing/partial/corrupt/foreign/symlink evidence refuses success. Source-only artifact limits are32MiB compressed,256MiB expanded,4096 members and32MiB per member and16MiB per stream; observation JSON is at most4MiB. No image, ELF, archive nesting or fixture dependency tree belongs in the upload.

The result can support only this declared source/runner/input closure. It does not establish a universal500ms guarantee, full-workspace diagnostic completion, public-installed latency, complete native temporal attribution, statistical significance or a memory win. Existing normal80 nativeUnsupported classifications and experimental artifact admission remain unchanged.


## Corrected successor after failed H690

Original run41329859987 and all eight failed outcomes/38 closed raw sessions remain immutable. This changed controller corrects authored input and observation contracts: multiline anonymous-property JSDoc attaches under the pinned TypeScript6 parser; each revision imports only its active factory, preserving all real diagnostics rather than filtering unused-import hints. Huge API endpoints join the exact source-produced host-to-request binding and nonempty, equal config/project/snapshot identities. Vben's predeclared collectorWait is not a fallback that invents missing API events. Memory continues all five declared milestones after semantic or owner-join errors, retains full publication vectors, and keeps missing owner provenance as UNJOINED coverage failure. No caps, product sources, provider, full diagnostic/currentness oracle, query schedule or trace policy change. No old partial timing is promoted to qualification.
