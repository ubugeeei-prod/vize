Part of #6848. Follows #13315. Moves eight more rules that still ran on the JSX lowering fallback onto the markup facade, one commit per family. Each rule now has one implementation that runs on SFC (L2 facade), JSX/TSX (L2) and TSRX.

## Rules ported

| family | rule | facade hook |
| --- | --- | --- |
| html | `html/id-duplication` | `enter_document` + `walk_branched_elements` |
| vue | `vue/no-template-key` | `enter_element` |
| vue | `vue/no-v-for-template-key` | `enter_element` |
| vue | `vue/no-v-for-template-key-on-child` | `enter_element` |
| vue | `vue/no-lone-template` | `enter_element` |
| vue | `vue/no-child-content` | `enter_element` |
| vue | `vapor/no-inline-template` | `enter_element` |
| vue | `vue/no-script-non-standard-lang` | `enter_document` |

Structural directives (`v-for`, `v-if`, `v-else`, `v-slot`) are read through `walk_authored_directive_ranges`, which every backend answers. All eight joined `sfc::facade::RULES`. On the corpus below, L2 and Relief give identical results for each of them.

`html/no-empty-palpable-content` is held back. Its options are on #6848.

## Rule-parity lanes (`node tools/support/compat/davinci/rule-parity.ts --write`)

| lane | after #13315 | after this PR |
| --- | --- | --- |
| `fallback` | 92 | 84 |
| `ir` | 67 | 74 |
| `ir-lowered` | 19 | 20 |
| `markup-facade` rules | 86 | 94 |

## Evidence

- Real-project corpus: 45,091 `.vue`/`.jsx`/`.tsx` files plus 1,068 rule-fixture and battery templates (bare and SFC-wrapped). Linted with the default preset, all rules, and all fallback rules. Compared against #13315:
  - 141,694 results are byte-identical.
  - 26 TSX results contain the same diagnostics in a different `LintResult` order, for the reason given in #13315. The CLI sorts diagnostics before output.
  - 0 content differences. Across the three configurations the dumps hold 438 `html/id-duplication` and 2,336 `vue/no-v-for-template-key` diagnostics, among others.
- `relief_parity::fallback_ports`: facade vs Relief hooks vs all-hooks on templates, SFCs, JSX/TSX and the JSX corpus, for every ported rule.
- SFC facade battery: parity fixtures and `eslint-disable-next-line` controls for each admitted rule.
- `cargo test -p vize_patina --lib`: all pass. `--test integration`: only the two type-aware `type/no-unsafe-template-binding` rows of `lint_history` fail (`multiline-v-for-source-9880`, `vue-path-alias-types-9864`, a macOS checker environment issue). Every row of the ported rules passes.

## Intended change: TSRX

These rules never ran on `.tsrx`. New tests in `linter/engine/tsrx/tests.rs` pin duplicate ids across exclusive regions, `<template>` without a directive, `inline-template`, and `<script lang>` text.

## Finding

`vue/no-script-non-standard-lang` scans the template lint source. On an SFC that source is the template block, so the rule never sees the SFC's `<script>` blocks: it reports nothing on the 41k-file corpus. The port keeps that behavior. Moving the rule to an SFC-level hook is a separate fix.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
