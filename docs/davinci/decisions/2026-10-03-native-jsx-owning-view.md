# Original-owning JSX decisions

Issues: #6839 (L3), #6840 (L4), #6829 (dialects).

The first JSX consumer takes the genuine completed `JsxFile` by value. Its
private construction retains the original parser `ProgramObservation`, complete
same-Program File, comments, spans, scopes and resolved references. The new
`vize_l3::jsx::build_jsx_decisions` visits the existing scalar preorder records;
it does not walk or clone the AST, parse again, serialize an intermediate tree,
or accept an independently supplied File, node id, observer, source/profile flag
or AST pair. `vize_l2_to_l3` reexports that sole producer.

The completed `NativeJsxAnalysis` owns this lower owner and its ordinary decision
Vec. Each read-only decision borrows the same owner's node. There is no public
constructor, mutable table, foreign-node query or owner extraction. Moves and
Vec growth preserve original arena body/source identity. A rejected projection
retains the complete original owner and typed whole-file-span issues; it exposes
no partially admitted decision stream.

This slice classifies intrinsic tags, genuinely resolved simple component roots
and static members, selected boolean/string attributes (`id`, `class`, `title`,
`disabled`, `data-*`, `aria-*`), text and empty comment containers in authored
order. Closing component names deliberately have no evaluation/reference.
Original text and attribute values remain undecoded parser values. There is no
Vue whitespace/entity normalization or target output admission here.

Fragments, spreads, directives/slots, other attributes, dynamic containers and
ordinary non-JSX expressions refuse the whole projection. This deliberately
includes incidental primitive initializers: there is no whole-script transform
authority. A component imported by the actual Program or bound in its genuine
function scope can be classified; guessed names and unresolved lower Files
cannot mint a view. TSX source follows the same original owning provider, without
granting type erasure or TS module output.

The new source was executed privately against immutable repaired owning provider
`94c2e5251984894ce7560ca42f50a41309f01df9` and its retained matching ordinary
cached parser/L0/L1/AST ABI. Eight new laws check authored role/value order,
original owner moves, real function-scoped JSX/TSX resolution, equal-byte foreign
owners, whole-view typed refusals, non-JSX expression refusal, Unicode nonzero
origin spans and lower unresolved sealing refusal. Whole current slice L3 and
the complete L2-to-L3 converter compile with production `clippy::all` plus all
thirteen deny lints. Those saved receipts remain original `2bd38de`/`94c2e525`
execution, independently reviewed as source-clear. The later `b98e7706` replay
followed genuine provider `af1388a9` on signed actual main `d36c4874`; its rebuilt
ordinary L1 and matching whole L2, eight laws and strict/privacy/source receipts
remain preserved with their original source and dependency qualifications.

The preserved private `1bc96c49` replay follows owning provider `ab9fce69`
on signed actual `dd05d54b`. Its literal accepted-main `5f66f931` merge was
clean and source-clear. The newer actual `21f3e560` adds genuine native-template
decisions and their L1 dev oracle: literal probes found central and L3 dev-dependency
conflicts. The preserved `f298c90c` replay follows corrected genuine provider
`86e824b1` directly on that authenticated main. It retains the incoming
`NativeTemplateDomAnalysis`, all existing registration/source/census clauses,
and the L1 dev dependency alongside its two parser/span dev oracles.

All 153 whole L2 source rows remain exact `ab9fce69`, so its retained rebuilt
library and 185 laws, including 18 NativeFile-prefix laws, are source-equivalent
inputs, not rerun results. Genuine whole L1 tree `3e9470ce` matches retained
`d5646383`. Fresh whole L3/converter production all-thirteen strict, the same
eight original-owning laws, three privacy failures plus a public positive, and
all source gates passed against this new source, including 74 selected tooling
laws and the complete official Node Croquis check of 34 committed files. JSX production and fixture
bytes are unchanged. The compatible `ea5458f8` parser and OXC/Shared/span cache
remain historical ABI inputs: actual vendor parser source has since changed.
This proof grants no current locked-parser/Cargo or hosted acceptance. Original
`1bc96c49`, `ef75fbb7`/`0bb33425` and all earlier execution/conflict receipts
remain preserved separately. Current public transport, exact Actions and queue
acceptance remain unfinished.

The preserved `aa6e9fe9` continuation follows genuine `51b7d49d` directly on actual
`9f42382f`. It preserves the incoming selected static-HTML body materializer,
all current native-template source, the L1 dev oracle, central clauses and exact
current storage/census rows. Its five owned Rust source/fixture paths are exact
`f298c90c`; only the existing registration/dev-oracle/census deltas and truthful
current-source documentation are composed. Root authorized this bounded
three-path correction without another local provider build or parser reproduction.

The current whole L2 and L1 sources differ from the historical `86e824b1` inputs.
The earlier 185 laws, whole-source identities, matching libraries and fresh
L3/converter execution therefore remain historical proof with their original
qualified source/ABI. They certify no current `9f42382f` locked closure. Current
local evidence checks only owned-byte conservation, source caps, formatting and
the official inventories. Fresh configured Rust and the top full Actions campaign
must validate the actual current locked dependencies before queue readiness.
No current compatible lower library, new runtime/native output or hosted
acceptance is asserted. Original `f298c90c`, `1bc96c49` and all earlier receipts
remain intact.

The preserved `aa3b86de` census continuation directly follows genuine `8e786ad8` on
actual `69ee3233`. Its five owned Rust paths remain exact `aa6e9fe9`; incoming
reference-query counter rows and measured L2 Scope/Span census are preserved.
Only this necessary owned metadata composition, current inventories, caps,
formatting and byte/tree conservation are checked locally. No compiler, parser,
provider or runtime build is repeated. One bounded source/transport peer and
fresh configured Rust/real100/top-full Actions are required before readiness.
Earlier whole-input identities, ABI libraries and execution remain historical;
this continuation grants no current ABI/runtime proof or new capability.

The current necessary continuation directly follows genuine `0bee8edc` on
actual `0fa168a6`. The incoming SSR consumer raises L3 Span use to 80; these
unchanged JSX files add three, so the official current census must measure 83.
All incoming SSR/For storage rows and reference-query counter facts are retained.
The five owned Rust paths remain exact `aa3b86de`. Only inventories, caps,
formatting and conservation are checked; no compiler/parser/provider/runtime
build, archive, new capability or current ABI proof is added. One bounded peer
and fresh Actions validate transport and the actual current locked closure.
The later doc-only main `3b5` change will be checked in the publication union,
without source-age reparenting. All earlier packets remain immutable.

Parent PRs [#7479](https://github.com/ubugeeei-prod/vize/pull/7479) and
[#7480](https://github.com/ubugeeei-prod/vize/pull/7480) actually merged. Their
historical native Stack #7481 membership/receipts remain intact. Root removed
only unmerged #7486 from that historical Stack and retargeted it to fresh main;
the corrected provider head update is guarded before registering a new actual
native Stack ordered #7486 then this L3 child. No individual auto-merge is
permitted; root queues only a contiguous exact-head green prefix and verifies
each candidate and actual merge. PR #7503 was published with genuine parent
#7486, and root registered native Stack #7504 in that order. The existing Stack also preserves external higher layers #7519 and #7538.
Only the two owned layers are candidates for this guarded source correction;
root retains all current membership and queue authority. Accepted requests and local source proof do not
close those gates.

Exact `ef75fbb7` required and full campaigns failed on the stale L3 Croquis
ledger: `Span` uses were recorded as 62, while the actual source scanner found 65. Earlier selected local tooling checks did not cover this matrix oracle.
The official generator correction changes only that row; the original failed
campaigns and receipt `cd0ba9e4` remain preserved. Their actual eight laws,
privacy, full Rust, coverage and measured 100-by-three successes do not accept
the successor. The current replay includes the genuine census check and retains
the complete incoming File prefix, rather than carrying the earlier conflicting
registrations or central/census hunks. No assertion, budget, allowlist or
execution policy is relaxed. New current-head required/full and measured 100
gates remain mandatory. The observed server-restacked `3eb6994e`/`5fdd472f` and later
`2f473328`/`ddf7bd62` heads remain separate preserved epochs. Source
The corrected `0bee8edc` provider and this true child remain private until
guarded same-PR publication; older source snapshots retain their original scope. Root alone registers and queues the actual ready prefix.

TODO: genuine native-upper owning Program transfer, original expression payload,
Vue JSX entity/whitespace and component slot rules, full module/source-map helper
order, native-produced and pinned Vue 3.5.35 runtime comparisons, complete
directives/slots/spreads, hosted canonical ledgers and fresh public transport.
Whole Cargo/current-main closure, hosted full Check, all 100 unchanged probes,
protected candidates, actual merge, defaults and product history remain pending.
No output, performance, allocation reduction or hosted acceptance is claimed.

## Current main integration

The existing L3 PR follows the replayed L2 provider on current main in native
Stack #7504, position 2. Actions identified its prior tooling failure as the
missing L3 consumption-matrix update: the authored Span count changes from 62
to 65. Regeneration repairs that artifact without changing tests or production
semantics. Fresh exact-head PR checks and protected merge-queue validation remain
required; the historical private-lane restriction describes only those earlier
receipts, while this integration lane owns public delivery through actual merge.

The queue conflict repair retains the incoming selected-template L3 provider,
its L1 test dependency and all central clauses, alongside this JSX owner and
its parser test dependencies. Consumption artifacts are regenerated from their
combined sources. Both JSX queue entries were removed together before a candidate
was minted; publication follows the preceding prefix actual merge into main and
requires a fresh exact-head PR campaign before native Stack queue resubmission.

The current public union follows literal main `8501146f` and its genuine
replayed L2 provider. Historical private transport restrictions above describe
those epochs only. The incoming locked fork parser remains the actual oracle;
all incoming inventories survive, and reviewed JSX rows sit beside the L3
section to avoid unrelated append conflicts. Fresh exact-head Actions and
protected candidates must pass before actual merge receives acceptance credit.
