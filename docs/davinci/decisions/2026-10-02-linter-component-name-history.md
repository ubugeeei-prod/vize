# Original component filename history witnesses

Issue: [#6881](https://github.com/ubugeeei-prod/vize/issues/6881).
Original change: [#4447](https://github.com/ubugeeei-prod/vize/pull/4447),
`eaafa5a1f67883277407fcf1c5f3f2b2101ef2ed`.
Prepared source parent: `dd6beada6373fc58af149e7e19cf0204710ec022`.
Capture replay parent: signed actual `4cabe83f7d0826907b6035b58d76798e92634927`.
Final registration parent: signed actual `745aec2c2eb5580f0c398168ef040cfd550f34b9`.

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

## Immutable source-built capture and registration

The actual [Check capture](https://github.com/ubugeeei-prod/vize/actions/runs/36967214570)
completed successfully on `9c08df5812d24f164254abfa727b02c9d583c41f`, tree
`165b91787d2515e69657ca0bacd1bde082bf940a`, attempt 1. Its
`test-scripts` job `110713462355` built the current public observer and
retained the full eight inputs, two fresh-process outputs/statuses each,
Cargo streams, option probe and source/build receipt. All eight warning,
zero-error/no-edit and unchanged-requery expectations passed. The existing
36-case source-built report had 36 exact matches and zero native passes.

Original artifact `11210836312` has literal service ZIP SHA-256
`709774db6915323ff5a87f14bb564398928e36efef0ea351b7f26e6ca222fb1d`.
A separate [read-only hosted transport](https://github.com/ubugeeei-prod/vize/actions/runs/37012643091)
on `23bfba9dac13e6f574fa786c52860bc7f74a49a5`, attempt 1, job
`110855835084`, verified that whole ZIP and every entry CRC. It emitted all
14 selected text files in 40 lossless frames, without running the product.
The host rehashed the original ELF, SHA-256
`49b407788011f5c3d10cd55f92f462b9994277e96cdbc1a035b5545ef9989dce`,
and bound it to the actual build receipt; the binary was not transported or
executed locally. The complete decoded capsule SHA-256 is
`07f1044882670f141ca3c406f03e9e071e506596e0e98a5fa7b968448a664995`.
The original raw eight-case capture SHA-256 is
`ce4243c1dcaf79a88cd02a6ece318f71a0631d147b84b3bd668fb79eca5303e9`.

Each new snapshot contains its untouched complete observed stdout, with
only an Insta metadata header added. The manifest binds every original input
and whole snapshot; the adapter uses the existing `--current-api` route.
All 36 previous inputs, oracle bodies and index entries remain unchanged.
The temporary capture helper and transport workflow/scripts are absent from
this final change. The shared runner now requires 44 complete comparisons,
each twice in fresh processes; native outcomes are explicitly unsupported.

| Original witness                 | Complete stdout SHA-256                                            | Bytes |
| -------------------------------- | ------------------------------------------------------------------ | ----: |
| `valid-kebab-case`               | `3283559e4d5c2d5c022bed44ead4f93cf4138421b22829f7c63167361b3f1ccf` |   690 |
| `valid-kebab-case-with-path`     | `c6f12bd67d7b4e0f0e75637802ea4e0444aec492a6e813e38eb27218dc8bfddd` |   736 |
| `valid-kebab-case-with-digits`   | `685419a7826ffd982be3ca03de68913a7ad9c0df2f27666cc4aab4cd6e5b5d34` |   696 |
| `invalid-mixed-kebab-and-pascal` | `7dc0ecc870c81b37807c028b73083adf50433450ff40122e3bca10ec7506ad55` |  1872 |
| `invalid-leading-hyphen`         | `21a220ae471f33829769dcca60f9662cc5b03eb2f29023c0b239d36d2f2823db` |  1869 |
| `invalid-trailing-hyphen`        | `d1eb61ea98dc3e399f46c12d40d2d3c3cabd934e99b13887294459533b0a87d4` |  1870 |
| `invalid-doubled-hyphen`         | `679ea214d2bd1d18ff4dba396774c5dc85c76bc2e7cb15a8c72bbbbdbcd331e8` |  1869 |
| `invalid-dotted-name`            | `85568c1544da88c54c4106c3f4c0d2d96b3b71d3a1f0a7bed419aecb667aa28d` |  1851 |

TODO: validate the registered 44 cases at the final published head and
protected queue candidate, retain their complete source-built reports and
verify actual merge. Original capture proves only its pinned source; it does
not substitute for later-head or queue execution.

This is bounded English coverage of eight original filename witnesses. It
does not close every helper edge, all three changed translations, repository
snapshot deltas, the whole original commit or the history denominator. The
503 candidate touches are still an inventory, not accepted requirements.
Native handled/equivalent/paired comparisons remain zero; #6881 stays open.
No parser, production rule, observer mode or product route is changed.
