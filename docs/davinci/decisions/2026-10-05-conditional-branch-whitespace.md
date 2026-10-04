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
existing native Vapor parity law retains its branch and comment controls and
adds the original inline reporter plus an outside-space control. Native DOM,
Vapor and SSR consumers must agree with the corrected compiler; this repair is
not a feature-stage completion claim.

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
comparisons. AST assertions separately cover the single Suspense child, comments,
outside space and the existing invalid-adjacency error.
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

The issue reporter is attributed as `ubugeeei <ubuge1122@gmail.com>`, verified
from public commits authored by the issue's GitHub identity.
