# Japanese rule explanation review (#8372)

This slice repairs authored rule explanations and their regeneration inputs. It
does not close the whole-site Japanese editorial issue. Native rendering, protected
merge, deployment and the remaining non-rule reading paths require separate proof.

## Original source and review coverage

The complete review uses #8512 source
`6f1fb62906ba5c0b74c885d8290fc2a49cfa2415`, after the category composition from
#8355. Each reviewer read the complete Japanese page, original English page,
configuration, prerequisites, Bad/Good files, graphs and support qualifications.
The aggregate owner then reviewed every complete before/after proposal against its
owning generator literal before applying it. Review markers alone are not evidence.

| Original page pairs                                                      |   Count | Owning inputs                                                     |
| ------------------------------------------------------------------------ | ------: | ----------------------------------------------------------------- |
| Vue references                                                           |     104 | `explanations-vue-{0,1,2}.ts`, `purpose-ja.ts`                    |
| Script/type references                                                   |      70 | `explanations-script-type-{0,1,2,3}.ts`, `purpose-ja.ts`          |
| Accessibility, CSS, HTML, ecosystem, Musea, Nuxt, petite-vue, SSR, Vapor |      77 | Other explanation groups, `purpose-ja.ts`, shared scope/notes     |
| Project references                                                       |      66 | `project-explanations-*`, `project-contracts-*`, project renderer |
| **Complete reference pairs**                                             | **317** | **251 single-file + 66 project entries**                          |

The four report-only audits proposed 247 complete Japanese fields in 22 owning
inputs. This change also corrects shared catalogue introductions, support labels,
scope descriptions and the migration table labels. Source rule IDs, default
severities, presets, options, availability, diagnostic producers, entry roots and
the 19 CLI / 16 experimental-library / 25 producer-free-contract boundaries stay
unchanged. The CLI group still distinguishes its 18 qualified source pairs from
the one illustrative reactive-flow graph scenario.

The original source inventory is durable in
`tests/_fixtures/docs-ja-rule-original-custody.json`: all 317 pairs retain their
original whole-page identities, whole executable/configuration/graph hashes and
support facts. Tests compare current complete code blocks to those original
hashes. All 780 existing rule Markdown files were also compared after regeneration:
no fenced source or graph changed; French, Portuguese and Chinese pages remained
byte-identical. This is preservation evidence, not runtime qualification.

## Meaning corrections

The changes explain the actual code rather than translating isolated words:

- `aria-hidden` hides decorative icons from assistive technology; it does not
  describe visually hiding an icon.
- `role="presentation"` on a focusable element conflicts with native semantics;
  the browser must ignore that role instead of erasing the link's semantics.
- `:slotted(.label)` selects the `.label` class, not an HTML `label` element.
- `useId()` provides unique IDs, and the example binds its returned value to
  `label.for` and `input.id`.
- `scoped` is attached to the style block, not the CSS selector or declaration.
- An arrow watcher has lexical `this`; the repair lets Vue bind a normal method
  to the component instance. `deep: true` is available in object form, rather
  than supposedly retained from a Bad example that never had it.
- Optional slot calls and `input?.focus()` are conditional calls. 「任意」 did not
  explain why the call is omitted when the slot or input is absent.
- Nuxt's rejected `test` shape is the old boolean property; the Good object form
  remains allowed. A petite-vue `v-scope` without a value remains allowed.
- Producer-free SFC conventions describe intended usage, and never claim a rule
  or diagnostic is emitted. Type-aware opt-ins still need the native Corsa runtime
  and a TypeScript project.
- Project explanations name the actual provider/consumer, preserve event payload
  limitations and keep unsupported source-fact and experimental-library warnings.

## Inherited technical claims

Five public explanation purposes inherited overbroad English and Japanese messages.
`project-purposes.ts` supplies bounded documentation purposes for the existing
scenarios. Runtime message catalogues and CLI explanation snapshots remain original;
their discrepancies require a separate product change with compatibility evidence.

| Existing scenario     | Accurate documentation qualification                                                                                                    | Primary evidence                                                                                                                                                                               |
| --------------------- | --------------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Array index mutation  | The example is the historical Vue 2.7 repair; Vue 3 tracks these assignments. This contract has no current producer.                    | [Vue 2 array caveats](https://v2.vuejs.org/v2/guide/reactivity.html#For-Arrays), [Vue 3 proxies](https://vuejs.org/guide/essentials/reactivity-fundamentals.html#reactive-proxy-vs-original-1) |
| Pinia getter snapshot | Copying the current primitive creates a snapshot; direct tracked/template reads remain reactive.                                        | [Pinia store access and extraction](https://pinia.vuejs.org/core-concepts/#destructuring-from-a-store)                                                                                         |
| Setup ownership       | Module-scope reactive state is legal Vue; this experimental example requires per-instance state. CLI does not emit the individual code. | [Vue shared reactive state](https://vuejs.org/guide/scaling-up/state-management.html#simple-state-management-with-reactivity-api)                                                              |
| Raw/proxy identity    | A proxy differs from its original object; `toRaw()` returns the original. Identity does not change on every unwrap.                     | [Vue `toRaw()`](https://vuejs.org/api/reactivity-advanced.html#toraw)                                                                                                                          |
| Async `watchEffect`   | The shown Bad code lacks cleanup/stale-response handling. The API supports cleanup.                                                     | [Vue watcher cleanup](https://vuejs.org/guide/essentials/watchers.html#side-effect-cleanup)                                                                                                    |

These documentation qualifications do not implement missing producers or make
illustrative sources diagnostic witnesses. Other locales and original CLI messages
have not received editorial acceptance for these inherited claims.

## Complete native paragraph controls

`docs-ja-rule-editorial.json` retains 24 independently reviewed complete original
and repaired paragraphs. Actual Chromium at published source
`3f82aa2f00277b34615be5cca9f32e3074832cba` reproduced all original individual-reference
paragraphs on desktop (1440) and mobile (390): all 48 matched the whole original text
and rejected the repaired-paragraph gate. No authored page or served DOM was changed
for this reproduction. Published category pages at that source are older and do not
establish the later composed inline catalogue acceptance.

The normal native navigation renderer now checks those complete repaired paragraphs
inside their actual owning categories, alongside every original code/graph/anchor,
theme, font, link and complete-pixel check. The original standalone availability
heading bookmark `設定できる-id-現在の-sfc-検出なし` and AllRules project chapter bookmark
`プロジェクト-ルールと-analyzer-契約-66` remain native targets. The availability bookmark
stays standalone, avoiding duplicate IDs when references are composed into a catalogue.

The original full Docs runs for #8574 (`38114235444`) and #8587 (`38114496671`)
exposed an incomplete embedding boundary in this slice. Both devices rejected
`/ja/rules/vue`: its three unsupported-rule references repeated the standalone
availability bookmark. The native category used the common `inlineReference`
helper, while only the alternate native embedding helper had excluded this ID.
Those full runs failed; their other successful pages do not qualify the source.

The common helper now excludes only this standalone bookmark outside fenced
examples. All three standalone targets, complete original source blocks, prose
and Bad/Good bookmarks remain intact. The complete generated Vue category changes
only by removing those three span lines. A source regression reads all 104 actual
Vue references and compares every complete code fence; a separate literal-source
case ensures bookmark markup inside a fenced example is preserved. Both fail
before the correction. Actual local SSG on desktop and mobile also reproduces the
unchanged strict duplicate-ID failure before correction. Fresh exact-head full
Docs and its original whole category controls remain required after the coherent
Stack replay; the earlier successful captures are not new-source executions.

Local generator reproducibility and all 17 original and new rule/accessibility
tooling tests pass with the original project-oracle dependencies installed. The
complete native Docs suite remains an Actions gate. Original public before-text is
not repaired-source browser evidence.

## Delivery and remaining acceptance

Follow the real #8512 indentation dependency and coordinate the #8334 overview
composition. Preserve incoming overview ownership and these natural Japanese inputs;
never resurrect the old index promise if the page becomes a genuine overview.
Register and verify the native Stack, qualify the exact child source in Actions and
full Docs, inspect whole rendered captures, and let the root own protected queue and
actual merge. After deployment, replay the original reading routes and bookmarks.

The five JA editorial layers remain separately owned. Shared external Markdown URL
resolution (#8566), shared table glyph geometry, the JSX fragment alias, remaining
non-rule editorial/navigation paths and public deployment acceptance are unfinished
until their own exact-source evidence passes. Neither this rule review nor the
existing marker count completes all 396 Japanese source pages in #8372.
