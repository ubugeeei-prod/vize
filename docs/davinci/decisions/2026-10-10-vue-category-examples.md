# Vue category examples on the current page

Issue: [#8334](https://github.com/ubugeeei-prod/vize/issues/8334).

This is a locally prepared, independently reviewable first slice. Third-party
issues #8328 and #8329 retain priority through successful publication and installed
acceptance. No docs PR, remote change, deployment or issue closure is implied.

## Decision

The Vue category includes every current `vue/*` rule's purpose, prerequisites,
configuration, complete Bad and Good source, and the reasons for both examples.
Readers use same-page rule and example anchors. Existing individual reference
routes remain available for deep links. Native heading IDs own the rule anchors,
avoiding duplicate IDs from a second explicit rule span. The four prior
non-rule heading fragments in translated category pages remain as alias anchors.

Generation composes the authoritative EN/JA references in the existing rule-page
pass. All five supported locales receive the same complete executable source
packets. French, Brazilian Portuguese and Simplified Chinese have explicit
translations of every purpose, both explanations, shared metadata and support
notes. Missing rule explanations or untranslated prose fail generation.

The authored category files remain compact indexes. Complete packets live in the
existing generated-content directory and replace those indexes during the
existing materialization pass. This keeps authored files within the 350-line
limit and excludes derivative packet paths from public routes and Open Graph
generation.

Only documentation fence metadata marks changed lines. Ox Content 2.81's native
`annotate="remove:...;add:..."` renderer supplies diff classes; source bytes remain
copyable without added prefixes. Changes are computed at build time. No new
runtime fetch, component, pipeline stage, handwritten `.mjs` or dependency upgrade
is introduced. The existing Vue SFC Open Graph implementation remains authoritative.

Where a translated prerequisite page does not exist, its link uses the real
English page. This avoids creating broken localized links or inventing translated
option documentation. Rule IDs, configuration values and technical identifiers
remain unchanged.

## Verification and remaining work

The existing deterministic generator and complete rule-reference tooling tests
cover all five Vue category pages, source-byte equality, support boundaries and
local anchors. The existing native annotation test suite includes an actual
renderer check for changed lines and complete copied source.

Local validation passed all 29 selected documentation tooling tests, strict
type-aware lint and deterministic generation. Pure rendering with the installed
Ox Content 2.81 native engine verified all five pages: 104 rules, 416 complete
code blocks, 416 same-page example links, and unique heading IDs per page. Each
page has 254 added and 206 removed annotated lines. Gzipped native HTML is
29,752–32,468 bytes. These are bounded local rendering receipts, not a browser,
SSG, deployed-navigation or performance benchmark result.

The Docs browser verifier now checks all five Vue category routes on desktop and
mobile, in both themes. Its receipts require all 104 complete packets, same-page
Bad/Good links, native diff classes and unchanged copied code.

TODO before publication: root must pair this decision with a short comment on
#8334, rebase onto actual current main, obtain exact-head Actions and protected
merge evidence, then verify the actual Docs build, Pages deployment and live
reader flow. Local rendering is preparation, not deployed acceptance.

TODO for the full issue: apply the same reader flow to the remaining categories
and cross-file examples in every supported locale. Preserve existing support
boundaries and measure page size and navigation responsiveness. #8334 stays open
until the complete original acceptance is met.

Proposed paired issue comment:

> The Vue category now has all 104 rule explanations and both examples on the
> current page in all five locales, using native diff annotations without changing
> copied source. Remaining categories, cross-file views and deployed verification
> stay tracked here; third-party release delivery remains first.

## Source Actions corpus correction

The first source Check at `f10f75f` exposed a stale natural v-on corpus
inventory in tooling shard 3/4 (job 114105009567). Generated Vue reader pages
add 60 real rows to `docs--content.tsv`, taking that inventory from 63 to 123
occurrence rows (excluding its header). Regenerate it with the unchanged corpus producer; retain all event, option
and storage maxima at two. No scanner, example bytes or product behavior changes.

The existing corpus producer's `--check` and all five v-on storage tests pass
locally. Fresh exact-head Check and Docs Actions are required after rebasing the
complete dependent stack onto the signed v0.439.0 metadata merge. Earlier source
runs remain evidence of the original failure, not acceptance of the new head.
