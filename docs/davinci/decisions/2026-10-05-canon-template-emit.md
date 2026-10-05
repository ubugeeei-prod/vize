# Template emits from authenticated setup macros (#7820)

The original unassigned type-only `defineEmits` SFC and complete authored
TS2345 event-name/payload vector are retained without editing reporter bytes.
The template context currently reads the unconstrained public instance `$emit`,
which loses the component's macro contract.

Use the same setup macro return type (`__EmitFn`) for type-only declarations
inside the setup lexical scope. This preserves local aliases and generic type
parameters. Runtime declarations infer from the same setup helper overloads.
The existing semantic helper plan excludes an authored/shadowed `defineEmits`;
absent macros continue to use their public instance contract. No generated text
rewrite, extra pipeline stage, native-stage claim or blanket diagnostic mask is
introduced.

Required source-built CLI and official TypeScript native 7.0.2/Vue declaration
oracles compare complete output/status for the exact original, valid calls,
runtime array names and unconstrained payloads, renamed local call signatures,
shadowed macros and absent macros. The real editor service compares both complete
original diagnostics and the exact typed `$emit` hover/range. Raw original and
control bytes, full outputs and official binary/package/source hashes are kept
in the native-phase Actions artifact. Expectations are independently authored;
actual execution remains pending. The generated consumer surface inventory is
regenerated from source.

Fresh exact-head strict/source/native Actions and unchanged instruction ceilings
are required before readiness. The release owner's first-cut publication hold
continues to govern queue entry; protected full/104-platform suites, actual
signed merge, issue closure and public consumer qualification remain separate.

Qualification shares the existing pinned `npm/cli` real Vue dependency through
normal ancestor lookup; workspace root has no direct Vue dependency. Isolated
case paths use that package without installation or symlinks. The editor binary
is pinned to the same authenticated official oracle; complete editor fixture,
config and raw response bytes are retained before full assertions. Original
source and all independently authored expectations remain unchanged.

Independent source review identifies model update events as part of the template
public contract. Combine authenticated macro emits with model updates using the
existing exact `model_update_payload` calculation; optional/required/default
semantics are shared with component emission. A required string model plus
numeric `change` event has complete valid/invalid update payload controls.
Generated Nuxt strict mode keeps `$emit` on the declared local binding (both
strict identifier paths exclude declared template context names), verified by
running the original full error vector through the strict CLI case. A generic
setup control checks the local type parameter. These fresh executions, together
with the original and actual editor vectors, are required before readiness.

The same source CLI deliberately writes two progress lines to stderr. Assert the
complete authored one-input count/root progress response, retaining raw stderr
and complete stdout vectors; no output filtering or runtime change is added.

At 02cbd, original/strict/generic/local/valid/absent/shadowed complete vectors
pass. The runtime array oracle displays the canonical union as change | click;
the model oracle emits complete TS2769 last-overload detail at the invalid
payload. Correct only those explicitly authored rendering/range expectations;
event membership, model payload semantics and original inputs remain unchanged.
Model CLI and original editor acceptance still require actual execution.
Artifact case labels replace Rust namespace colons for portable filesystem
names, retaining the original test name in metadata; receipts assert exact nine
names separately from editor-inputs and include original hidden fixture bytes.

At 8f963, all event membership controls pass. The remaining mixed-model CLI
assertion differs only in multiline diagnostic indentation: the established
CLI renderer retains every line without the native text printer indentation.
Keep separate explicit full CLI and native TS2769 expectations; neither output
is filtered and no production source or diagnostic range changes. Exact-nine
CLI plus complete editor acceptance must pass again on the successor head.

At 61465 all nine native/CLI full vectors pass, including the mixed model
positive and negative calls. The original complete editor diagnostics and
range pass, but native QuickInfo at the invalid call prints a union event and
never[] rest instead of the independently authored overload presentation.
Retain that failing assertion while adding an independent actual-Vue native
LSP oracle with both valid calls and the complete two bad-call diagnostics.
Capture its full invalid/valid QuickInfo, source/config, runtime identity and
pinned Vue declaration bytes. Compare original and oracle presentation before
any expectation decision; never[] is not unconditionally accepted. Refresh
official generated consumption inventories for new source inputs.

The bridge hover view supports deserialization only. The oracle capture
serializes every contents variant and range field explicitly in the test
helper, without changing public bridge types, content or any assertion.
