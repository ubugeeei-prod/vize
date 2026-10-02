# Original component filename history witnesses

Issue: [#6881](https://github.com/ubugeeei-prod/vize/issues/6881).
Original change: [#4447](https://github.com/ubugeeei-prod/vize/pull/4447),
`eaafa5a1f67883277407fcf1c5f3f2b2101ef2ed`.
Prepared source parent: `dd6beada6373fc58af149e7e19cf0204710ec022`.
Capture replay parent: signed actual `4cabe83f7d0826907b6035b58d76798e92634927`.

## Original semantic requirement

The original rule accepts PascalCase or kebab-case component filenames. It
checks the `.vue` filename stem, rather than a script `name` option. The new
ASCII kebab predicate allows lowercase letters, digits and hyphens, requires
the first segment to start with a lowercase letter and rejects empty segments.
The rule reports only when neither the PascalCase nor kebab predicate accepts.
The witnessed mixed-capital, empty-hyphen-segment and lowercase dotted names
remain reported.

The six changed or added original test functions supply these eight witnesses:

| Original test                               | Filename                       | Warning count | Semantic branch                                  |
| ------------------------------------------- | ------------------------------ | ------------: | ------------------------------------------------ |
| `test_valid_kebab_case`                     | `my-component.vue`             |             0 | Ordinary kebab filename                          |
| `test_valid_component_kebab_case_with_path` | `src/components/job-board.vue` |             0 | Component path, without the Nuxt pages exemption |
| `test_valid_kebab_case_with_digits`         | `grid-2-col.vue`               |             0 | Nonempty digit segment                           |
| `test_invalid_mixed_kebab_and_pascal`       | `my-Component.vue`             |             1 | Capital inside a hyphenated name                 |
| `test_invalid_hyphen_boundaries`            | `-my-component.vue`            |             1 | Empty first segment                              |
| `test_invalid_hyphen_boundaries`            | `my-component-.vue`            |             1 | Empty last segment                               |
| `test_invalid_hyphen_boundaries`            | `my--component.vue`            |             1 | Empty internal segment                           |
| `test_invalid_dotted_name`                  | `page.block.vue`               |             1 | Dot satisfies neither convention                 |

Every source is the original `<div>Content</div>` with no extra whitespace or
SFC wrapper. The original test helper registers this sole rule with
`RuleRegistry::new()` and `Linter::with_registry`. It asserts warning counts;
it does not freeze complete original English diagnostic bytes or configure
the current observer's preset/help explicitly.

## Complete current public observation

Keep the original source, filename, sole rule and expected zero/one finding.
Use the existing source-built `lint_history_observer --current-api`: public
`lint_template`, `Incremental` preset, explicitly enabled sole rule, English
locale, full help and unspecified Vue/Vapor options. These explicit capture
options are recorded rather than attributed to the historical test helper.
The current later PascalCase policy preserves these eight expectations.

The strict input stores the complete original SHA, source and options. The
observer returns the complete `Case` and `Observation` Debug bytes: ordered
diagnostics, messages, severity, authored spans, labels, help, offers, complete
initial `LintResult` and unchanged requery. This rule offers no fix; every
case expects zero edits and actually requeries the original bytes. Each
process repeats the whole observation, and two fresh processes must agree.
No diagnostic construction, normalization or count-only oracle is accepted.

The shared build receipt binds the exact committed source revision/tree,
workspace package, observer source, lockfile, Cargo JSON artifact, successful
build completion, toolchain, executable and raw build/probe hashes. Existing
Actions evidence upload retains all raw attempts, stderr, failures and the
receipt under `target/differential/linter-api/`, even on failure.

## Capture and acceptance status

The eight strict original inputs are prepared for source-built Actions
capture. `component-name-original-capture.json` retains complete input and
stdout/stderr bytes, hashes, statuses and repeat results. This initial capture
is explicitly `pending-complete-oracle-registration`: it supplies no fixture
equivalence or native acceptance. No matching current-source local observer
receipt/executable was available, so no old binary is used as a substitute.

TODO: freeze the actual eight complete observations, register their immutable
oracles in the shared manifest, then rerun the actual source-built comparison
at the final published head and protected queue candidate. The existing 36
input/oracle bytes and their current registry remain unchanged during capture.

This is bounded English coverage of eight original filename witnesses. It
does not close every helper edge, all three changed translations, repository
snapshot deltas, the whole original commit or the history denominator. The
503 candidate touches are still an inventory, not accepted requirements.
Native handled/equivalent/paired comparisons remain zero; #6881 stays open.
No parser, production rule, observer mode or product route is changed.
