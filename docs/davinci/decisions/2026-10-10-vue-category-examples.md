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

## Protected delivery rebase

Current main made the old bottom branch conflict only in the final canonical
record row. Rebase the genuine native Stack onto current main, retaining
every incoming decision and source ratchet. Move only this slice's original
unchanged clause to the existing tooling-input paragraph, and place the
category child beside it when that child rebases. No rule, example, locale,
rendering assertion or whole-page capture is removed. Earlier source and
Docs receipts remain historical; the refreshed heads require fresh complete
Actions, protected queue and actual deployed acceptance.

## Composition with delivered preset and options metadata

The exact old top Docs run completed all 180 desktop/mobile routes and all
1,419 Open Graph frames; all five Vue routes retained 104 rules and 416 complete
source blocks per device. These receipts qualify the old heads only.

Actual current main delivered preset membership for two deprecated Vue rules
and typed `valid-v-slot` options. The old category child conflicts with three
translated whole-rule tables because it replaces those tables with complete
same-page indexes. Rebase both genuine layers onto that actual main once and
regenerate with the unchanged metadata producer. All 416 original complete Vue
code blocks remain byte-identical in each locale. Both layers require their
own fresh source and full Docs Actions, protected merge, and deployed acceptance.

## Composition with delivered navigation controls

Actual signed main `538c07ac7eca17db3151fe68cdf38586bc561bb0` introduces the
shared navigation route list and motion, contrast, command-tab and theme checks.
Both that main and the observed future queue projection conflict in the browser
verifier. Compose the genuine parent once, retaining every incoming route and
control, all five localized Vue routes, both devices/themes and complete pixels.
All 104-rule/416-block controls remain unchanged. Old runs are historical;
the composed source requires fresh full Actions, protected and deployed proof.

## Delivered renderer worker composition

The genuine actual `a936260f7c` composition retains the delivered bounded
renderer. The entrypoint keeps the three additional Vue locale routes, and the
moved page owner keeps the five-locale palette guard. Every original authored
packet, capture, clipboard, font, navigation and refusal assertion is retained.
