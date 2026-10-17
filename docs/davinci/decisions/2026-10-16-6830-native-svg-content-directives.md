# Classify native content directives by Vue component identity

Issue: [#6830](https://github.com/ubugeeei-prod/vize/issues/6830).

App E2E [38866347055](https://github.com/ubugeeei-prod/vize/actions/runs/38866347055)
introduced ten `vue/no-v-text-v-html-on-component` findings on npmx's `<g>`
watermarks after [#9107](https://github.com/ubugeeei-prod/vize/pull/9107).
That change used lexical HTML/SVG/MathML ancestry to decide whether a tag was
a component. Named SVG slots and a component's SVG root fragment do not need
a literal `<svg>` ancestor in their own source.

Vue's [compiler-dom native-tag predicate](https://github.com/vuejs/core/blob/v3.5.35/packages/compiler-dom/src/parserOptions.ts)
recognizes HTML, SVG and MathML tag names globally. Its renderer carries the
receiving SVG namespace through component and slot boundaries. Component
identity and the element's namespace are separate decisions.

Use the existing `is_native_tag` predicate for this rule's fallback. Keep the
existing dynamic-component analysis and explicit component classification.
Unknown lowercase names, kebab-case components, PascalCase components and
explicit `is="vue:..."` substitutions still produce errors. Native `view` and
`text` at the root are SVG tag names, so #9107's tests that called them unknown
components had incorrect expectations; actual unknown names retain coverage.
This does not exempt native content from other rules or change HTML validity.

The pinned eslint-plugin-vue 10.9.2 default reproduces both original findings
at `FacetBarChart.vue:300:13` and `TimelineChartDepSizeSvgSlot.vue:16:68`.
Its documented [`ignoreElementNamespaces: true` option](https://eslint.vuejs.org/rules/no-v-text-v-html-on-component.html#options)
agrees with Vue's native classification for these cases. The differential
test retains both reference configurations; the default reference's finding
is not treated as proof that Vue resolves a component.

The corpus at `tests/_fixtures/differential/lint/native-svg-content/` retains
the complete two original files and license from npmx commit
`c9e8be91e61eeeae956e2068722caa09e293cc68`, independently re-fetched and matched
byte for byte, plus source digests and nineteen native/component controls.
Rust exercises template and SFC entry points, the whole originals, and the
same originals with SVG groups changed to actual components.

The public Vue 3.5.35 oracle parses and compiles both complete templates.
Its renderer then runs the exact original watermark group in a receiving
chart slot and the complete original SVG fragment in a receiving component.
Both create native SVG `g` elements and assign `innerHTML`, even when a `g`
component is registered; an unknown lowercase control resolves its registered
component. These are focused compiler/renderer observations. Whole-app
expectations are reviewed separately in the
[application lint record](2026-10-16-6830-app-lint-expectations.md).

The pre-fix Rust run rejects the root `g` control and the complete original
FacetBarChart. The repaired source passes all three integration tests and all
23 existing rule unit tests, including dynamic `is` handling. All three public
oracle tests pass. Hosted whole-app validation remains a separate check.

## Keep the new originals in automatic corpus sweeps

The first repair Check run, [38882495270](https://github.com/ubugeeei-prod/vize/actions/runs/38882495270),
found the two new whole-source Vue fixtures in the existing Canon projection
and L3 remarks sweeps. Their fixed corpus censuses therefore needed additive
registration. The files remain in both sweeps; the repair does not exclude
them or replace existing coverage.

Canon retains its original 64 comparisons and the seven previously registered
additions. A separate authenticated registry adds exactly two diagnostic
comparisons from FacetBarChart; the complete Timeline source contributes zero.
Both original files are checked by path, byte count, mode and SHA256, including
missing-input and same-size mutation controls. All six focused projection
tests pass, with no source-mapping divergence.

The L3 baseline adds only the two file rows and six complete remarks belonging
to these sources. Removing those additions must reproduce the old 553-file
baseline byte for byte. All existing corpus assertions and observations remain
in place; a fresh Check run must confirm the complete combined sweep.

Full qualification also exercises the L1-to-L2 sweep with `legacy-differential`.
Its unchanged producer adds the same two files and 43 complete remarks: nine
applied and 34 missed. Removing only those additions restores the original
553-file, 1,240-remark baseline byte for byte. The existing generator also
updates its required `remarks-backlog.md` companion. A second run without the
update flag passes with 555 files, 1,283 remarks and zero observed changes.
All 43 new absolute source spans were reviewed against the original SVG files;
existing records, source pins, assertions and lowering behavior remain intact.
