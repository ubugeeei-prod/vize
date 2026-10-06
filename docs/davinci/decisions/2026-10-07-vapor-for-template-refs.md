# Vapor loop template refs (#7882)

Paired decision: https://github.com/ubugeeei-prod/vize/issues/7882#issuecomment-6019300313

The original report has two independent failures: component refs are forwarded
as props, and native element refs inside `v-for` lack array ownership. This
slice fixes only the latter. Keep #7882 open until the component/defineExpose
half also passes the complete original `world / 3` mounted result.

The existing retained Vapor lowering already emits the runtime setter's
fourth `true` argument for authored `ref_for`, but no transform injects that
attribute. Track lexical For depth around its actual child lowering and use
that context when producing `SetTemplateRefIRNode`. Nested/transparent loop
bodies inherit it; leaving each body restores the enclosing depth. Existing
explicit `ref_for` and loop-free scalar refs retain their behavior. No parser,
walk, serialization, public API, additional pipeline stage, or budget changes.
This is existing-product correctness, with no new Davinci eligibility credit.

The differential directory retains the whole original issue, both original
SFCs and command, and the immutable upstream `template-refs-v-for-update`
plant at pikax/vue-benchmarks `0e3c360a4780e74e9dd4888cb6a56a24372020d4`.
Its actual exposed `referencedText` must be `a,b`, then `a,b,c` after `add` and
nextTick; no replacement alias or synthesized ref value is used. Derivatives
add removal/clear, bound refs, transparent/nested loops with a scalar sibling
AFTER the loop, and the original numeric-loop-only clause. Every derivative
is named and hash-bound separately; the whole original component report is
retained without pretending that its remaining symptom is fixed.

Six laws compare complete independently authored tree/namespace/event/exposed
vectors under actual DOM and Vapor runtimes in development separate-template,
production inline-template and production separate-template modes (36 mounts).
The existing SFC runtime helper gains opt-in calls/reads of actual exposed
methods; every ordinary observation is unchanged when these inputs are absent.
The new requests retain complete input/modules/source, status, stdout bytes and
stderr bytes beside the existing Nextest JUnit BEFORE process assertions.
Original compile/ref snapshots and existing mounted tests are not rerecorded.

Source and mounted execution are pending exact-head ordinary Actions; protected
full Rust, unchanged 100+4 instruction caps, actual signed merge, and a later
installed release remain required. Cached installed 0.435.0 failed this plant;
that historical result supplies the reproduction, not current-source proof.
