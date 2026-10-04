# Original JS/TS module DocumentLinks consumer design (2026-10-04)

Tracking: [#6871](https://github.com/ubugeeei-prod/vize/issues/6871),
[#6872](https://github.com/ubugeeei-prod/vize/issues/6872), and
[#6883](https://github.com/ubugeeei-prod/vize/issues/6883).
The [paired issue drafts](./2026-10-04-native-module-document-links-consumer-issue-draft.md)
are unposted. The original PRIVATE DESIGN baseline is preserved below; root
authorized private source preparation after whole 4d76 and focused440 review.
The source supplement remains unexecuted/unpublished and grants no acceptance,
standard-route change, or native/history credit.

## Exact source and ready operands

This design branches from physical prerequisite
`2d3e6dc21b45a13756ad0cbf4da81934c65938bf`, a genuine child of context
`90fcfb246bcbc514916ae8270148e0b7caa133a3`. Both preserve literal release-main
`caee1656927e1232f4aaabff2328246decb8bc58`. Native Stack #7783 is the real
#7771 -> #7782 dependency. Its fresh source/protected acceptance and actual
merges are separate from this design. Prior ec68/0bc source acceptance does
not qualify this future consumer.

The older private design `c5044a081a` remains preserved. Its original owner
correction still applies. Its `../`, canonicalization, and required watcher
coverage proposal is superseded by the narrower actual physical provider;
none of those proposed capabilities is granted here.

| Actual source under `crates/vize_maestro/src/`        | Genuine authority and limit                                                                                                                       |
| ----------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------- |
| `source_project/navigation/retained.rs`               | One original Program parse and sole File producer; original arena, syntax, File, snapshot and lines survive the worker mailbox.                   |
| `source_project/navigation/{cache,profile,worker}.rs` | Actual Document language/options, same physical snapshot cache, bounded live workers and owned messages; Names/selected families remain separate. |
| `source_project/project/host/modules.rs`              | Actual owning server context capture; bare/shared DocumentStore gives typed host refusal.                                                         |
| `server/state/module_links.rs`                        | Real applied primary root/checker/load origin, same-session checked generation, actual mutation and termination.                                  |
| `server/state/module_links/physical.rs`               | Same-session received nonempty file-event epoch, not acknowledged watcher coverage.                                                               |
| `source_project/project/tracked/{modules,targets}.rs` | Genuine ready-result source/context publication and source/context/event target observation/recheck/publication.                                  |
| `server/module_target{,/unix}.rs`                     | Nonempty explicit `./` batches, pinned Unix nofollow/nonblocking component handles and regular-file identity; non-Unix typed refusal.             |
| `server/native_navigation.rs`, `server.rs`            | Existing explicit opt-in native endpoints; standard document links remain legacy.                                                                 |

L2 `davinci/vize_l2/src/lang/js/file/module_source{,/read}.rs` is actually merged
original Module source authority from #7711. There is no Maestro consumer call
on this base. `FileArtifact::original_module_sources` owns a checked original
ProgramInput and borrows its same completed File. `for_each` requires discarding
ALL visited prefix on any error. Original statement-site access is O(1); it is
not another body walk, token scan, neutral projection, or request discovery.

## Bounded public contract

Propose explicit `vize/nativeModuleDocumentLinks`, accepting ordinary
DocumentLinkParams and returning an owned source-ordered `Vec<DocumentLink>`.
This endpoint opts into point observations. It does not replace
`textDocument/documentLink`, add resolve capability, or reinterpret settings.
Only actual `javascript` and `typescript` Documents with original explicit
Module/default ProgramOptions qualify. Filename suffixes never select profile.
JSX/TSX, definition, Script/Unambiguous, SFC, selected-template, Names, recovered
syntax and incomplete File observations refuse. Minimal experimental builds
keep actual context-unavailable refusal; they do not enable `native` implicitly.

Static side-effect/binding imports and source-bearing named/all reexports use
only genuine OriginalModuleSource occurrences. Type-only forms navigate files
without claiming type/runtime resolution. Comments, arbitrary string values,
quoted imported/exported names, require calls and local/default exports add no
module source occurrence. Any actual dynamic import or empty-source export gap
refuses the whole response. Empty Program remains the exact provider refusal.

An admitted nonempty local-only Module may return `[]`, but only under its same
ready source and live server-context publication. This empty result grants no
importer-parent, physical target, URI-resolution or watcher authority. No empty
batch is passed to the physical provider, whose EmptyBatch refusal stays exact.
Unavailable or stale source/context never becomes empty success.

For nonempty operands, use the existing physical policy exactly: decoded `./`
requests with named case-sensitive js/jsx/mjs/cjs/ts/tsx/mts/cts/vue extensions.
No parent `../` component, implicit extension/index, alias/package/absolute
request, URL, percent/query/fragment/backslash spelling, empty component, NUL,
CR or LF is admitted. Other spellings receive only the existing provider's
actual policy; this design does not claim a broader control-character rule.
Original escaped spelling is eligible only if its decoded request qualifies.

The ready result supplies its exact file URI, with no query/fragment and only
local authority, and context supplies the actual primary root. Root/source
parent/target components must meet actual nofollow directory/regular-file
observations. An open importer need not exist on disk, but its parent does.
Other primary-root-external documents refuse for a nonempty target batch.
No target source is parsed, and no TypeScript/Node module-resolution claim is
made. A target's resource extension does not grant target language admission.

## Worker-local original view and owned response

After the existing producer consumes its first ProgramInput and finishes File,
mint a fresh O(1) ProgramInput::checked from the SAME retained admitted syntax,
SourceBlock and unit 0, then move it into file.original_module_sources(input).
Retain the view or a private typed admission refusal locally across the
mailbox loop. Keep genuine fresh ProgramInputError separate from genuine
ModuleSourceError; never manufacture one error type for the other. Failure of this new read must not retire valid definitions or
references. Other worker families refuse the new command. No extra parse,
Program walk, pipeline stage, resident operand index or Send/Sync owner occurs.
Scope the view/query/mailbox lifetime to end before dropping original File and
syntax, with the arena last; response ownership never changes that order.

The proposed `retained/module_links.rs` owns query-only collection. For each
receipt it copies decoded_request into an owned response operand and derives
Range from quoted_span using the SAME snapshot's existing LSP line starts and
strict UTF-8-to-UTF-16 coordinates. Both original quotes are included. Raw
escapes, continuations, CRLF and non-BMP prefixes remain authored geometry;
decoded length does not determine the range. Keep original quoted/declaration
spans, kind and namespace in private authored proof, never as caller authority.
Only this original worker can construct its private owned operand response.

Run for_each to normal successful completion. Callback coordinate failures
record a private first error; the callback cannot signal a Result. Any callback
failure or later provider error drops all staged operands. Genuine L2 unit,
span and kind survive typed mapping. Sort only the owned response by actual
(quoted.start, quoted.end), with an explicit comparator. Separate declarations
with equal requests remain distinct; one multi-specifier reexport appears once.
There is no request-text search or request-based deduplication.

## Small ready-value plumbing prerequisite

Actual ProjectQueryResult<T> has publication methods but no read of its owned
T. Filesystem observation needs the original prepared decoded operands BEFORE
final publication. Do not detach them through mutable out-of-band response
captures or publish the response merely to retrieve candidate strings.

The smallest proposed addition is a private read-only borrow of this result's
OWN prepared T, via SourceQueryResult and its ProjectQueryResult wrapper,
restricted to source_project internals. Reading owned prepared data is not a
current-source, target or publication grant. Preserve private fields and the
original query/cancellation lease. Consumer response objects remain private;
no public caller may inject a prepared operand, source URI, span or snapshot.
The exact API/lifetimes require root/peer review before implementation.

Actual SourceQueryProject context capture has no source/cancellation guard.
Add only a private query-bound context capture on the ready ProjectQueryResult:
its original result.with_current(host.documents(), ...) calls the SAME retained
host.capture_module_link_context inside that source guard. This establishes
source/cancel -> context order without FS/await or an alternate host. Capture
after the real worker reply, before target IO. An unavailable context must not
escape before that original source check. Equal copied contexts or capturing
directly from ServerState cannot replace this query-bound path.

This borrow lets the consumer derive temporary &[&str] from the same ready
owned response, call ready.observe_module_targets(context, candidates), then
drop that temporary borrow before consuming ready for final publication.
Existing methods perform source -> context -> event checks before/after IO;
all IO and awaits stay outside DocumentStore/module guards. No value clone,
resident duplicate index or snapshot substitution is necessary.

## Whole-response algorithm and publication

1. Begin the real SourceQueryProject query for the original Document URI and
   run the original worker request under its registered cancellation pair.
   The ready result owns complete operands or the genuine typed refusal.
2. Capture current context from THIS ready result under its original source/
   cancellation guard, then its same retained authentic server host. No caller
   path, default, Names ticket, legacy resolver or other ServerState supplies
   context. This same-query guard precedes even a context-unavailable error;
   source validation remains mandatory after the mailbox reply.
3. For zero operands, consume the same ready result through
   publish_with_module_link_context and return only the owned empty vector.
   The callback is synchronous and cannot reenter source/context/cache APIs.
4. For nonempty operands, read candidate references only from that ready value,
   observe the whole batch, prepare complete owned response data, then recheck
   all retained root/parent/final component identities through fresh traversal.
5. Require equal target/operand lengths before any zip; mismatch is whole typed
   refusal. Preserve input order and every occurrence. No truncate/partial link
   array may result from zip or a late failure.
6. Consume the SAME ready result and its RecheckedModuleTargets through
   publish_with_module_targets. The guarded callback only attaches exact owned
   URLs to prepared ranges and returns the entire Vec<DocumentLink>.

Each DocumentLink has exact Range and target, tooltip/data None. Pinned lsp-types 0.94.1 source omits tooltip/data when None; authored entire
wire envelopes must retain that omission, never a present JSON null. This is
source-only serialization qualification, not actual RPC execution. No resolve token or
synthetic target is added. The callback performs no FS, await, state lookup,
worker wake/retire, store reentry or user code. URI/range allocation may be
prepared before it when lifetime-safe; no arbitrary size-dependent host work
is presented as O(1).

Original source cancellation/version/revision/language/close/reopen/rename and
same-host identity remain real final guards. Context A->B->A, overflow, unwind,
shutdown/exit, foreground/input EOF/error and retained background Arcs remain
actual provider retirement. Received-event ordering invalidates the batch;
unknown watcher coverage never becomes a covered capability. Fresh traversal
proves final per-component observations, not global filesystem atomicity or
any future unchanged path/content promise. No initialize/registration handshake
or file watcher is added by this consumer.

## Typed failure and hosted proof

Keep navigation/source/provider/coordinate errors separate from context/target
IO errors; do not stuff non-cloneable IO state into the existing Clone/Eq
NavigationRefusal enum. A distinct module-link error may retain genuine
ModuleSourceError and ModuleTargetError. Delegate unchanged existing navigation
failures to their current RPC mapper via a child module or narrow visibility;
copying that mapper into a second divergent path is unnecessary. Cancellation maps RequestCancelled;
actual source/context/event supersession maps ContentModified. Unsupported
module/policy/platform/host and target IO get explicit typed native refusal,
never [] or legacy fallback. Freeze exact added codes/messages before source
implementation review; no number is reserved by this private design. The real
context-publication consumer must also remove the now-obsolete conditional
unused-import expectation for ModuleLinkPublicationError when that outer
reexport becomes used; no suppression is retained to disguise new usage.

Planned, unexecuted complete fixtures cover:

| Family          | Complete original proof                                                                                                                                                                                                  |
| --------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| JS/TS positives | Side-effect/binding/type imports, named/all/type reexports, actual files, exact public object/envelope and source order.                                                                                                 |
| Occurrences     | Equal requests in separate declarations, deduplicated multi-specifier reexport, no comment/name/local-string invented occurrence.                                                                                        |
| Geometry        | Both quote kinds, raw escapes, every supported line continuation, CRLF/non-BMP prefix, full authored UTF-16 ranges and actual target URIs.                                                                               |
| Whole refusal   | Empty Program, dynamic/empty export gaps, foreign/profile/recovery, late lone surrogate/coordinate/policy/missing target; zero prefix publication.                                                                       |
| Context/source  | Applied nondefault config/root/load origin, primary scope, minimal/bare/foreign, cancelled/stale source before context-unavailable, source and config ABA, cancellation after ready/reuse and final guarded publication. |
| Physical/events | Real nofollow files, parent/root replacement, FIFO/symlink/directory, late target replacement, received events and checked epoch; no covered/future-path assertion.                                                      |
| Reuse/cleanup   | Definition/reference/link queries share the same original Program/File/parser identity and worker; no extra parse on context-only mutation; all resources retire on capacity/cancel/unwind.                              |
| Transport       | Real initialized server complete RPC controls, held reply then EOF/drain/retained Arc, unchanged standard links and original legacy sessions.                                                                            |

Use the existing source-built affected native-navigation and protected full
Actions, unchanged original inputs/oracles and all 104 instruction ceilings/
ratchets. No local build/install or extra manual campaign is proposed. Root/peer
must first clear the ready-value API and whole design, then any authorized
private implementation freezes production and all complete expected laws before
publication. Use a true dependent Stack only with the then-real parent heads;
otherwise branch fresh actual main after the prerequisites literally merge.

The d473 private predecessor is preserved. Its independent source review
identified the unguarded early context capture; this narrow successor binds
capture to the genuine ready source/cancel guard and updates planned refusal
ordering. This is design correction only, with no source execution claim.

This design does not close #6871/#6872/#6883, migrate LSP7, supply full module
resolution or choose the standard/native default. Context/physical provider
source acceptance, protected delivery, this consumer's future source/runtime,
fix-history acceptance and default replacement stay distinct.

## Private source preparation and frozen response contract

The initial implementation worktree `feat/native-module-document-links-20261004`
and frozen `db10d903f45aebef50be3c796769e63eead36812` remain preserved. Its strict
successor uses `feat/native-module-document-links-strict-20261004`, branched from
that same genuine2d3 ancestry; earlier design/source worktrees remain preserved.
Parent7771/physical7782 are now actually signed-merged50c/69c with independently
qualified current full104×3 and complete normal/minimal/strict proof, recorded
in their companions and paired terminal comments. Their acceptance does not
qualify this new consumer. Fresh actual-main ancestry is required before any
later authorized source publication.

The worker slice authored by the existing parallel agent is frozen6970ff4 and
transported without byte changes as8337025. It remints one checked ProgramInput
from the same retained syntax/block/unit after the sole File finish. A private
RetainedSources owns that sealed OriginalModuleSources view and one borrowed
same-factory File pointer; even valid zero operands check this original join.
It adds no table, AST walk, parser call, pipeline stage or resident duplicate
operand index. Each command stages one owned response vector, discards every
visited prefix on any provider/projection/cancellation error, sorts by original
quoted span and collapses only repeated rows of one original occurrence.
Distinct equal requests remain distinct. The whole view/query scope ends
before File/syntax/arena destruction.

SourceQueryResult and ProjectQueryResult add only an internal immutable borrow
of their same ready owned value. A new ready-context method executes capture
under that original source/cancellation guard. The consumer uses actual worker
operands, then context, nonempty physical observation/recheck and same-result
whole publication. Valid zero and typed read refusal use source+context
publication, never a physical EmptyBatch or an unguarded early response.
Only `vize/nativeModuleDocumentLinks` is registered. The response is a complete
Vec<DocumentLink> with original quoted UTF-16 ranges, actual point target URI,
and pinned omitted None tooltip/data. Standard documentLink/resolve/capabilities
remain unchanged. No payload span, decoded string or externally paired File
becomes authority. Native Unix policy/UnknownCoverage remains exactly the
accepted provider's bounded contract.

| Actual refusal                                         | Complete added RPC code/message                         |
| ------------------------------------------------------ | ------------------------------------------------------- |
| Original ProgramInputError                             | -32013 / Native module input refused                    |
| Original ModuleSourceError                             | -32014 / Native original module sources refused         |
| Owned target cardinality mismatch                      | -32019 / Native module target cardinality refused       |
| Unavailable/foreign context                            | -32015 / Native module-link context unavailable         |
| Superseded/updating/retired context                    | ContentModified / Native module-link context superseded |
| Target event/foreign snapshot/physical identity change | ContentModified / Native module target superseded       |
| Unsupported platform                                   | -32016 / Native module target platform unsupported      |
| Other actual path policy                               | -32017 / Native module target policy refused            |
| Original operation/path/IO refusal                     | -32018 / Native module target unavailable               |

Original navigation/source refusals delegate to the existing mapper; no copied
semantic mapper or IO state is injected into its Clone/Eq enum. Original query
cancel wins over unavailable context. Full original errors remain private
development data, while wire errors contain only their exact code/message.

The initial db10 freeze has58 unique authored, unexecuted laws:26 worker,5 ready-value/context,
5 whole consumer,7 ready original-operand publication and15 complete RPC laws.
Its anticipated Unix source execution is55 normal and37 genuine minimal with
overlap; two feature-specific alternatives and one non-Unix law explain the
unique total, not extra Linux passes. These counts require actual Actions.
Whole vectors cover original imports/type/mixed/string forms, source-order
occurrences, Unicode/CRLF/continuations, genuine File/profile/empty/dynamic/read
refusals, foreign zero views and private storage, real file creation/replacement/
symlink/directory/late-prefix refusal, actual nondefault checker load origin and
config/root ABA, queryA cancellation despite queryB's identical cached snapshot,
source→context→event priority, capacity/reuse/drop/unwind and complete wire
lifecycle. The EOF/error/drop law parks polling of the actual registered RPC
while its original worker is held, observes the real input termination before
tower-lsp drains, then requires the whole native refusal and unchanged standard
null frame with retained state Arc. It does not pretend that this pending wire
request already reached its physical-ready phase; separate same-worker ready
publication laws cover that phase. The cfg(test)-only pause adapter retains no
new product state or request authority.

Root and an independent whole-source peer must review every production/law
blob, exact API/privacy/lifetime and full authored frames before publication or
Actions. This private preparation performs no build/install/runtime/campaign.
Only then may genuine fresh-source normal/minimal/strict and protected full104
qualify it. Broader resolution/packages/aliases/../, stronger watcher coverage,
standard routing/history/default closure remain unfinished.

## Original strict-module receipt correction

Whole independent source review of db10 found one concrete omission. The pinned
original decoder admits legacy literals with empty syntax diagnostics while
retaining `AdmittedProgram::has_legacy_literals()`. Neutral completed File and
`original_module_sources` do not themselves reject that scalar. An original
`import '\056/child.ts';` or unrelated `010` could therefore reach this consumer.

The privately authorized successor checks that SAME original scalar once, before
its fresh checked input, and returns existing `NavigationRefusal::Syntax`. No
source scan, AST walk, parser diagnostic, admission policy or neutral provider
changes. The existing ordinary definition/reference worker still uses its
original complete File. The initial6970/833/db10 blobs and unexecuted counts
remain historical; the correction changes only this consumer admission and
adds complete source laws without editing any earlier expected body.

Two additional authored laws cover raw octal import, all seven original legacy
number/string controls, modern numeric/hex/Unicode escapes, raw escaped backslash
text and comment text. The worker checks exact decoded request/quoted range and
unchanged old definitions/references; the actual registered RPC law freezes the
whole existing Syntax error and complete successful frames with real files.
The current total is60 unique unexecuted laws:27 worker,5 ready-value/context,
5 whole consumer,7 ready publication and16 RPC. Anticipated Unix execution is
57 normal/38 genuine minimal with overlap; feature alternatives and nonUnix
stay distinct. Final corrected whole source/root/peer review, fresh actual-main
publication and hosted normal/minimal/strict/protected acceptance remain required.

## Genuine publication replay

Root and independent whole-source peer clear corrected `0ac280ba3e`, with all
39 blobs and60 unexecuted laws retained. A new isolated publication worktree
`feat/native-module-document-links-main-20261004` starts on literal actual main
`d71d8398187dfbee63c26f1e4f2946fbe9fe38f9`. It applies only the consumer delta
relative to genuine2d3; actual signed50c/69c parent source and all incoming
decisions remain. Narrow incoming-union and equivalent two-line successor3060
reviews are CLEAR. Independent Draft and automatic normal/minimal/strict/source
Actions require actual acceptance; no runtime/protected credit transfers. Paired published decisions: [#6871](https://github.com/ubugeeei-prod/vize/issues/6871#issuecomment-5979248254), [#6872](https://github.com/ubugeeei-prod/vize/issues/6872#issuecomment-5979248418) and [#6883](https://github.com/ubugeeei-prod/vize/issues/6883#issuecomment-5979248589). Initial53457 affected Rust builder111421601672/run37197221270 fails E0308 at the two new strict-law DocumentStore calls (CompactString versus owned String), before native laws execute. Lossless to_string conversion only repairs fixture transport; original input/expected bytes and production/API stay exact. Full failed logs remain; fresh normal/minimal/strict and current protected acceptance are required. Source9b961/run37197893255/builder111423544092 actually passes207 SourceProject,57 complete native RPC,107 normal module-link and61 genuine no-default laws plus minimal check; final strict Clippy fails only the two introduced to_string calls banned by the existing policy. Borrowed as_str().into() replaces those fixture conversions without suppression or altered input/expected/API. Whole source remains red; fresh final strict/source/protected acceptance stays mandatory and non-Unix remains unexecuted.
