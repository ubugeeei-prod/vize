# Original import-meta and slot-key diagnostic projects

Issue: [#6879](https://github.com/ubugeeei-prod/vize/issues/6879).

## Exact original requirements

Add two authored projects to the existing shared batch fixture adapter:

| Original fix                                                                                          | Exact authored project                                                                 | Complete original expected diagnostic vector                                                     |
| ----------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------ |
| `6e0f763bd0f5e986036244c2c17c9b658b0596bd` ([#7109](https://github.com/ubugeeei-prod/vize/pull/7109)) | `src/env.d.ts`, `src/App.vue`, original helper `tsconfig.json`                         | One TS2322 at `src/App.vue:3:7`, severity 1, `Type 'string' is not assignable to type 'number'.` |
| `c6e43ca98cbffe099a6ef323006139d796d62208` ([#7108](https://github.com/ubugeeei-prod/vize/pull/7108)) | `src/Declared.vue`, `src/Inferred.vue`, `src/App.vue`, original helper `tsconfig.json` | Empty                                                                                            |

The original `import.meta` input uses the project's custom `ImportMeta`
interface. Its string-valued mode assigned to a number must remain a real
type error. The original slot project forwards a numeric loop key through both
declared and inferred slot payloads, then calls `key.toFixed()` in their parents.

Every input carrier preserves the original Rust raw-string bytes, including
terminal newlines and the helper config's lack of a terminal newline. Each pack
retains the original test source, exact helper excerpt, Git blob identities,
SHA-256 values and literal byte offsets in `input-provenance.json`.
The provenance law compares the actual carriers against those source spans.

Expected diagnostics come from the complete original authored assertions;
they are not newly measured or invented baselines. The old helper projects all
diagnostics and then sorts the vector. These vectors contain zero or one row,
so the new returned-order assertion preserves the complete original expectation.
The helper's explicit error-to-severity mapping supplies severity 1.

## Mandatory execution and shared accounting

The existing optional unit tests stay unchanged. Two new ordinary tests in the
existing `fix_history_diagnostics` binary execute the production
`BatchTypeChecker`. Missing real Vue, failed construction, scanning, checking,
or any unexpected diagnostic fails. There is no runtime-absence early return.

The two exact test identities join the audited PR deferral list. The full
required T1 profile still selects every workspace case, requires real TSGO and
uses zero retries. No full-tier filter, instruction ceiling or ratchet changes.

The shared adapter now registers eight packs / 19 projects / 74 source and
configuration carriers. Existing pack bytes and expected diagnostics remain
unchanged. Each new pack pins its original fix revision; the original six keep
their original fixture revision. Registry and pack identities must agree.

The already implemented full-worker observer saves these actual vectors after
their unchanged complete assertions. Successful JUnit bodies, actual extracted
executables, input hashes and source/build archive receipts remain mandatory.
All four current-attempt workers must reconcile every registered project.
The actual dependent slice must use the same protected native GitHub Stack
policy as its provider and capture consumer.

## Limits and verification

These are start-position batch diagnostic contracts. End, related information
and raw backend fields are absent from this public API and stay unavailable.
All 19 rows retain explicit native unsupported state and zero native credit.
Actual public block types, exit codes and success are retained beside each
original assertion as observed/unbaselined fields. They do not become a full
public-result match until fresh actual captures are frozen and checked.

The original slot input does not prove declared slots omitting `key`, negative
key-type diagnostics or complete mapping ranges. The import-meta input does
not cover every module-target, comment or string-literal boundary affected by
the removed polyfill. Whole bundled fix-history coverage remains incomplete.
#6879 stays open, and no product route changes.

Original-byte/projection laws and shared rejection/accounting laws pass locally.
They establish provenance and mandatory registration, not actual checker
execution. Local Cargo, Clippy and real TSGO runs were held for limited disk
space. Fresh exact-head Actions, both actual project checks, complete shared
worker artifacts, protected queue execution and actual main merge are pending.
