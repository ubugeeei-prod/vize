# Original HTML v-for callback diagnostic fixture

Issues: [#6879](https://github.com/ubugeeei-prod/vize/issues/6879),
[#6849](https://github.com/ubugeeei-prod/vize/issues/6849).
Private implementation based on literal main
`298f87942349401db0a3cbed2891897ba62c0eb8`; fresh source-built Actions and
protected actual delivery remain required. No runtime acceptance is inferred.

The single proposed `v-for-source-original` pack retains the whole original
HTML callback function from fix `04aedfb8e8b41dfae89fe65256e2703fc3718cf3`
([#3818](https://github.com/ubugeeei-prod/vize/pull/3818)) and parent
`b77e0a21485b9c9fb23506ee9b30fc857e4345a1`. It keeps the full authored SFC
with `const items: any = []` and `items.filter(value => value)`, together with
the original strict ES2022/ESNext/Bundler/noEmit `src/**/*` configuration.
There is no `vueCompilerOptions`, inherited configuration or extra checker flag.
The selected original function, entire test file, original helper excerpt,
source/configuration literal windows and whole Canon/Croquis patch are pinned
independently. Two ordered 334-line patch carriers preserve all original bytes.
They are historical references, never executed as current helper code.

The immutable whole historical public list has exactly TS7006, severity 1,
`Parameter 'value' implicitly has an 'any' type.` in `src/App.vue` at one-based
authored UTF-16 6:36. The original test cites byte-identical vue-tsc 3.3.4 /
TypeScript 6.0.3. Its helper sorts the entire vector; the one-row result has no
ordering ambiguity. The current shared production `BatchTypeChecker` observer
must compare every returned diagnostic, in order, and fail if it adds, removes
or changes a row. No expected value is recaptured from a current backend.
The public legacy API supplies no end, related information or raw backend
report; those fields and the actual historical loader/SDK snapshot remain
unavailable. Available current block/status fields are retained unbaselined.

The original optional-runtime early return is replaced only at this registered
fixture boundary by the existing mandatory real-runtime T1 harness. Its actual
source-built archive receipt, executable hash, copied input hashes, complete
public result and successful JUnit body are reconciled across four workers.
Absence, checker failure or a vector mismatch fails honestly; successful
observation artifacts are not claimed after a failed assertion. Unchanged
contract declarations first move to a support module in a move-only commit;
the only callable addition is one small `check_pack` test. The PR tier excludes
that exact runtime test; the protected full profile has no filter and no retries.

All 28 prior projects, 11 packs and 100 source carriers remain byte-identical.
The proposed totals are 29 projects / 12 packs / 102 carriers, pending actual
qualification; every native adapter stays null. The historical ledger stays
**271 / 263 / 31**, and no historical row or whole issue is closed by this case.
Current original-File and selected-template gates still refuse the original
`any` declaration and callback/For projection. No primitive surrogate or native
source replacement is introduced, and full graph/configuration/SDK/snapshot/ABA
coherence and default replacement remain unfinished.

The other three #3818 projects, contextual generic-slot omission, imported
template props, source-offset/name-capture consumers and Vue2 snapshot remain
TODO. The #6009 full original project/comment-scanner obligations, remaining
#3783 content-mapper/snapshot obligations and the separate syntax-only Program
proposal remain read-only scope under the
[original coordinate audit](./2026-10-04-typechecker-coordinate-history-audit.md).

The maintainer's reproducible **10x typechecker** target is unfinished. This
fixture establishes a diagnostic contract, never timing evidence. A separate
existing-workflow baseline proposal must require identical full diagnostics,
input/configuration/tool/binary identities and repeated measured process
boundaries; no benchmark campaign is dispatched by this change.
