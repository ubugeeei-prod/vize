# Streamed workspace symbol sources

Related: [#3952](https://github.com/ubugeeei-prod/vize/issues/3952).
The [paired issue decision](https://github.com/ubugeeei-prod/vize/issues/3952#issuecomment-6053225769)
records the same bounded scope and remaining qualification.

The immutable current-source Misskey campaign at
`d0d0c7db0db537c3cbc6735aa0d8f25296e26166` retained all three failed sessions.
Session 2's original churn oracle measured 132032 KiB (128.94 MiB) at
`file-lifecycle-1`, above its unchanged 128 MiB ceiling; the preceding closed
sample was 102084 KiB. Its independent 50ms-target sampler measured a maximum
of 129620 KiB at monotonic `144832525995`. That observation falls inside the
post-delete `workspace/symbol` request 46 (`144717467293` to `145111316...`).
The two samplers have different observations; the smaller independent sample
does not supersede the original failure. Neither is a true peak or PSS.

The original symbol route loads all workspace source text before collecting
symbols. The retained fixture manifest contains 926 eligible Vue/script sources
(3942891 bytes), whereas its workload metric counts 888 sources under `src/`.
The complete source vector survives while each SFC/script is parsed. Subsequent
lifecycle samples fall below the ceiling; this evidence identifies bulk scan
allocation, not a proved retained-document leak or measured optimization result.

Reuse the existing paths-only inventory for symbol requests. On the existing
background worker, read each closed source, collect its original symbols and
release that text before reading the next. Keep all open buffers as one stable
source-set snapshot; they override disk and retired namespace entries, including
new unsaved files and the existing outside-root open-buffer surface. Union and
sort URIs in the original lexical order, then apply the unchanged symbol ranking
and 100-result cap once after collecting the entire surface.

Preserve existing root, directory exclusion, incomplete-discovery and unreadable
source rules, including the original complete open-buffer-only answer when a
worker cannot start or its channel fails. Open snapshot sharing adds no second
text copy. Partial walks return available sources without caching the partial
inventory. Membership/root generation changes and any open/change/close/rename
source-set mutation during the worker scan force a fresh complete attempt.
No document shard guard crosses parsing or an await. Closed text is reread on
request; no source-text or symbol-result cache is added. Reference queries retain
the existing whole-source API.

The corpus retains unchanged original Vue and authored Unicode/CRLF script
controls and complete independently authored empty-query symbol objects.
Regressions compare every object with the original materialized consumer,
exercise all 120 tied symbols before the original 100-result truncation, assert
one live closed-source buffer, and cover roots, exclusions, failed walks,
retirement/recreation, dirty/outside-root buffers and in-flight project and
document mutations. Existing production workspace-symbol and file-lifecycle
oracles remain required.

The [paired source-methodology decision](https://github.com/ubugeeei-prod/vize/issues/3952#issuecomment-6053506988)
records the root owner's explicit authorization to add only the seven changed Rust paths
from this producer's actual call chain and regressions to the paired source
methodology: the existing IDE collector and extracted source collector, server
symbol route, project inventory and extracted path/stream helpers, and stream
tests. This permits the actual reviewed producer delta to run the original
400-provider qualification; it changes no request, input digest, complete
response comparison, source/lock custody, observed output, timeout or budget.
No directory wildcard, unrelated production path or harness-only shortcut is
admitted. Independent review verifies the exact seven-path footprint and unchanged original
inputs/oracles; the generated source receipt names this admitted producer. Fresh
exact-source qualification precedes any benchmark credit.

The first exact-source build rejected a test-only wildcard `unreachable!()`;
an exhaustive mutation-case enum preserves all seven race cases and assertions.
Its separate native bare-script law timed out waiting for an LSP response after
`lib.dom.d.ts` initial diagnostics. That failed observation remains retained;
this producer decision supplies no success or unrelated-failure claim for it.

TODO: qualify this exact source on Actions and the protected queue; the root
owner coordinates any separately authorized current-source campaign. This source
change does not measure an RSS improvement, close #3952 or #6883, qualify all
134 projects, change the native provider route, or complete release acceptance.
All existing budgets, source/semantic oracles and baseline session counts remain
unchanged. No baseline dispatch is authorized by this decision.
