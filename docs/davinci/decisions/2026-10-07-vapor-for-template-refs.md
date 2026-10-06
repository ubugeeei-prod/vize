# Vapor loop template refs (#7882)

Paired decision: https://github.com/ubugeeei-prod/vize/issues/7882#issuecomment-6019300313

The original report has two independent failures: component refs are forwarded
as props, and native element refs inside `v-for` lack array ownership. This
slice fixes only the latter. Keep #7882 open until the component/defineExpose
half also passes the complete original `world / 3` mounted result.

The existing retained Vapor lowering already emits the runtime setter's
incorrect fourth `true` argument for authored `ref_for`, but no transform injects that
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
add removal/clear, function refs, transparent/nested loops with a scalar sibling
AFTER the loop, and the original numeric-loop-only clause. Every derivative
is named and hash-bound separately; the whole original component report is
retained without pretending that its remaining symptom is fixed.

Six laws compare complete independently authored tree/namespace/event/exposed
vectors under actual DOM and Vapor runtimes in development separate-template,
production inline-template and production separate-template modes. The original
numeric derivative keeps authored `<script setup vapor>`, which forces Vapor
even when the requested backend is DOM; run that derivative honestly only via
createVaporApp. The other five ordinary-script sources exercise both actual
backends (33 total mounts), with no input or oracle weakening.
The existing SFC runtime helper gains opt-in calls/reads of actual exposed
methods; every ordinary observation is unchanged when these inputs are absent.
The new requests retain complete input/modules/source, status, stdout bytes and
stderr bytes beside the existing Nextest JUnit BEFORE process assertions.
The stdin-write Result is also retained; EPIPE still reaps the child and saves
its complete process/error witness before failing the original assertion.
Original compile/ref snapshots and existing mounted tests are not rerecorded.

Source and mounted execution are pending exact-head ordinary Actions; protected
full Rust, unchanged 100+4 instruction caps, actual signed merge, and a later
installed release remain required. Cached installed 0.435.0 failed this plant;
that historical result supplies the reproduction, not current-source proof.

The first exact be8 Actions compiled all source but failed all six new laws.
Static DOM controls passed before their first Vapor failure; DOM trees were
correct while actual refs stayed empty. Preserve those complete process/module
packets, including the numeric `0` result, rather than changing expectations.
The primary pinned Vue rc.9 setter has signature `(el, ref, refFor?, refKey?)`;
move the array bit to its third argument. Its factory captures the scope owner,
not an early setupState snapshot. The earlier early-capture hypothesis is
withdrawn. Public `getCurrentInstance()` intentionally excludes Vapor, and
string-to-setupState assignment is development-only in this runtime.

Join ref-capable original setup declarations through the existing SFC-captured
setter. A private generated getter map is limited to actual SetupRef,
SetupMaybeRef and SetupLet metadata already retained by the script provider.
Static string keys present in that own map become their raw declaration value,
with the key retained as refKey; other strings, functions and Ref arguments
reach the original setter unchanged. Getters preserve a reassignable declaration
rather than capturing a proxy-unwrapped value. Factory creation still occurs
inside the same actual owner; its cleanup and post-render queue stay native.
No generic-instance/private runtime export, instance mutation, additional
parse, AST traversal, pipeline stage or adapter process is introduced.

The independently authored `:ref="itemElements"` broad derivative also failed
its first Vize DOM arm: Vize passes a proxy-unwrapped array there. Retain its
complete source and authentic failure as an unfinished direct-ref control;
this is not a universal refusal by Vue (its compiler uses binding metadata).
The admitted function-ref control instead stores only callback-provided actual
nodes, and removes the exact item key on the native null cleanup callback.
All whole expected tree/ref vectors remain authored from that source's semantics;
original issue, original upstream plant, numeric/static/removal/nested/scalar
sources and expectations remain byte-exact. Thirty-three mounts/108 phases are
prepared, not claimed executed. Shared security and all current mounted/full
queue gates remain unresolved until fresh actual Actions and signed delivery.
