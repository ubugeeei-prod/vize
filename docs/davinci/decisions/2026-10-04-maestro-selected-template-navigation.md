# Original selected-template Maestro navigation

Issues: #6836, #6871, #6872 and #6883. This is an additive explicit preview
consumer, separate from existing original Program/NativeSfc navigation and
standard product handlers. Its real dependencies are #7633 (the once-selected
whole SFC owner) and #7640 (shared original Handler/For File position queries).
Both providers actually merged before this consumer publishes: #7633 at
`d807a097efc081cf9501a626146370c84fb54c72` and #7640 at
`c241d75fa0bceea9479098d24b820dc1411f539c`, with terminal protected Checks
37154243123 and 37154511071. Private source is not hosted execution or product completion.
No synthetic integration branch, second parser or legacy fallback supplies a
missing prerequisite.

## Actual request and source ownership

The existing `experimental-source-navigation` feature registers two additional
explicit requests through the real production LspService builder:

- `vize/nativeTemplateDefinition` accepts GotoDefinitionParams and returns the
  array of every original authored declaration Location. No preferred var site
  is invented; an implicit `$event` has an empty definition array.
- `vize/nativeTemplateReferences` accepts ReferenceParams and returns original
  use Locations, adding the real declaration sites when includeDeclaration is
  true. Sorting applies to the owned response spans, not resident File facts.

Existing `vize/nativeDefinition` / `vize/nativeReferences` keep their original
Program and normal NativeSfc routes, including the actual JSX/TSX ProgramOptions
and Vue script/expression embed behavior. Standard LSP handlers and defaults
stay under their existing product gates. Selecting a different request family
does not try one producer and then parse the same input through another.

SelectedVue carries the actual server VueConfiguration as a coarse freshness
key. The source role remains editor SFC/Vue; raw configured dialect values do
not confer standalone HTML or other dialect admission. The actual once-selected
producer currently refuses all scripts/styles. Therefore genuine handler-local
navigation is the ready admitted scope, while whole-SFC For collections needing
setup bindings remain original producer refusals. SetupProgram custody and
broader For/outer-binding navigation are separate unfinished provider/consumer
slices; no File symbol is forged to bypass that boundary.

The dedicated worker calls `lower_selected_sfc_native` once for its immutable
SourceSnapshot. Snapshot, Allocator, Descriptor/whole observation, consumed
NativeTemplateView and completed File remain ordinary thread-local owners for
the whole mailbox loop. Original HandlerBody syntax and resolution tables stay
with that same File. Only owned command inputs and Location/error responses
cross the channel. No AST/File/allocator becomes Send/Sync, no self-reference,
unsafe lifetime cast or borrowed node survives retirement.

Definition sites come from the actual HandlerDeclaration rows and their existing
retained authored projection; references come from the shared sealed File query.
No source-name scan, numeric ID, synthetic binding or re-resolution determines a
target. Actual half-open UTF8 spans become authored UTF16 ranges through the
existing original source coordinate API. A provider refusal discards every
partial callback result before producing any Location or publishing a response.
The real selected-SFC refusal remains owned and sticky through its worker loop.

## Concurrency and final publication

Two coarse URI maps keep Original and SelectedTemplate observations separate,
sharing one real atomic live-owner limit of 16. Each mailbox remains bounded at
16 with explicit Busy refusal; capacity/spawn failures stay retryable. The slot
is released after the original locals have dropped, including panic/unwind.
No cache lock survives parsing, awaiting a response, joining or publication.

Both maps check physical snapshot/revision freshness under their own admission
mutex. Genuine host-change hooks retire stale entries in both maps while
preserving a newer entry captured between the mutation and notification.
Close and project drop retire both families without joining under the locks.
Actual configuration changes are checked before the final host-guarded
publication; an old response refuses and both changed caches retire. Source
query cancellation and a dropped response receiver prevent queued work or stale
publication. No coarser digest or equal source bytes replace physical ownership.

## Genuine laws and acceptance still required

Private source adds fifteen real SourceProject laws and four whole JSON-RPC
laws. They use actual server configuration, the production selected-SFC owner,
real original File queries and production service registration:

- original local shadowing, every hoisted-var declaration site, implicit event
  identity/empty definitions, exact entity/CRLF/non-BMP UTF16, half-open holes;
- repeated unchanged File/Descriptor/selected/HandlerBody identity, independent
  original/selected owners, sticky script/style/encoded-For refusals;
- both-family close/configuration/source-change retirement, fresh-owner
  preservation, dropped queued requests, real worker unwind/project drop;
- shared live capacity and mailbox Busy, retry after actual owner exit, full
  JSON-RPC result/error arrays and actual didChange/didClose behavior.

Local source review, rustfmt, seven pure storage/summary laws and exact census
generation passed under the no-local-build constraint. The pure workflow
contract could not import the absent local yaml package; no installation or
local Rust build was attempted. The existing mandatory native-navigation hosted action
must run the actual affected source/RPC laws, minimal compilation and strict
Clippy; its protected full recipe remains mandatory. Exact-head source Actions,
full queue suites, unchanged all100 x3 instruction ceilings/ratchets and the
literal protected merge are required before acceptance. No skipped law,
API availability or private source body receives runtime credit.

The source refresh onto actual main preserves incoming JSX/profile/source laws
and the central paragraph; only the source-qualified Maestro migration shard
changes in the canonical inventories. TODO: publish this consumer, execute the existing hosted action and
track its full protected queue/actual merge. No default switch, cross-file graph,
whole dialect/product or #6883 fix-history closure is claimed.
