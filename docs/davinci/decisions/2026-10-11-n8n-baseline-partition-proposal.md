# Fixed n8n baseline input partitions (#6830)

This is an inert source proposal. The ordinary source campaign still defaults
to its original serial observer. No partitioned native campaign, paired speedup
or gate-wide gain has been measured for this change.

The retained normal PR Check 38066980020 at source 0f5a/tested merge 459e observes
1017s for Test JS packages: 264334ms for the original JS suite and 727418ms for
project transport. Baseline preparation consumes 334008ms; its two unchanged
51-rule per-rule modes total 288979ms. The complete source/receipt attribution is
retained in ordinary-pr-js-1017s-source-attribution.json, SHA d5cf5ecac203ee09.

The proposal partitions the original ordered 1369 inputs into fixed contiguous
ranges 685 and 684. Each input runs all six original modes sequentially with the
same settings, options, full ordered diagnostics, cold-call and zero-warm-call
laws. The original revision/options/LRU/collected-cache controls run once in the
last partition. No product, cache, native logger, controller, pinned host worker,
licensed input or 51-rule configuration is changed.

Source preseed and both real process groups share one 900000ms deadline and one combined 32MiB
stdout/stderr budget. Fatal exit, signal, spawn error, output overflow, or parent
interruption ends the campaign, kills every started owned group, and waits for
close. A normal direct-child close also drains ignored-pipe descendants before
completion, so a later sibling failure cannot leave its former group alive.
The parent saves and verifies complete licensed hash backing before either child
starts, avoiding a duplicate-text partial-write race. This is source custody,
with its overhead included in both the partitioned arm timing and deadline; native startup hashing is
unchanged. Only the final worker creates cache-control backing additions.
Complete per-worker byte streams, terminal fields, native arguments/results,
source bytes and vectors remain separate. The join rejects missing, duplicate,
reordered or substituted input/mode rows and preserves their entire payloads.
The existing aggregate native verifier still checks every PID/load/call, source
hash, activation/options/result and per-file call count.

Before any native execution or ordinary default change, a whole semantic peer
must approve the frozen source and the same-source pair. The proposed pair uses
one immutable checkout/native build/toolchain/corpus and keeps host mode
concurrent for BOTH invocations. Only VIZE_N8N_BASELINE_MODE is serial versus
partitioned. Each invocation gets a fresh staging root and retains every original
phase/control/native proof. Qualification requires complete successful campaigns,
whole vectors/options/cache fields equal (elapsedMs retained but excluded from
identity), and all native source/activation/result and control laws. Compare
baseline phase walltime separately from project and total JS gate walltime.
Do not use historical source 0f5a timing as a substitute for the new serial arm.

The whole original 289-line observer and 311-line qualification reconstruct exactly
by removing the selector, control guard and optional caller branch, then reversing
two formatter-only line wraps. The
serial branch keeps the original vp argv, environment, timeout, maxBuffer, raw
capture and assertions. Existing two concurrent project hosts remain unchanged.

The finite eight-path source peer and ten inert process/source controls pass.
The earlier ignored-pipe descendant control is retained RED against the old
helper, and the malformed controlled CRLF child source is retained RED before
its string-serialization repair. Neither control executed product/native inputs.

The manual-only `component=n8n-baseline` dispatch uses
`.github/workflows/ci-walltime-benchmark.yml`. It checks out the run's immutable
`github.sha`, uses the original JS setup, builds the existing debug native once,
and packs the bridge once. It then runs the unchanged complete controller twice
with concurrent project hosts, changing only the explicit baseline mode and
owned timing filename. The second arm runs after a first-arm failure unless
cancelled; either failure keeps the job failed. The always uploader retains both
staging trees, whole logs and the one native build receipt/process. Original four
comparison jobs, default `all`, required JS suite and ordinary defaults are intact.

TODO: whole manual workflow peer and exact-source Actions admission; same-source
Actions pair; report measured result; only then decide whether the ordinary
baseline default should change.
The inherited unfinished Docs comparison and remaining CI acceptance stay open.
