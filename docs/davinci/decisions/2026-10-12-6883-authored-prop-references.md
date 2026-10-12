# Authored component-prop references

The [LSP fix-history issue](https://github.com/ubugeeei-prod/vize/issues/6883)
keeps two source-specific reference contracts: Lew UI's `lang` prop has 12
authored locations, and Misskey's `code` prop has four. Each group includes
the original declaration and template use plus consumers of the same imported
component. Other authored cases keep their original two-location contract.

The original [v0.447 Matrix run](https://github.com/ubugeeei-prod/vize/actions/runs/38205026873),
attempt 1, source `ef34d1a0409fc711a6c5cefcf0708b0cce78f502`, failed both
reference assertions because the driver expected only the two local locations.
Its complete responses already contain the valid public prop consumers below.
All 16 targets are inside their initialized authored roots. The evidence does
not justify filtering the product's references to local declarations and uses.

The [separate contract table](../../../tests/tooling/support/authored-prop-reference-contracts.json)
preserves all 13 complete fixed source/config bodies, their byte lengths,
SHA256 values and Git blob identities, the complete original registry oracles,
and the original failed project objects, local pairs, complete parsed reference
responses and request parameters.
The `historicalFailures` fields retain those failures and mark the later stages
unexecuted; they are never inputs to the expected-vector constructor.

The [helper](../../../tests/tooling/support/authored-prop-references.ts)
requires every fixed source/config body, regular non-executable files at their
declared physical paths, and the entire original oracle. It resolves each direct component import, checks the fixed project's
include/alias rules, and derives the prop-key ranges from unique source contexts
using UTF-16 offsets. It builds fixture URIs through the existing authored-root
boundary. It accepts no native response as an input.

Lew UI is pinned to
[`abdcecdcdf611211b0502383a6dbfacded2c9608`](https://github.com/lewkamtao/lew-ui/tree/abdcecdcdf611211b0502383a6dbfacded2c9608).
All ten external attributes have direct imports of `LewCodeHighlighter.vue`.
The fixed `tsconfig.json` includes `docs`; its complete body remains mandatory.
Positions below are zero-based, half-open UTF-16 ranges and select `lang`.

| Fixture-relative file              | Start → end     |
| ---------------------------------- | --------------- |
| docs/components/LewCodeBox.vue     | 66:41 → 66:45   |
| docs/guide/Install.vue             | 11:37 → 11:41   |
| docs/guide/Locale.vue              | 17:37 → 17:41   |
| docs/guide/QA.vue                  | 35:37 → 35:41   |
| docs/guide/QA.vue                  | 42:37 → 42:41   |
| docs/guide/Start.vue               | 23:37 → 23:41   |
| docs/guide/Start.vue               | 29:37 → 29:41   |
| docs/guide/Start.vue               | 40:37 → 40:41   |
| docs/layout/LewCodeHighlighter.vue | 8:2 → 8:6       |
| docs/layout/LewCodeHighlighter.vue | 21:13 → 21:17   |
| docs/layout/LewDemoBox.vue         | 215:47 → 215:51 |
| docs/layout/LewTypeCode.vue        | 14:39 → 14:43   |

Misskey is pinned to
[`810faa8e5db4bbf9d6d408a61df148559340ee99`](https://github.com/misskey-dev/misskey/tree/810faa8e5db4bbf9d6d408a61df148559340ee99).
Its Storybook source imports `./MkCodeInline.vue`, uses that component in the
template and supplies `args.code` under `StoryObj<typeof MkCodeInline>`.
`MkMfm.ts` imports the same component through `@/components/MkCodeInline.vue`
and supplies the key in `h(MkCodeInline, { code: token.props.code })`.
The complete frontend config maps `@/*` to `./src/*`, includes TS and Vue
sources, and excludes `*.stories.ts` rather than `*.stories.impl.ts`.
These four ranges select `code`; the prefix is
`packages/frontend/src/components/`.

| File after the prefix        | Start → end   |
| ---------------------------- | ------------- |
| MkCodeInline.stories.impl.ts | 31:2 → 31:6   |
| MkCodeInline.vue             | 6:30 → 6:34   |
| MkCodeInline.vue             | 11:1 → 11:5   |
| global/MkMfm.ts              | 397:5 → 397:9 |

The same `code: token.props.code` line also occurs at line 389 in a different
`MkCode` render call. The helper identifies the unique complete
`h(MkCodeInline, …)` context before finding its key. That other component and
the right-hand `token.props.code` value remain excluded.

The existing driver's only changes are one import and a wrapper around its
complete original local pair. The whole URI/range sorting and exact comparison
remain intact. The registry, generic laws, subsequent requests, timeouts,
rename expectations and lifecycle expectations remain byte-identical.
The [native prop-reference laws](../../../crates/vize_maestro/src/ide/references/corsa_tests/component_props.rs)
remain whole: `cross_file=false` still permits the same Child/parent prop group,
and the 64 unrelated same-spelling component attributes stay excluded.

The [source controls](../../../tests/tooling/authored-prop-references.test.ts)
declare all 16 positions independently. They run the actual driver with a
bounded test session through the references assertion and stop at an explicit
prepareRename sentinel. They reject each missing/duplicate location, each URI
and all four range-coordinate mutations, foreign/private targets, the real
same-spelling collision, changed or omitted source/config bodies, symlinks,
executable files, parent-directory substitutions, mixed cohorts and altered complete oracles. Every negative group is followed
by a fresh complete positive. Twenty Node laws pass with zero skips: eight new
controls, nine unchanged generic laws and three unchanged registry laws.
This is source-level evidence only; the test session supplies no native result
or evidence for any later stage.

The original sessions ended at the references failure and then closed.
prepareRename, rename, dependency completion, unsaved repair and file lifecycle
remain unexecuted for these two original cases. Fresh Linux native and Matrix
execution of the complete existing oracles is required before closure. No
performance, RSS, 10x target, installed-product or release success is claimed.

The retained attribution packet has SHA256
`29c15daa430f266e1ae95beb4c109f7dbaf2a3cbcf511aebc1421917a73772ec`;
the independent source-declared proposal has SHA256
`d02bd9d9d8b9ea841af3b0b49f1b46731ee6807a3ea24e0e7e4ed0d6fe45f1fa`.
Their originals are retained under
`/tmp/vize-447-lsp-cause-mapping-20261012/`. Original whole-22 custody is
`/tmp/vize-447-original-matrix-full22-custody/terminal-complete-custody.json`,
SHA256 `1bb9a26af560319bc24003e76c8c5740891df67edee27f6dbef443b248a491ac`.
Internal generated native sources/API frames were absent from those archives;
the mapping is correlated to complete source, not claimed as a captured
generated-source receipt. Upstream repositories were read only.
