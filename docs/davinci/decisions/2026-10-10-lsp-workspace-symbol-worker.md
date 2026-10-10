# Reuse the native workspace-symbol worker

Related: [#3952](https://github.com/ubugeeei-prod/vize/issues/3952).

Two independent full Check jobs retain the same Misskey lifecycle failure:
[38038369897 / 114173487502](https://github.com/ubugeeei-prod/vize/actions/runs/38038369897/job/114173487502)
at source `c70dd1f1c69ec5f9cdc1075c19413dd36af69829` samples 131336 KiB,
and [38033465773 / 114158996019](https://github.com/ubugeeei-prod/vize/actions/runs/38033465773/job/114158996019)
at source `156dfb1db81dff4c5665681d9dbe4f9b29351b61` samples 131768 KiB.
Both exceed the original 128 MiB server ceiling during file lifecycle, after
40 edit cycles remained below 110 MiB. They use the same pinned Misskey
revision, 583 Vue / 888 Vue-and-TypeScript files, 287 publishes, and scale 1.
All samples have three processes and the tree peak stays below 384 MiB.
The [immutable before fixture](../../../tests/_fixtures/differential/lsp/workspace-symbol-worker-rss-3952/README.md)
retains complete metrics, job logs, actual producer source and every digest.
These records do not establish a true peak, independently authenticated binary
identity, a retained-document leak, or a causal allocator explanation.

The existing #8277 producer already releases every closed-source buffer before
reading the next. Its request still starts a new native thread for every symbol
scan. Reuse that same blocking execution boundary within its owning inventory:
one lazy worker, one bounded async sender, at most one buffered command beside
the running command, and an owned one-shot result for each request. Waiting
for capacity yields the LSP executor. Dropping an inventory closes its sender;
no join blocks the editor. A failed worker returns the existing open-buffer-only
fallback, and a later request can start a replacement. A cancelled receiver
releases its result and input ownership when that actual command finishes.

No parser, native AST/arena, source text, symbol result or response is cached in
the worker. Keep the existing streamed collector, full lexical source order,
open-buffer authority, retired namespaces, root/source-set revision retries,
complete ranking and final 100-result cap. Reference discovery/read and cold
path discovery keep their original one-shot worker scheduling byte for byte.
The producer footprint is the existing inventory field/module and symbol call
boundary plus the new worker and seven resource laws. It adds no pipeline stage,
level serialization, provider substitution or budget change.

The seven resource laws verify lazy creation, distinct inventory owners, reuse
across 40 sequential and overlapping commands, one running command, running,
queued and backpressured cancellation ownership, actual native thread-local
teardown after dropping an idle inventory, and worker failure/restart. An
explicit lost-reply control holds the old connection's commands, starts a
genuine replacement worker, then drops a delayed old command and requires the
next request to reuse that replacement's thread.

The concurrent six-law native boundary host exposed a genuine restart race:
the failed command drops its result sender before its native receiver finishes
unwinding, so the next request can reach the dying worker. Retire a connection
on result-channel loss using its owned `Arc` identity. A delayed old failure
cannot clear a newer replacement, and the bounded sender is never cloned.
The [race archive](../../../tests/_fixtures/differential/lsp/workspace-symbol-worker-rss-3952/worker-race-manifest.json)
retains the complete authentic failure, original and revised producer bytes,
all local logs and the explicit field-owner host. Seven laws pass in that host;
it supplies no whole-product or genuine Inventory qualification. Independent
review corrected its initial model's wrong source type name before publication.
The authored law now uses the actual `Inventory`, which must pass in Maestro
Actions with the original symbol, mutation and reference controls.

Historical source `c3b847fa86e6f50c619e735cc4fe0c3dbb812662` passes its
PR source/native checks and the original
[Misskey job 114179488929](https://github.com/ubugeeei-prod/vize/actions/runs/38040456635/job/114179488929).
It samples a 125388 KiB server peak (122.45 MiB, below the unchanged 128 MiB cap)
and 349648 KiB tree peak (341.45 MiB, below 384 MiB but higher than both before
tree peaks). All 287 retained publish records match both original before runs
in arrival order after changing only their temporary outer fixture URI prefix.
These records contain sorted normalized diagnostic code/message/range/severity/
source fingerprints, not full raw LSP packets or intra-array order/tags/data.
The source-build-required flag was absent for this churn session; its PID 81040
is not independently bound to the retained hashed source-build executable.
The [17-file historical witness](../../../tests/_fixtures/differential/lsp/workspace-symbol-worker-rss-3952/after-manifest.json)
retains complete metrics, log, comparisons, build receipt and literal source.
Its eight workload/harness/registry files match both before commits byte for
byte. This observation grants no revised-worker performance, allocator, leak,
true-peak, overall-memory or 10x claim.

Independent review admits only the two new `workspace_project_files/worker.rs`
and `worker/tests.rs` paths to the existing original400 producer footprint.
The inventory/symbol paths were already admitted. Preserve every original400
input and whole oracle, source/binary custody, shipping release recipe,
measurement window, process count and budget. Reproduce the paired-original
400 gate from this PR's exact frozen source and actual common ancestor; dispatch
`lsp-current-baseline.yml` with that same exact reviewed `source_sha` for three
fresh serial sessions. This finite admission changes no helper behavior and
authorizes no other source, workload or budget waiver.

TODO: qualify the fresh composed head and seven genuine Maestro laws on
source/full Check, its complete original400 pair and the frozen-source
three-session current baseline. Existing complete materialized-symbol, ordering,
mutation, reference and failure controls remain unchanged and required.

Frozen source `ed9c575a1fa266b17eef5771de216ca2d5dcb81a` completes the
[original400 pair](https://github.com/ubugeeei-prod/vize/actions/runs/38043082997)
against its actual common ancestor `65ab25da5b491cef946e304355de5346b7621208`.
Both use the identical locked runtime and shipping `cargo build --release -p
vize` recipe. All 79 complete comparable request rows match; all 81 whole
responses match after only their captured native session-root spellings change,
and all 18 whole notifications match exactly. Preserve both complete raw streams,
source/binary receipts, original400 Vue / 134 TypeScript inputs, cold requests,
invalidation controls and 20 warm calls. One pair's four warm mean ratios are
1.0318–1.0421 after/before; this grants no speedup or 10x claim.

The same source's [finite three-session baseline](https://github.com/ubugeeei-prod/vize/actions/runs/38043726535)
passes every original 40-cycle semantic and budget law, with one additional
completion call per cycle and bounded raw/RSS instrumentation. Its sampled
server maxima are 114984 / 112876 / 113996 KiB and tree maxima are
338360 / 337304 / 333520 KiB, below the unchanged 128 / 384 MiB ceilings.
Each session observes at most three processes. Every sampled server PID,
birth identity and executable digest joins the actual source build; the exact
Corsa 7.0.2 executable is observed too. The retained whole-packet audit passes
again without executing a provider. This instrumented workload is separate
from the sparse historical `c3` witness. P95 completion transport latency is
5.89 ms; leaf broken/repaired latencies are 59.68 / 53.89 ms. Compatible shared
refresh is 95.27 / 88.09 ms, with causal application unproven.

All seven genuine Maestro laws pass on actual PR projection `ea946a5e`, whose
four owned production files equal `ed9c` byte for byte. This closes the model
host gap, including actual Inventory thread teardown and late reply retirement.
The [13-member immutable qualification archive](../../../tests/_fixtures/differential/lsp/workspace-symbol-worker-rss-3952/qualified-ed9c-manifest.json)
retains both whole official artifacts, literal source, reviews and raw Rust jobs.
Its observations belong to `ed9c`, with no true-peak/PSS, allocator/leak,
overall-memory, cold-cache/startup-tail, GUI/public/default/history or 10x claim.

The fresh n8n workflow requires an acceptance helper genuinely merged after
the original `65ab` baseline; `ed9c` therefore retains its actual
`MODULE_NOT_FOUND` failure. Incorporate actual signed main `9c9ffb8b`, containing
the merged helper, without copying or skipping it. Preserve every incoming
decision and all reviewed worker bytes, and correct only the stale four-law
description. Fresh composed-head source/full checks, original400 and current
resource qualification remain required. Broader distinct-provider, 134-project,
installed/public and #3952 completion remain unfinished. Protected queue,
actual merge and subsequent release remain required.
