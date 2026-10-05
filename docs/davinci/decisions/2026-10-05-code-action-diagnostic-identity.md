# Select template code actions by diagnostic identity

P0 [#8000](https://github.com/ubugeeei-prod/vize/issues/8000) reports two
warnings on the exact same authored `<img>` range. The action handler discarded
`context.diagnostics`, and its template pass constructed the default linter
instead of the project's opinionated configuration. Consequently the light bulb
for `vue/html-self-closing` returned `a11y/alt-text`'s suppression, and neither
action identified the diagnostic it resolves.

The existing configured Patina constructor is extracted into a bounded private
module without changing published diagnostics. One existing template lint pass
uses that constructor. For a nonempty client context, each fix or suppression
must match a current template diagnostic's string rule code, exact authored
UTF-16 range and `vize/lint` source. It attaches that complete client diagnostic
in `CodeAction.diagnostics`; duplicate context entries do not duplicate actions.
The requested range and requested-kind filter still apply. No new parse,
provider, cache, serialization or pipeline stage is introduced.

An empty diagnostic context retains the existing cursor-range discovery
contract and response envelopes. Auto-import, ref operations, JSX actions and
native TypeScript fixes retain their existing paths. The request's URI selects
the resident document, and the existing revision guard rejects a buffer change
during a request. Diagnostic objects have no URI or client-version field:
current rule recomputation rejects repaired or displaced old ranges, without
inventing provenance or claiming arbitrary stale-version identification.

The corpus retains the report's complete 47-byte App.vue, 103-byte configuration
and authenticated issue body. Its `.vue.txt` carrier is materialized as the
original `src/App.vue` only by the source-bound CLI/RPC oracle; it does not add an
unrelated compiler census input. Frozen complete JSON expectations are authored
from the rule messages and existing diagnostic/output contracts, not copied from
new observed output.

Three Rust laws check complete same-span actions, full diagnostic payloads,
reverse/duplicate contexts, foreign/numeric/missing identities, repaired buffers
and disabled configuration. The source-built default CLI and whole-wire LSP
oracle retain original lint JSON, both individual rules, combined requests,
kind/range/URI refusals, exact suppression effects, open/change/close versions,
real replacement fixes and stale-diagnostic refusals. Existing inline UTF-16,
CRLF, JSX, kind hierarchy and native TypeScript controls remain unchanged and
mandatory in their normal Actions scopes. Passive existing session capture
retains complete framed input/output and failure streams; explicit source
receipt validation prohibits a global executable or build fallback.

The first source Check at `17dd7ed3` stopped the Rust lane on the extracted
constructor's `clippy::question_mark` requirement (job `111784552775`). The
equivalent optional lookup now uses `?`; all configured options, inputs and
complete runtime expectations are retained. Fresh successor Actions are required.

The same source-built tooling run retained 21 observations before refusing the
first suppression publication: the authored fixture incorrectly expected the
other rule to remain. Existing Patina `@vize:forget` treats its payload as a
reason and suppresses the entire next element. Both suppression variants now
require complete empty publications and null obsolete shifted requests; all 36
observations, individual-rule actions, restores and actual replacement-fix
controls remain. The official failed artifact `11347834938` retains the complete
process receipt and framed session (SHA-256 `7a12d20637a3823695445e5270c1779c397302104df7aec73de7f9daa923b001`).
The existing consumer-surface generator also registers only the extraction's
two moved coordinates and new test dependency row.

Source `7fbe62f7` retained 33 observations before refusing the secondary
`Fix.vue` publication: its configured single-word-name error must accompany
the spacing warning. The complete source-derived name diagnostic now remains
before and after the real spacing edit; the original URI, input, opinionated
preset, individual fix actions and all 36 slots are retained. Artifact
`11349626938` preserves that failed process and full session (SHA-256
`53491a7805a12fd445b40241300ff1c592568ee3e73830d3b95fdb6c53c90991`).
The accompanying Rust lib-test Clippy failure requires an equivalent borrowed
one-element slice; production code and diagnostic payloads remain unchanged.

Actual source Actions, complete new runtime observations, every old corpus,
protected full suites/all 104 unchanged instruction ceilings, signed actual
merge and release publication remain pending. Static source review alone gives
no runtime, Windows, native-migration, performance or delivery credit.

Paired issue decision: https://github.com/ubugeeei-prod/vize/issues/8000#issuecomment-5995087452
First source corrections: https://github.com/ubugeeei-prod/vize/issues/8000#issuecomment-5995695437
Second source corrections: https://github.com/ubugeeei-prod/vize/issues/8000#issuecomment-5996202440
