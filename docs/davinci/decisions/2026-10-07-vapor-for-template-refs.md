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

The bounded source peer found that the first wrapper's global `Object`
reference could resolve an authored setup declaration. Use a null-prototype
getter map and exact string `in` membership instead, with no user-shadowable
intrinsic reference. The same ref-present bootstrap's existing marker definition
uses a fresh-object constructor; ref-absent and VDOM emission remain unchanged.
An independently authored Vapor scalar control adds only `const Object = {}`
to the exact existing scalar source, retaining full mounted-node/ref/unmount
expectations. Seven mounted laws now prepare 36 mounts/114 phases; two unit
laws remain prepared, not runtime acceptance. This preserves the first 86ec
run while repairing the concrete lexical ownership issue.

First 86ec ordinary Actions passed production Clippy but stopped test compilation
before any new mounts: the two new unit buffers need the existing L0 allocator
and `Vec::new_in`, not a nonexistent allocator-free constructor. Correct only
that test setup using the actual exported allocator/Vec API; retain the complete
240420-byte compiler log and its original E0599 outcome. This adds no product
allocator or pipeline change and supplies no execution credit.

The bfdea test compilation then exposed the exact allocator argument trait:
OXC Vec::new_in accepts `&impl GetAllocator`, while L0 implements that trait
for `&Allocator`. Both test buffers therefore pass `&&allocator`, exactly as
the current SFC CSS tests do; the earlier single-borrow source inference is
withdrawn. Preserve the 240803-byte E0277 log/SHA256
7d6352cfd40651a5ad4e5cf44074343a068b855df50a0944e54245a230dae73b.
Every product byte, mounted vector, fixture, allocator API and budget is unchanged.

Source composition now genuinely incorporates actual signed main
ef84821d30fa0d8538b2472fef34418e75380523, including the delivered critical
shell-quote repair (#8137). Preserve every incoming canonical line prefix and
all incoming source/API/default/configuration changes. The corrected 524a
producer passed Clippy and all test compilation; four Rust workers were still
executing when this union was prepared. Their results are historical source
evidence only, not current-union runtime/protected/merge credit. Re-run ordinary
Actions on the genuine union without any advisory, fixture or budget waiver.

Historical 524a source execution is now authenticated: affected Clippy/build
and all four Rust shards passed; all nine named new laws passed. Four small
JUnit/process ZIPs authenticate all 16518 executed cases and exactly 36 retained
source/module/process packets with 114 whole phases, including every original
backend/build-mode combination, raw numeric `3`, actual upstream `a,b` then
`a,b,c`, removal/clear, scalar and callback/null cleanup, and authored Object.
All ZIP digests/89 CRC members and source bytes match. This fixes the retained
first own failures without rerecording original sources or expected vectors.
Source receipt SHA256
9fe67755fddfab68a7cc6e60d1b025fd5a816cc36b229524c06368916f369602
is bound only to 524a/Check37493212500; that whole workflow still had the obsolete
security lock failure. The genuine ef848 union must pass its own fresh Actions
and full protected queue, signed merge and installed consumer gates.

Fresh ac1 source execution (Check37588012881) independently passes all four
Rust workers and all nine new laws. Four authenticated small ZIPs retain all
16518 cases and the exact 36 source/module/process packets/114 whole phases;
full input bodies, modes and complete phased vectors equal the prior authored
controls. Source proof SHA256
af4cb0bc2cfdf4c60bfcb552193f36394de18a4a1c6619f14a3474b275a07e5e
is specific to ac1 and its generated dcb1 checkout, with no protected/install
credit. The same run's two tooling failures identify our missing generated
Croquis BindingType consumption row: the new retained-metadata helper changes
17 files/133 sites to 18 files/140 sites. Run the existing whole-source Node
generator and check; it changes only that SFC shard row, not product code,
fixture inputs, assertions, provider APIs or instruction ceilings.

The fresh security gate also detects new GHSA-6qxp-vccf-f47h in the actually
locked @modelcontextprotocol/sdk 1.30.0 under npm/mcp-musea; the primary report
requires >=1.31.0. The delivered shell-quote fix is already incorporated.
Keep this existing PR Draft/off queue while the security owner repairs the
new advisory; do not waive or raise its exact accepted-high list. Incorporate
the actual dependency merge and require fresh current-source/protected tests,
actual signed merge and installed replay. The component-ref half of #7882
and broader direct-array binding remain unfinished.

The separate MCP SDK repair #8148 actually signed-merged at
2026-10-07T08:21:15Z as 706a5b7886c363f6c67a03964ac55f26c5a2a341
(valid GitHub signature, sole parent ef848). Genuinely incorporate this actual
main, including its locked 1.31.0/catalog and complete new decision record.
The automatic merge preserves every incoming canonical line prefix at 350
lines and all ref implementation, original fixture, complete law and runtime
observer bytes. No package, advisory-list, cap or expected-vector edits are
made by this slice. The whole 5f source Check also completed: all four tooling
and Rust shards plus strict canonical corpus passed; its only failures were
the now-remediated SDK advisory and required report. Its independent source
receipt retains nine laws/16518 cases/36 mounts/114 exact complete phases
(SHA256 54c2683e4a57de9deab6dce3ec643fbb8e516aa2892bfa5d3f1a7cd31f984e34).
These are historical source results; the genuine current union must pass its
own exact-head Actions, protected full Rust/unchanged ceilings, actual signed
merge and installed replay. Restore normal Ready because the concrete source
and controls are complete and the actual security prerequisite is delivered;
auto/queue remains off until current-source checks pass. Component/defineExpose
and broader direct-array acceptance remain unfinished; #7882 stays open.
