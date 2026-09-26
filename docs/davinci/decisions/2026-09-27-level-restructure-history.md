# Level Restructure — Legacy Fix History and Deletion (2026-09-27)

This page is part of the canonical
[decision record](./2026-09-27-level-restructure.md). It retains the decisions
from the same maintainer design session.

## Legacy deletion criteria

Tracked in [#6828](https://github.com/ubugeeei-prod/vize/issues/6828) and
[#6852](https://github.com/ubugeeei-prod/vize/issues/6852)–[#6854](https://github.com/ubugeeei-prod/vize/issues/6854).

1. Every legacy fix adds its input to the differential corpus. The merge
   queue checks that the Davinci lane matches.
2. Zero fallbacks across the corpus, every dialect and the real-project
   corpus.
3. Rules 1 and 2 hold for a stability period before deletion.
4. Past fix history becomes fixtures for each product before Davinci
   replaces that product's legacy path. Rule 1 ([#6852](https://github.com/ubugeeei-prod/vize/issues/6852)) covers fixes from
   now on; these issues cover the past:

   | Product      | Issue                                                      | Fix commits in history |
   | ------------ | ---------------------------------------------------------- | ---------------------- |
   | Type checker | [#6879](https://github.com/ubugeeei-prod/vize/issues/6879) | 271 of 354             |
   | Compiler     | [#6880](https://github.com/ubugeeei-prod/vize/issues/6880) | 609 of 1152            |
   | Linter       | [#6881](https://github.com/ubugeeei-prod/vize/issues/6881) | 255 of 499             |
   | Formatter    | [#6882](https://github.com/ubugeeei-prod/vize/issues/6882) | 56 of 87               |
   | LSP          | [#6883](https://github.com/ubugeeei-prod/vize/issues/6883) | 238 of 421             |

The criteria apply per product: compiler output, lint diagnostics, formatter
output, type-check diagnostics and LSP snapshots. Acceptance rates are
published for native Davinci work only.

### Differential corpus

Tracked in [#6891](https://github.com/ubugeeei-prod/vize/issues/6891)
(shared harness), [#6892](https://github.com/ubugeeei-prod/vize/issues/6892)
(dialect coverage) and [#6893](https://github.com/ubugeeei-prod/vize/issues/6893)
(automatic reduction).

**Current state (2026-09-27):**

- 146 real projects are pinned in `tests/_fixtures/_git`, listed in the
  `vue-ecosystem-fixtures.json` registry.
- The nightly real-project matrix (22 shards) runs the compiler, type
  checker, linter, formatter and LSP lifecycle. It checks that the tools run
  (exit codes, parse errors), not legacy-vs-Davinci parity.
- A byte-level legacy-vs-Davinci comparison exists only for the compiler.
- In-repo fixtures: 442 `.vue` files in `tests/_fixtures` and 95 in
  `tests/fixtures`.

**Decisions:**

- **One shared differential harness serves every product.** It runs the
  legacy lane and the Davinci lane on each input, and only the comparator
  changes per product:
  - compiler: byte-identical output per target
  - linter: identical diagnostics and fixes
  - formatter: identical output
  - type checker: identical diagnostics (position, message, code)
  - LSP: response snapshots

  It emits one result format, which also feeds the native-only acceptance
  rates ([#6853](https://github.com/ubugeeei-prod/vize/issues/6853)).

- **Tiers:**
  - In-repo fixtures run in the merge queue (T1). This includes the
    fix-history and value-sensitive fixtures.
  - The 146 real projects run nightly (T2) for every product.
- **Dialect coverage is recorded in the fixture ledger** for each fixture:
  Vue version, petite, pug, JSX dialect and quirks.
  - Gaps are filled with real OSS projects where they exist (for example
    Vue 2).
  - Rare dialects (Vue 0.x and 1, quirks) get corpora generated from the
    grammar.
  - The zero-fallback gate
    ([#6854](https://github.com/ubugeeei-prod/vize/issues/6854)) is measured
    per dialect.
- **Nightly differences become fixtures automatically:**
  1. `vize reduce` minimizes the input.
  2. The minimal input is committed as an in-repo fixture that runs in the
     merge queue.
  3. An issue is filed for an agent to pick up.

  Minimizing keeps fixtures small and avoids copying third-party files
  wholesale, which also avoids license issues.
