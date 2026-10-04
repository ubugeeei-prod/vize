# Conditional branch whitespace (#7831)

The [reported compiler regression](https://github.com/ubugeeei-prod/vize/issues/7831)
leaves whitespace-only siblings between successfully joined `v-if`, `v-else-if`
and `v-else` branches. In Nuxt's production root, those siblings become extra
Suspense slot roots and the app renders an empty comment. Inline content also
gains unwanted spaces.

## Decision

Remove only whitespace-only siblings between the previous conditional and a
successfully joined branch. Defer the vector removals until the active child
borrow has ended, and resume traversal at the shifted child index. Keep the
existing comment ownership, unrelated text, outside whitespace, branch keys,
expression processing and non-adjacency diagnostic. This correction belongs to
the existing compiler; it adds no stage, serialization or native readiness
credit. Instruction ceilings remain unchanged.

The existing native L1-to-L2 lowerer also saved and re-emitted the same branch
gaps to match the old compiler. Remove that preservation and its per-chain
vector, while keeping `drop.branch-gap` provenance and outside whitespace. The
reviewed storage ledger lowers this file's `allocVec` bound uses from 14 to 12:
the removed scratch vector's type and constructor are the only deleted uses.
existing native Vapor parity law retains its branch and comment controls and
adds the original inline reporter plus an outside-space control as their
existing explicit `Legacy(Element)` routes, without extending admission. Native DOM,
Vapor and SSR consumers must agree with the corrected compiler; this repair is
not a feature-stage completion claim.

The lint facade presents authored nodes rather than rendered roots. Its L2
projection now reads gap text from its existing L1 surface and shares the native
lowerer's whitespace normalization, including inherited `<pre>` and Unicode
whitespace. This keeps every original differential trace and the pinned battery
census intact after compiler regions stop emitting gap text. The original inline
reporter, comment gaps, nested `<pre>` and Unicode gaps add independent controls;
no new pipeline stage, legacy-provider dependency or census relaxation is added.

## Preserved evidence

The dedicated
[`conditional-branch-whitespace.manifest.json`](../../../tests/_fixtures/differential/compiler/conditional-branch-whitespace.manifest.json)
pins the exact Nuxt root and both compiler-only reporter templates, plus the
reported Nuxt 4.5.2 configuration, app and page. It also pins independent expected
HTML for all five Nuxt root branches and the minimal Suspense, inline,
surrounding-space, multiple-chain and nested-chain controls.

The existing CLI compiler manifest has a single fixed SSR adapter. This runtime
corpus uses its own manifest and the source-built public DOM and SSR compiler
APIs instead of changing that unrelated CLI contract. The Rust regression in
`crates/vize_atelier_sfc/tests/conditional_branch_whitespace.rs` supplies complete
generated modules to the real Vue production runtime and server renderer. Both
whitespace strategies exercise 18 states, for 36 DOM and 36 complete SSR
comparisons. Core parse/transform AST assertions separately cover the single Suspense child, comments,
outside space and the existing invalid-adjacency error.
Those AST observations use the product's DOM tag classifiers; generic parser
defaults classify `<component />` differently and cannot be its AST oracle.
The original self-closing `<div />` is retained with its existing recoverable
diagnostic code, complete message and authored span pinned in both compilers.

The pinned Nuxt 3.19.3 module-build fixture now requests `whitespace: "preserve"`
and keeps its existing production SSR, hydration, click and route assertions.
Its native binding must be built from the candidate source before its results
can qualify this fix; the source-binding companion work owns that requirement.
The preserved Nuxt 4.5.2 source is reporter evidence, and this change does not
claim to have executed a full Nuxt 4 production build.

## Qualification

Local checks cover formatting, digest pinning and runtime-harness controls with
the available Vue 3.5.35 installation. They do not qualify candidate compiler
behavior. Exact candidate Actions must execute the Rust regressions with the
repository's pinned Vue 3.5.43, and the genuine candidate Nuxt binding must pass
the existing build, SSR and browser checks. Full protected suites and unchanged
instruction ceilings, actual merge and release remain required. The PR stays
draft and outside the queue until the batch owner's concrete review.

Actions [37225787003](https://github.com/ubugeeei-prod/vize/actions/runs/37225787003)
at source `385203ee8a7b440286f250d8188002739862e374` executed and passed the
complete 36-state production DOM/SSR runtime law. That run still failed the
authored lint view, storage ledger and two test assumptions described above;
their correction requires fresh Actions and receives no transferred acceptance.

The issue reporter is attributed as `ubugeeei <ubuge1122@gmail.com>`, verified
from public commits authored by the issue's GitHub identity.

The prospective Stack projection onto main `688da7cc` conflicted only because
this record and the accepted pnpm setup decision appended to the same physical
paragraph. Retain that new main entry and move the complete compiler decision
to an existing stable paragraph, preserving all other source bytes and the
350-line canonical record. Fresh exact-head source Actions remain required.

The authorized actual-main replay follows #7840's signed `5d879b27` merge.
Real-rebase all five child commits onto preserved parent `08183052`, retaining
the full incoming #7828, parent #7838 and compiler #7831 canonical clauses.
All 27 owned production/fixture blobs and each original author/message/trailer
remain exact. Previous proof IDs retain their original source; the new actual
composition requires fresh source Actions before highest-only Stack admission.
