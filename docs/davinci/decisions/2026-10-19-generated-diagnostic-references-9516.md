# Separate native references for hidden generated diagnostics

Issue #9516 tracks 228 TS2493 and 187 TS2353 findings reported during the
historical Misskey investigation. Their original unfiltered diagnostic packet,
generated programs and mappings remain unavailable. Neither the older assembled
Matrix JSON nor the custom-directive artifact supplies that evidence. These 415
findings remain unclassified; this change does not close #9516 or reopen #9016.

Two different, retained references demonstrate why projected diagnostics alone
cannot establish that a generated program is clean:

- Escaton's #9427 fixture records two TS2493 findings on generated empty-event
  tuple reads. Commit `2e5e976ec6105311fd7a4399c0f6cd8b6dfb8250` (#9435) repairs
  the deferred type lookup and runtime extraction.
- Escaton's #9428 fixture records a TS2353 on generated `dataTest` event-prop
  inference. Commit `96f6a1b52b` (#9437) repairs the props contract.

The original sources and complete official-compiler output fixtures stay in
`tests/_fixtures/content-mapper/escaton-component-{events,props}.json`. The
existing official Content Mapper campaign continues to compare those complete
streams and its authored-error controls without changes.

## New reference boundary

The new native test reuses those authored SFCs through the current Batch API.
It removes only the external `contentMappers` plugin setting from the new
Batch configuration and retains the original configuration separately. It uses
the installed test Vue provider and the selected Corsa backend, rather than
claiming the historical reporter's compiler, provider or project identity.

The test retains the complete generated configuration, authored and generated
modules, source maps and decoded pre-assembly diagnostic vectors. These vectors
contain full messages, start/end ranges, codes, severities and source fields;
they are decoded `LspDiagnostic` values, not untouched RPC frames. Native image
bytes/hash, Vue manifest and the returning client's selected-project receipt
identify the observed backend and program.

The two valid sources must have no raw errors or public findings. Their complete
raw suggestion vectors must match the named, unused generated declarations,
including TS6196, severity 4, source, full messages and generated ranges. Separate
authored TS2493 and TS2353 controls require their complete messages, generated
ranges and authored positions to survive. One counterfactual restores only the
pre-#9435 runtime expression in the current generated program; it must produce
an unmapped TS2493 with the exact generated range. This is a helper replay, not
execution of the old factory. Current source maps are retained for context but
are not applied to that different-length replay.

## Execution

The existing Native Canon action runs the new test with its required backend,
serially, after `native_syntax_semantics`. Its existing always-upload artifact
retains the packets under `native-syntax-semantics/generic-component/`.
The ordinary Rust cohort also compiles the test; its PR envelope intentionally
disables native execution and supplies no native-success credit.

No generator or assembly policy changes are justified by the available
evidence. In particular, TS2493/TS2353 are not added to the three-code checker
give-up policy. Actual execution of the new native reference remains an Actions
qualification requirement. Recovering or reproducing the original Misskey
classes remains separate work.

The first execution reached only the empty-event case: its raw response also
contained four TS6196 suggestions for unused generated type aliases. Its full
messages/ranges and generated declarations are retained; this is not a new
TS2493/TS2353 finding. Cases now retain their packets independently before the
overall gate: a caught assertion continues evidence collection, and every such
failure still fails the test at its final gate. No raw suggestion is filtered
or accepted by that observation change.

The next genuine capture reached all four cases and the replay. The empty-event
source had four TS6196 suggestions; the data-attribute source had two. Both had
no errors and no mapped findings. Each authored negative retained its one error
and the same four suggestions; the old runtime expression produced one generated
TS2493 and four suggestions. The expectation repair describes these whole
vectors, including their request URI owners, without filtering or sorting the
observed vectors. Every missing, extra or altered diagnostic still fails.
Those captures used TypeScript 7.0.2 and the locked test Vue 3.6.0-beta.10,
not the official Content Mapper fixture's Vue 3.5.20. Successful execution of
the repaired source remains required; the captures themselves were failed tests.
