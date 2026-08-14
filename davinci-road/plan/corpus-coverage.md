<!-- GENERATED FILE — do not edit by hand.
     Regenerate: node tools/davinci/corpus-coverage.mjs --write
     Verify:     node tools/davinci/corpus-coverage.mjs --check
     Generator:  tools/davinci/corpus-coverage.mjs -->

# Corpus construct coverage

Counts of the [taxonomy.toml](./taxonomy.toml) construct dimensions observed in the **hydrated** corpus projects registered in `tests/_fixtures/vue-ecosystem-fixtures.json` (Davinci P0-6). This file is generated; it goes stale whenever the taxonomy, the fixtures manifest, or the set of hydrated fixture submodules changes — regenerate with `--write`, verify with `--check` (byte-compare). The `--check` staleness gate can only join `tests/tooling/davinci-matrices.test.ts` once CI hydrates the full corpus; until then the scope-proof footer below is the honesty mechanism.

## Scan scope

Sources scanned per hydrated project (from the manifest's `vueGlobs`, plus `petiteVueGlobs` for the petite-vue entries):

| project             | sfc (html) | sfc (pug) | jsx/tsx | html |  js |
| ------------------- | ---------: | --------: | ------: | ---: | --: |
| `wave-ui`           |          0 |       219 |       0 |    0 |   0 |
| `dho-web-client`    |          4 |       211 |       0 |    0 |   0 |
| `vue3-admin-design` |          7 |         0 |     111 |    0 |   0 |
| `vue3-antd-admin`   |         99 |         0 |      29 |    0 |   0 |
| `vue-core-vapor`    |        105 |         0 |       0 |    0 |   0 |
| `vue-jsx-vapor`     |          0 |         0 |     104 |    0 |   0 |
| `wakapi`            |          0 |         0 |       0 |   29 |   6 |
| `petite-vue`        |          0 |         0 |       0 |    6 |   0 |

## Per-construct counts (hydrated projects only)

### Dimension 1: element_kind (start-tag classes)

| project             | native | component | slot | template | svg | mathml |
| ------------------- | -----: | --------: | ---: | -------: | --: | -----: |
| `wave-ui`           |   5549 |      2828 |  155 |     1558 |  39 |      0 |
| `dho-web-client`    |   3251 |      1665 |   30 |      218 |   4 |      0 |
| `vue3-admin-design` |    355 |       845 |    0 |        1 |   2 |      0 |
| `vue3-antd-admin`   |    239 |       621 |   34 |      137 |   2 |      0 |
| `vue-core-vapor`    |    507 |        89 |    9 |        5 |   2 |      0 |
| `vue-jsx-vapor`     |    341 |       118 |    8 |        0 |   0 |      0 |
| `wakapi`            |   1418 |         0 |    0 |        2 |   2 |      0 |
| `petite-vue`        |     89 |         0 |    0 |        2 |   5 |      0 |
| **total sites**     |  11749 |      6166 |  236 |     1923 |  56 |      0 |
| **projects seen**   |      8 |         6 |    5 |        7 |   7 |      0 |

### Dimension 2: directive (attribute names, incl. `:` / `@` shorthand)

| project             | v-if | v-else-if | v-else | v-for | v-on | v-bind | v-model | v-show | v-html | v-text | v-once | v-memo | v-cloak | v-pre | custom |
| ------------------- | ---: | --------: | -----: | ----: | ---: | -----: | ------: | -----: | -----: | -----: | -----: | -----: | ------: | ----: | -----: |
| `wave-ui`           |  234 |        52 |     44 |    56 |  396 |   1904 |     176 |     24 |     51 |      1 |      0 |      0 |       0 |     0 |      6 |
| `dho-web-client`    |  907 |        60 |    125 |   125 |  727 |   3219 |     183 |     81 |      5 |      0 |      0 |      0 |       0 |     0 |     24 |
| `vue3-admin-design` |    0 |         0 |      0 |     0 |  159 |     40 |      85 |      9 |      1 |      0 |      0 |      0 |       0 |     0 |      2 |
| `vue3-antd-admin`   |   60 |         2 |     12 |    39 |  164 |    530 |      47 |      5 |      1 |      0 |      0 |      0 |       0 |     0 |      0 |
| `vue-core-vapor`    |   54 |         4 |      7 |    26 |  206 |     86 |       5 |     14 |      0 |      0 |      0 |      0 |       0 |     0 |      1 |
| `vue-jsx-vapor`     |    5 |         4 |      5 |     5 |   72 |      0 |      15 |      5 |      1 |      0 |      1 |      0 |       0 |     0 |      2 |
| `wakapi`            |    6 |         0 |      0 |     2 |   47 |     55 |       9 |     10 |      1 |      0 |      0 |      0 |      15 |     0 |      9 |
| `petite-vue`        |    3 |         0 |      1 |     9 |   15 |     17 |       8 |      4 |      1 |      0 |      0 |      0 |       1 |     0 |      9 |
| **total sites**     | 1269 |       122 |    194 |   262 | 1786 |   5851 |     528 |    152 |     61 |      1 |      1 |      0 |      16 |     0 |     53 |
| **projects seen**   |    7 |         5 |      6 |     7 |    8 |      7 |       8 |      8 |      7 |      1 |      1 |      0 |       2 |     0 |      7 |

### Dimension 3: modifier_class (modifier tokens on the applicable directive)

| project             | event | key | mouse-button | v-bind | v-model |
| ------------------- | ----: | --: | -----------: | -----: | ------: |
| `wave-ui`           |    28 |   6 |            0 |      0 |       0 |
| `dho-web-client`    |    12 |   2 |            0 |      0 |       9 |
| `vue3-admin-design` |     0 |   0 |            0 |     15 |       0 |
| `vue3-antd-admin`   |     6 |   1 |            0 |     67 |       0 |
| `vue-core-vapor`    |     0 |   2 |            0 |      0 |       0 |
| `vue-jsx-vapor`     |     6 |   0 |            0 |      0 |       0 |
| `wakapi`            |     6 |   0 |            0 |     29 |       0 |
| `petite-vue`        |     0 |   2 |            0 |      0 |       0 |
| **total sites**     |    58 |  13 |            0 |    111 |       9 |
| **projects seen**   |     5 |   5 |            0 |      3 |       1 |

### Dimension 4: binding_source — declaration-site presence signals (SFC file counts, NOT per-expression attribution)

| project             | setup | props | data | inject |
| ------------------- | ----: | ----: | ---: | -----: |
| `wave-ui`           |     3 |    67 |  118 |      4 |
| `dho-web-client`    |     0 |   161 |  106 |      0 |
| `vue3-admin-design` |     7 |     0 |    0 |      0 |
| `vue3-antd-admin`   |    92 |    35 |    0 |      0 |
| `vue-core-vapor`    |   103 |    11 |    0 |      0 |
| `vue-jsx-vapor`     |     0 |     0 |    0 |      0 |
| `wakapi`            |     0 |     0 |    0 |      0 |
| `petite-vue`        |     0 |     0 |    0 |      0 |
| **total sites**     |   205 |   274 |  224 |      4 |
| **projects seen**   |     4 |     4 |    2 |      1 |

### Dimension 5: block_combination (SFCs whose top-level blocks match the combination exactly)

| project             | template-only | template-script-setup | template-script | template-both-scripts | template-script-setup-style-scoped |
| ------------------- | ------------: | --------------------: | --------------: | --------------------: | ---------------------------------: |
| `wave-ui`           |             7 |                     0 |             116 |                     0 |                                  0 |
| `dho-web-client`    |             0 |                     0 |              65 |                     0 |                                  0 |
| `vue3-admin-design` |             0 |                     0 |               0 |                     0 |                                  7 |
| `vue3-antd-admin`   |             1 |                    43 |               1 |                     0 |                                 43 |
| `vue-core-vapor`    |             0 |                    92 |               1 |                     0 |                                  0 |
| `vue-jsx-vapor`     |             0 |                     0 |               0 |                     0 |                                  0 |
| `wakapi`            |             0 |                     0 |               0 |                     0 |                                  0 |
| `petite-vue`        |             0 |                     0 |               0 |                     0 |                                  0 |
| **total sites**     |             8 |                   135 |             183 |                     0 |                                 50 |
| **projects seen**   |             2 |                     2 |               4 |                     0 |                                  2 |

## Skipped (not mechanically derived by this scan)

- **binding_source per-expression attribution** — mapping each template identifier to its declaration site needs scope analysis (the croquis engine's job). The table above reports file-level declaration-site signals only (`<script setup>` present / `defineProps`-or-`props:` / `data()` / `inject`); the `global` source has no mechanical signal and is not measured at all.
- **`v-slot` / `#` shorthand** — scanned (1671 occurrences across hydrated projects) but reported nowhere above: the taxonomy has no `v-slot` directive row today.
- **JSX plain props** — every JSX prop is an expression binding; counting them all as `v-bind` would be noise, so only `v-*` props and `on[A-Z]*` event props (counted as `v-on`, with `_modifier` suffixes matched to modifier classes) are classified.
- **petite-vue built-ins** — `v-scope` / `v-effect` have no taxonomy row and land in `custom` (the not-in-builtin-set escape hatch).
- **Lexical limits** — pug templates are scanned line-heuristically (no pug parse); wakapi's HTML interleaves Go `{{ }}` template actions that the scanner skims over; TSX start tags reuse an HTML regex (single-uppercase-letter names are dropped as probable type parameters, other generics can leak); SVG/MathML descendants count via a fixed unambiguous-name set, so namespace children whose names collide with HTML tags count as `native`; unknown `v-on` modifier tokens (custom key aliases) are ignored.
- **Element kinds in scripts** — render functions and template strings inside `.js`/`.ts` sources are not scanned; only the file classes in the scan-scope table are.

## Scope proof (assurance rule: empty means proven-empty, never silently partial)

- **Hydrated: 8 of 142 manifest projects.**

> **PARTIAL CORPUS — this report measures 8/142 projects.** Every count above, including every zero, is a statement about the 8 hydrated projects only. The remaining 134 manifest projects are **unmeasured**, not empty. Do not read dimension coverage off this report until the full corpus is hydrated (P0-6 leaves the full-coverage step open pending corpus hydration in CI).
