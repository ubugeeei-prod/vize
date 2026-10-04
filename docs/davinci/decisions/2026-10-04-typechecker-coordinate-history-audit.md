# Original type-checker coordinate history audit

Issues: [#6879](https://github.com/ubugeeei-prod/vize/issues/6879),
[#6849](https://github.com/ubugeeei-prod/vize/issues/6849).
Private audit on actual main `097c09631acc69eb2b2793cfcb8afdddb3844fc3`.
No public docs-only PR, fixture admission or checker API change is proposed.

The ledger remains **271 rows / 263 not-admitted / 31 classification-pending**.
The current shared manifest has 27 projects in ten registered packs, all with
null native adapters. These three original fixes have no registered project in
that manifest. Existing vectors, classifications and denominators are unchanged.

The [source receipt](./2026-10-04-typechecker-coordinate-history-audit.json)
retains exact fix/parent identities, all changed-file blob identities/hashes,
the Canon/Croquis patch hash, six complete original test functions, original
project/configuration/observation/package helpers, nine full source inputs,
and exact original configuration literals. All 27 original function/helper/input
windows were independently compared with the pinned Git blobs. No execution,
expected-value recapture or title-only coverage is inferred from this audit.

| Requirement | Exact fix and parent | Complete historical public list |
| --- | --- | --- |
| Leading template TypeScript suppressions | `e199979dfb32dfd8eb9ad32a2928fb010768f239` / `dcb5bae88876fdaf5015d6499b0eb288447b9371` (#6009) | One original structural-array SFC: empty list. Both authored `@ts-ignore` and `@ts-expect-error` comments survive. |
| v-for source callback mapping | `04aedfb8e8b41dfae89fe65256e2703fc3718cf3` / `b77e0a21485b9c9fb23506ee9b30fc857e4345a1` (#3818) | Native case: exactly TS7006 `value` at 6:36; component case: exactly TS7006 `value` at 7:38; generic-slot and imported-template-prop projects: empty lists. |
| Split classic/setup script mapping | `b3c2933be5afbdb5f2df07da093f40bc7ce72187` / `8f1953c0637060efe9d1103f9426000a39238054` (#3783) | Full original `refreshGridItems()` SFC: exactly TS7006 `it` at 10:43, rather than the previous 10:9. |

All three nonempty lists have severity 1 and the exact message
`Parameter 'value' implicitly has an 'any' type.` or the original `it` variant.
Coordinates are one-based authored UTF-16 starts; the receipt independently
checks each start against its original identifier. It does not invent an end.
All original configurations use strict checking, ES2022, ESNext modules,
Bundler resolution, noEmit, strictTemplates and `src/**/*`. The original
checker uses default BatchTypeChecker options without extra flags.

The original tests return early when their runtime is absent. Their project
helper links workspace Vue/Vite/@vue when available, otherwise uses test stubs.
Their snapshot helper sorts public diagnostic tuples. Every selected project
expects zero or one row, so sorting does not hide an ordering distinction in
these six original lists. It still supplies no historical returned-order or
raw-report evidence for a larger project. The public Batch Diagnostic has
file/start/code/message/severity/blockType, **no end or related information**.
Those unavailable fields must be recorded as unavailable, never as empty.

## Ready fixture boundary and remaining bundled scope

Three separate source-bound Batch packs can carry these six exact projects,
all original source/configuration bytes and the existing complete expected
lists through the mandatory `fix_history_diagnostics::check_pack` observer.
Registration must retain the original functions/helpers/patch receipts,
actual pinned runtime identities and complete returned public results. Runtime
absence/check failures must fail. Source-built Actions acceptance and protected
delivery would be required before any new fixture credit. Native adapters stay
null, all existing expected vectors stay immutable, and no recapture supplies
new end/related or backend values. This is a concrete proposal, not registration.

The whole #6009 patch also changes guard expressions and introduces three
comment-scanner unit laws: ordinary leading comments, suppression text inside
literals, and non-leading object comments. Its one empty project alone does
not cover all of those obligations or independent unused-suppression errors.

The #3818 patch also adds original source-offset facts in Croquis, rewrites
template-only props before mapped loop emission, avoids local/global-name
capture, and deliberately withholds source mapping for unresolved generic-slot
callbacks. Native typing needs genuine contextual generic-slot/prop types;
copying that historical diagnostic omission cannot establish raw parity.
Its Vue2 elm snapshot change and broader scope consumers remain separate scope.

The #3783 patch rebases synthetic merged-script offsets for ordinary and generic
script spans and line mappings. Its content-mapper unit law and four real-world
snapshot deltas remain separate original requirements. The one TS7006 project
does not qualify all snapshot changes or a two-script native checker.

Actual merged bounded-array custody makes #6009's original declaration neutral
File-complete. It still makes the whole setup unit ineligible for template
binding exposure. #3818 additionally needs authentic callback/For/slot/props
projection. #3783 retains a complete function, original `any` annotations and a
classic type export across two original script owners; current native File
completion and `project_vue`'s multiple-script refusal are independent blockers.
No reduced primitive substitute qualifies these originals.

## Distinct original Program syntax projection proposal

`vendor/oxc_parser/src/observation.rs` already owns one complete original parser
result and mints immutable AdmittedProgram only after fatal/Flow/error checks.
It retains exact source, requested language/module/JSX profile, ParseOptions,
Program, comments and lexer receipts. This is explicitly syntax admission,
not semantic validation. `vize_l1::embed::syntax::NativeSyntax` additionally
retains actual authored/decode coordinates and an explicit Program grammar;
`parse_program_once` uses the normal pinned parser once, without a wrapper.

A **new, distinct** L4 syntax-only document could borrow that actual owner,
require a whole original default compiler-profile Module and exact source
window, and emit linked original bytes without walking or reconstructing the
Program. Its own type must carry the owner/profile/mapping and expose no
File-completion, binding/navigation, VueSetup, JSX semantic-role or template
visibility authority. Existing `ProgramProjection` and strict checker APIs
must keep their current complete-File/owning-JSX contracts and refusals.

A later explicit configured-source consumer would require authoritative real
path/kind/Module goal, unchanged source/configuration guards, actual diagnosing
options, complete raw Full reports and original UTF-16 mapping, with cleanup on
every refusal. Configured modules cannot be inferred from a suffix or guessed
filename. Source-copying would retain typed function/ambient/type-alias syntax
for the backend; it would not establish complete typed File semantics.

Independent laws would need original AST/comments/source/profile/lifetime and
move custody; copied-source/raw-Program/formatter-options/Script/recovered/Flow
refusals; unchanged current strict API refusals; full original typed-function
TS6/TS7 vectors and capability differences; related/end mapping; source/options
mutation; closure/reaping; and actual protected instruction gates. No API or
implementation is added by this proposal. It grants no two-script SFC, array
template binding, Ref unwrap, imported-graph, loaded-source snapshot, ABA or
diagnostic-snapshot coherence. Those wider prerequisites and #6879 stay open.
