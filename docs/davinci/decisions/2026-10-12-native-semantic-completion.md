# Required native semantic completion and terminal ownership

For #6879 and #3984, qualify the parser-to-configured-session completion and
native process teardown on the source-built CLI in the existing native-phase
Actions job. This supplements the original 400-case paired gate, full Stack
qualification and App snapshots. It changes none of their inputs or verdicts.

The frozen input is original record 271, `leaf-parity-escaped-import-keyword`,
base `d55b8238`, retained in the composed `15e51ec040` artifact. The complete
fixture is in `tests/_fixtures/differential/typechecker/native-semantic-completion`.
The corpus manifest binds repair `7a782477742608f5fa98646448c53321974f6d44`
and original source `d55b8238f89bab4418a1a961bd8db1c84911cb42` separately;
the original complete provenance records the composed artifact head and both
CLI/native binary hashes without replacing them with the current producer.
Its manifest authenticates eight physical input files: four Vue files,
`shared.ts`, `tsconfig.json`, and both original `leaf-types` package members.
The original public packet reports five program files. Preserve every original
input byte and the original command, stdout and failing stderr in the fixture.

The unchanged expected public stdout is the entire original packet: TS1260 at
`Comp0.vue` 1:59, TS2304 at `shared.ts` 1:21, all five file rows, complete
program/config/options membership and both aggregate errors. Required parser
completion retains public exit 1 and produces empty stderr after the repair.
The original authored `shared.ts` program's `LeafGlobal` diagnosis remains an
intentional negative; its eight original inputs remain byte exact. A separate
ordinary-import control in the generated program must resolve that global with
a complete clean vector. A normal TS2322 case and
its actual repaired source require complete wrong and clean public packets.

The private batch law uses the normal factory and the public path's complete
requested URI workload. Its
test-only writer retains every generated and physical file, source maps, the
whole raw diagnostic response, selected config bytes, full normalized options
and roots, and `getSourceFileNames` request/response from the diagnosing LSP's
retained API attachment. The process observer belongs to this same LSP owner.
The initial `041d5df` Actions qualification found the original leaf declaration
in native source membership, disproving the test's assumed absence. The revised
law reads its complete encoded source text through that retained attachment,
matches all 55 original physical bytes, and compares the complete project and
source membership. The `b872794` full artifact disproved another test assumption:
the generated program genuinely loads that global and has no TS2304, while the
public path also requests the original authored `shared.ts` program, which
produces the frozen TS2304. The private law now requests those same six URIs,
retains both complete configured-project receipts from the one diagnosing owner,
and requires the unchanged whole two-diagnostic vector. Its source view uses the
reviewed test-only unique whole-descriptor selection and preserves the actual
production singleton admission or refusal with the full native project vector.
A second actual configured session uses only the ordinary import correction and
requests the five generated-program URIs; it requires no diagnostics while
proving the same loaded declaration. This positive is generated-program scoped,
not an authored-public-program clean claim.
After explicit close, require exactly one completed receipt, full empty stderr,
actual exit 0, reap, no timeout or kill and no reader/process errors. The
[authenticated native close capability](./2026-10-12-native-shutdown-eof-capability.md)
retains the complete initialize packet. This observation does not replace the diagnosing
process with a separate native API invocation.

Three executable controls keep the real native executable and original input.
Unexpected UTF-8 and non-UTF-8 stderr prefixes must replay the entire wrapper
prefix and wrapper-authored EOF byte vector and preserve original stdout.
Each adapter owns a separate process: retain wrapper and actual native PIDs and
statuses, require actual native exit 0 and wrapper exit 17, and refuse a clean
public result with empty stdout and exit 1. Keep the complete public stderr;
the wrapper's status is never claimed as native's status.
The bounded control adapter forwards full original protocol frames and retains
both raw directions. Only an exact params-less/id-less exit after the matched
native shutdown response becomes native stdin EOF. Every other frame remains
byte exact. The authentic native child is reaped with exit 0 before the adapter
injects the final EOF line or exits 17; native stderr remains independently
retained and fully replayed. Wrapper bytes/statuses are never attributed to native.

`tools/benchmarks/scripts/typechecker-native-semantic-completion.sh` requires
the exact `SOURCE_SHA`, actual native path and required-native environment,
unsets disabling and unrelated capture flags, and reuses the existing locked
`ci-opt` profile, features and target. It archives all test and child packets
under `$RUNNER_TEMP/native-source-recipe-semantic-completion`, including on
failure through the existing `native-source-recipe*` archive membership. The
initial run's old immutable packer omitted the former directory, so that run's
console refusal is preserved without claiming full packet custody. The new
directory is retained by both original and current packers. It adds no builder.

Root formatting excludes only the two original published
`vendor/{corsa_core,corsa_lsp}/.cargo_vcs_info.json` paths. Formatting them would
alter authenticated registry bytes. Mandatory complete published custody remains
required; all other project JSON keeps its original formatting qualification.

Source preparation, custody and format checks are not native acceptance.
Exact composed-head Actions compilation, all new real-native laws and the
unchanged full delivery gates remain required. No performance improvement,
API-only lifecycle coverage, fix-history closure or release claim follows
from this supplemental gate. The parent delivery change owns the paired issue
comment and workflow wiring.
