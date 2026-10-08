# Repository-owned TypeScript migration

Paired decisions: [#6828](https://github.com/ubugeeei-prod/vize/issues/6828#issuecomment-6057070074)
and [#6830](https://github.com/ubugeeei-prod/vize/issues/6830#issuecomment-6057070381).

## Decision

Write repository-owned executable tooling in TypeScript. Run erasable TypeScript
with the existing pinned Node runtime; do not add a transpilation pipeline,
blanket `any`, `@ts-nocheck`, suppressed compiler errors, or relaxed fixture and
performance gates. Migrate providers together with every live executable/import
caller, then consume the same interfaces in independently reviewable slices.

Use move-only commits before type and call-site changes. Derive each move from an
explicit path map so a conflict can be reapplied to fresh `main`.

## Current inventory

The immutable inventory source is actual `main`
`e8a00fbb629d3e092e08fd474a58a01d1aeadc47`.
`git ls-files '*.mjs'` lists **765 tracked paths**, including hidden Actions.
The count includes fixture programs and compatibility witnesses; it is not an
assertion that every path can change without reviewing its byte ownership.

| Area                        | Tracked paths | Required ownership review                                |
| --------------------------- | ------------: | -------------------------------------------------------- |
| Actions bootstrap           |             2 | Host Node availability before JavaScript setup           |
| Docs scripts and previews   |            58 | Docs and Open Graph Stack owns its independent migration |
| Editor host tooling         |            12 | Packaged host, fixture and public runtime boundaries     |
| Package and example helpers |            30 | Node 22 public floor, emitted artifact interfaces        |
| Differential drivers        |            22 | Every original corpus input and packet remains immutable |
| Tooling tests and support   |           250 | Same discovered tests, assertions and full outputs       |
| Benchmark producers         |           111 | Exact runner, sampling, iterations and budgets           |
| Compatibility support       |           269 | Import, Rust launcher and workflow consumer graph        |
| Owned command helper        |             1 | Rust launcher and strict runtime contract                |
| Crate oracle wrapper        |             1 | Original Babel package/version and whole result contract |
| Fixture programs            |             9 | Original user-world bytes and executable identities      |

Inventory execution excludes `.git`, dependencies and local artifacts, while
using Git's tracked path authority so hidden `.github` files cannot disappear.
Original blobs read through `git cat-file --batch` contain 4,941 literal `.mjs`
references in 1,214 source/config files, including package manifests; 1,984
references resolve to tracked module paths. Markdown, snapshots and dependency
locks retain their own historical or external ownership. These include emitted/provider filename strings and
are a review queue, **not** a substitution map. A live module edge must resolve
relative to its consumer; Rust commands, workflow shell and manifest entrypoints
require their own executable identity checks.

## Runtime and checking contracts

The workspace pins Node **24.14.0**. Public npm packages keep Node **22** as their
floor; changing handwritten private tooling does not authorize raising it.
Node's native type stripping starts in **22.6**, and default stripping starts in
**22.18**. A major-only Node 22 test cannot prove support on Node 22.0.

Bootstrap Actions currently invoke cache and MoonBit scripts before `setup-vp`
in some jobs. Other aggregate helpers run without that setup. Qualify those
with a deterministic runtime before moving their entrypoints; do not assume
`FORCE_JAVASCRIPT_ACTIONS_TO_NODE24` changes a shell's `node` executable.

The root strict `tsconfig.node.json` currently covers tooling `.ts` tests and
Vite+ task configuration. Type-aware linting alone does not prove a strict
compiler pass. The bounded project uses the already pinned native TypeScript
7.0.2 compiler via `tools/support/typescript/check-project.ts` in `check:repo`.
A real compiler control must reject type mismatch, implicit parameters and
non-erasable syntax. Extend that project as provider graphs migrate; do not
waive remaining paths. The existing Node 22/24 Actions matrix also runs the
original comparison and source-planning tests.

## Order and remaining work

1. Migrate comparison-base and source-planning providers, their original tests,
   and all live callers atomically. Keep unknown paths fail-closed, the original
   no-renames Git comparison and every required merge-group suite unchanged.
   Add a bounded strict compiler project consumed by the existing Check path.
2. Migrate Actions bootstrap after deterministic Node identity is qualified.
   Preserve all cache keys, post-step behavior, lock/toolchain fingerprints and
   installer hashes. Keep every original whole-output and refusal control.
3. Migrate independent package/editor helpers, then benchmark and compatibility
   provider graphs in bounded worktrees. Use true GitHub Stacks for dependent
   providers and consumers; root owns protected queue admission.
4. Migrate remaining tooling tests with their real discovery and selector
   consumers. Preserve the original test count, assertions and raw witnesses.
5. Audit residual tracked `.mjs` paths explicitly. An immutable exception needs
   an exact path, original owner, content identity and compatibility reason;
   a whole fixture directory is not an exception. Generated `dist/*.mjs`, npm
   export paths and public runtime fixture bytes receive separate compatibility
   review; renaming handwritten source does not silently change their contract.

Active LSP, n8n, Glyph, HTML selection, release custody and Docs worktrees retain
file ownership. Coordinate their imports and launch paths before replaying a
migration on their actual merged source; do not modify or rebase their branches.

## Acceptance

A renamed file alone is unfinished. Preserve source and output bytes after type
erasure, execute every original test, verify the strict project and exact-head
Actions, then track protected candidate and actual merge. Broad removal remains
unfinished while any non-exempt handwritten `.mjs` path survives. Publication,
installed acceptance, performance improvements and issue closure are separate
claims and receive no credit from this migration's source proof.

## First-slice local proof

`e19e25e1af` contains four 100% byte-exact moves and no other change. All 15
original planner laws passed before migration. After migration, 46 original,
adjacent real-Git/census/canonical/queue/compiled-document and new strict
compiler laws pass with zero skipped or filtered tests. The actual strict
six-file native project and focused eight-file Vite+ check pass. Canonical
decision row 225 is the sole edited row; the other 349 remain byte-exact.
Exact-head source Actions, protected candidate and actual merge remain pending.
