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
