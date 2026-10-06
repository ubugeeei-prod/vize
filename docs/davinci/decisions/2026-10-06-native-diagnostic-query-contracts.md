# Native diagnostic query contracts after failed bulk gains

Issue: [#7698](https://github.com/ubugeeei-prod/vize/issues/7698).
Consumer: [#8038](https://github.com/ubugeeei-prod/vize/pull/8038).
This is a source design on eaa3268f, not an executed algorithm correction.
The [cost record](./2026-10-06-native-bulk-cost.md) retains the failed genuine
comparison: all nine lanes were slower, with median ratios1.409/1.608/1.602.
Keep the consumer Draft/offqueue. The finite0.434 release stays independent.

## Fixed primary authorities

Use the actual native package7.0.2 gitHead
`2bd066d87f5bafd315be9f40889d0a60b9e58e0b`, not upstream main or another
installed SDK. Cargo pins corsa_client/corsa_jsonrpc1.14.0; their cached crate
archives match the lock checksums and the inspected source matches those
archives. No dependency or native executable change is proposed.

- [API grammar](https://github.com/microsoft/typescript-go/blob/2bd066d87f5bafd315be9f40889d0a60b9e58e0b/internal/api/proto.go):
  category requests accept snapshot, project and an optional singleton file.
  There is no file-array diagnostic parameter.
- [API dispatch and snapshot lookup](https://github.com/microsoft/typescript-go/blob/2bd066d87f5bafd315be9f40889d0a60b9e58e0b/internal/api/session.go):
  syntactic, semantic, suggestion and declaration category endpoints exist.
  They resolve the project inside the supplied snapshot, then the optional
  source file inside that same program. Omitted file selects all program files.
  The SDK's getDiagnosticsForSnapshot/Project/File and describeCapabilities
  method names are absent from this native dispatch/unmarshaller contract.
- [Program collectors](https://github.com/microsoft/typescript-go/blob/2bd066d87f5bafd315be9f40889d0a60b9e58e0b/internal/compiler/program.go):
  category results are sorted and deduplicated. The native workgroup handles
  the all-file collector internally; it is not a public selected-file batch.
- [Project program construction](https://github.com/microsoft/typescript-go/blob/2bd066d87f5bafd315be9f40889d0a60b9e58e0b/internal/project/project.go)
  installs the custom project checker pool. The built-in compiler pool's
  grouped-checker branch therefore does not describe this retained project.
- [Project checker ownership](https://github.com/microsoft/typescript-go/blob/2bd066d87f5bafd315be9f40889d0a60b9e58e0b/internal/project/checkerpool.go):
  diagnostic lifetime uses the dedicated index0 checker and semaphore.
  Temporary query slots and persistent API identity are separate lifetimes;
  diagnostics cannot be relabelled to use those slots as a speed shortcut.
- [Checker suggestions](https://github.com/microsoft/typescript-go/blob/2bd066d87f5bafd315be9f40889d0a60b9e58e0b/internal/checker/checker.go):
  GetSuggestionDiagnostics calls checkSourceFile, including full checking when
  not already checked and unused checking when required. Thus whole-program
  suggestion collection can still check unrequested admitted files after the
  selected semantic calls. SkipTypeChecking remains authoritative.
- [Original LSP categories](https://github.com/microsoft/typescript-go/blob/2bd066d87f5bafd315be9f40889d0a60b9e58e0b/internal/ls/diagnostics.go)
  collect syntax/semantic/suggestion/declaration in that order for one file.
  The [LSP server](https://github.com/microsoft/typescript-go/blob/2bd066d87f5bafd315be9f40889d0a60b9e58e0b/internal/lsp/server.go)
  selects a language service by URI; its document diagnostic request has no
  API snapshot selector, and it registers no workspace diagnostic endpoint.

SDK1.14.0's JSON-RPC request_optional_value sends a request, then synchronously
waits on recv/recv_timeout inside its async function. API request_json awaits
that directly. Combining those futures does not establish concurrent pending
requests. Native AsyncConn does launch request goroutines and serializes reply
writes, so this finding is specific to the actual client path, not a claim that
the native transport rejects concurrency. No new thread pool, parallelism
budget or lifetime change is justified by these facts.

## Actual preserved workload

The authenticated historical fe781d/d43 archive11376158162, SHA256
3c1f8cb07b6fb07f9ebc9d677d87d4bb4e69891286961568a6e50dd55a445ff9,
contains681 distinct selected semantic names and3784 native source names in
each of its cold/broken/repaired snapshot generations. All681 names belong to
that sealed program. Each generation acknowledges683 category requests:
681 selected semantic calls, one whole syntax and one whole suggestion call.
The3103 unrequested names include native dependencies and library inputs;
the name count does not measure how many pass SkipTypeChecking or their cost.

The ONE earlier observer measured whole-program semantic work as the dominant
warm operation. It did not measure this later selected-file sequence. The
global suggestion source walk and681 SDK waits are concrete operations, but
neither their individual durations nor the slowdown's causal split is known.
No historical whole681 or gain evidence transfers to a new implementation or
current main. The current eaa ordinary/source-native passes are quality only.

## Supported options and chosen next source candidate

| Option                                         | Actual contract                                          | Qualification                                                                                                                                                                              |
| ---------------------------------------------- | -------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| Omit file on existing category endpoints       | Supported,3 calls or4 with declaration                   | Smallest candidate removes the selected semantic request loop; full-program checking remains. The earlier all-file implementation was also slower, so call reduction does not prove gains. |
| Select suggestion as well as semantics         | Singleton file supported                                 | 681 inputs would require1363 calls without declaration. Do not implement this unresolved request-count tradeoff.                                                                           |
| SDK combined project/snapshot/file diagnostics | Present in SDK source, absent from native dispatch       | Unsupported at the actual pin; no guessed endpoint or schema.                                                                                                                              |
| Parallel selected requests/workpool tuning     | Native transport and compiler internals have concurrency | No safe faster mapping is established for this client and dedicated diagnostic checker; no relabelled lifetime or invented concurrency limit.                                              |
| Reuse original LSP document endpoint           | Supported original category converter                    | A URI request does not select the sealed API snapshot. Keep the original complete fallback; same process alone does not prove immutable generation ownership.                              |

Prepare a bounded source candidate using the existing all-file category calls
on the same admitted snapshot/project/attachment. Preserve global syntax,
declaration/configuration behavior and category order; do not add a separate
program/global call, process, parser or pipeline stage. The complete response
must first satisfy the existing schema and exact native-source membership.
A malformed or foreign main source refuses the entire conversion before any
requested-name projection. Legitimate unrequested program rows may be omitted
from the requested output without reading their text; related locations on a
retained requested row still use the same snapshot-owned text provider.

Filtering must preserve each native category's row order and retain every
requested diagnostic, Unicode coordinate, nested message, severity/style and
related location. Reconstruct requested rows in the original request order,
including duplicate requests and successful empty vectors. Do not truncate
deep messages, hide a malformed/foreign source as an accepted empty result,
or substitute later filesystem text. Retain the complete original category
responses and full old/new raw witnesses in existing test custody before
assertions. Real request/release errors still eagerly attempt snapshot release,
retire/reap the owner and return the entire original LSP fallback vector.

TODO: implement and independently review that source correction, including
positive admitted unrequested rows and malformed/foreign/alias refusal controls.
Keep all original profile/retirement/lifetime/snapshot inputs and three complete
681 generations (2043 raw/public rows); update actual request-count laws to the
real whole-category calls rather than relabelling historical683 acknowledgements.
Require fresh compilation, all original native laws and full source equality.
Only after a defensible all-build correction and root's publication thaw may
ONE observer0, same-host, distinct fresh-binary matched comparison run, with all
physical source, config, driver, native PID, budget and whole-vector guards.

This candidate may only restore a prior request shape. Genuine faster matched
results are still required for Ready/admission, followed by native Stack
membership, protected fullRust/unchanged104, actual signed delivery and release.
No second campaign, default/CLI/LSP/native-product completion, budget change or
10x claim follows from this source design. #7698 remains open.

## Private source correction

The same private consumer now implements the existing3/4 category calls. Build
exact native-name and requested-name sets once from the admitted provider;
decode the entire category and reject empty/foreign main names before stable
requested-name filtering. A malformed unrequested row also refuses the whole
batch. Requested spans, nested messages, style and related locations still use
the existing complete converter and same-snapshot text reads. Unrequested main
rows do not enter that converter or cause related-text reads. Every actual raw
category response or real request error is retained before decode in cfg(test)
custody; ordinary builds add no observation serialization.

Two added pure laws cover complete output/order/related/duplicate/empty behavior
and atomic foreign/alias/fileless/malformed-unrequested refusals. Preserve the
two older selected-file laws byte-exact. The existing five-original-file native
law retains all original source bytes and whole LSP comparisons; its expected
request array now contains three actual global categories, and its full raw
semantic witness must contain the independently erroneous unrequested file.
All original eight profiles, reader retirement, lifetime and whole681 inputs
remain unchanged. Existing helper commands automatically include the two added
pure laws; no workflow/driver/pin/budget alteration is required.

Compared with eaa's current loop, this removes681 singleton semantic requests.
Compared with the old source9f5ee043, the call shape is restored, not new: that
source already made3/4 calls and already avoided related-text reads for
unrequested rows inside its URI grouping guard. Genuine new differences are
exact all-main ownership admission, filtering before unnecessary unrequested
path-to-URI work, and full raw category failure custody. It does not establish
cheaper native checking, save previously eager unrequested text decoding or
prove gains. Returned-row observation counters still count the full validated
response, with the ordinary observer disabled.

Rust formatting, source census and diff checks may qualify this private source,
but compilation and all execution remain pending. During the0.434 hold do not
push it or dispatch the whole681/matched campaign. After thaw and frozen source
review, existing ordinary Check and native-phase Actions must compile every
affected feature path and execute18 bulk laws (14 pure/4 native), unchanged
snapshot/lifetime laws and all original helper commands. Three global requests
per profile, plus declaration when configured, should replace historical102
profile acknowledgements with78; the72 whole response comparisons remain.
These are intended counts, not observed execution. Full681/2043 parity and a
defensible ONE matched observer0 comparison still gate performance acceptance.

The frozen production review found no source blocker, but the eight-profile
harness still unwrapped bulk/original Results before its first disk capture.
Retain the full bulk Result and category custody immediately before unwrap,
then append/save each complete original Result before unwrap or the next query.
Keep prior bulkOutcome/comparison fields, all72 equality checks and the new3/4
request assertions. This test-only failure-first correction changes no native
operation, product, fixture or gain claim; fresh compilation/execution remains
mandatory and the source is still private during the publication hold.
