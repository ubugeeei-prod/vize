# Musea capture and accessibility reader flow

Related issues: #8365 and #8372.

## Decision

Explain when to change capture settings, how to inspect the first comparison,
and how to read node-level accessibility evidence before approving a baseline.
Retain the entire original English article and capture recipe. Add a complete
authored Japanese translation with descriptive links to the localized Musea
setup and visual regression workflow.

Animation freezing defaults to `disabled`, caret hiding to `hide`, and the
browser motion preference to `no-preference`. Explicit `reduce` requests the
page's reduced-motion behavior; actual movement depends on the page's support.
The existing recipe and its English code comments remain byte-exact.

The CLI writes `a11y-report.json` with `--json`, otherwise `a11y-report.html`,
to its output directory (`.vize` by default). Describe `nodes`, `targets`,
`any`, `all`, `none`, `data`, `incompleteResults` and `incomplete` without
changing their contracts. Manual-review results are not confirmed passes, and
an audit error means the variant was not audited.

## Localized workflow bookmark

The existing Japanese Musea heading lacked the English
`visual-regression-testing` bookmark. Add only that explicit span before the
unchanged translated heading. Removing the span restores every incoming guide
byte, including all code fences, URLs and configuration explanations.

Declare that single insertion as the fourth patch in the existing finite
editorial succession entry. Preserve its original digest, all three previous
configuration patches, every other entry and witness, and the strict helper.
The old manifest correctly refuses the undeclared alias. The declared
successor still reverses to the immutable original editorial corpus.

## Qualification and remaining work

The existing More, reviewed translation and Japanese editorial laws remain
required. Add a regression for the complete capture recipe, authored
regeneration and both localized workflow destinations. Native Actions must
render both pages; actual merged-source publication and a stable public
desktop/mobile reader check must also follow the real VRT link to its heading.
Source review alone does not establish that public result. Broader Japanese,
More and installed Musea acceptance remain separate work.
