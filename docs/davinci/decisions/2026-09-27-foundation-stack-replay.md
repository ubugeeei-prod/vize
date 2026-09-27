# Foundation stack replay after CI repair

Issues: [#6831](https://github.com/ubugeeei-prod/vize/issues/6831) and
[#6832](https://github.com/ubugeeei-prod/vize/issues/6832). Preparation only;
publication and fresh Actions are controlled separately.
The [publication parent](./2026-09-27-foundation-publication-parent.md) records
the subsequent replay onto the selected CI tiers without rewriting this receipt.

CI parent: `1065e2dcc3592a025bfbc7689b80d88dd9c97a34`. This parent retains
current main changes and validates PRs against stacked base branches. Foundation
work follows CI speed work; its prior green runs do not certify this new head.

## Dependency gate

The following existing decisions moved verbatim from the central record to
keep that entry point within its 350-line cap:

- The initial #6831 gate scopes physical level/conversion packages and the
  owners of production aliases (L0/Carton and L3/Impeto today). It reports three
  existing normal legacy entry edges and five derived root-to-entry paths.
  Entries and their supported root sets only shrink; target, optional and
  rename variants are distinct. Dev oracle and build kinds are kept separate.
- The declaration gate runs on every PR, merge group, push and full check,
  including docs-only allowlist changes. Additions, stale permissions and
  missing identities fail. Physical L4 and external resolved closure remain
  unproved; #6831 stays open until the allowlist is empty and enforcing.

The two replayed commits are full stable-patch-identical to their originals:

| Original                                   | Replay      | Stable patch id                            |
| ------------------------------------------ | ----------- | ------------------------------------------ |
| `99a5e3403aca86582832f9aeb219330139371db3` | `fce86c070` | `0c982bc301a5fb55bf45ed6b0a129d63b4246f80` |
| `1dade31b7871db7dc5bd057af7ef0710e3e3678c` | `8b16f20c6` | `e6a50292c4740542921aeeca380f5102b1341b99` |

Existing dependency, stage identity, workflow and strict aggregate contracts
passed all 60 tests using actual locked Cargo metadata. No local production
build or workspace compilation ran. The required Check workflow keeps the CI
parent's PR-base handling and merge-group suites; only the declaration gate
and required aggregate dependency are added semantically. Existing comments
were shortened by the original patch.

## Physical L3 layer

The package move retains 56 R100 paths in a move-only commit. The generated
consumption shard move retains one R100 path in its own commit. Reference,
manifest, immutable published-semver baseline and shard-identity edits follow
separately. All six replayed patches have identical full stable patch ids:

| Original     | Replay      | Stable patch id                            |
| ------------ | ----------- | ------------------------------------------ |
| `c5cf11dd9`  | `15e0abcba` | `d39a2272e29030aa9c941ad95b8261fecbb582a1` |
| `7c00920e5`  | `5519e31d2` | `109c975611e30e3a060ee7b5de5e8f506f041979` |
| `574a9da415` | `37e087927` | `6a738da5b30bcd980875122a706257fcd31f286c` |
| `7fee1759f`  | `7a4ef2e24` | `7401c993b3d4683fb9a345c1f36458b5f5333636` |
| `1d08c324`   | `57c2e868d` | `b20dd7da2e458c5048aba18a07bba52d2a302176` |
| `aa0b3de2`   | `c86bedb52` | `4996fee917e880dbee57f5a650297ef728477188` |

The stability table correction `aa0b3de2` is included in this layer. Its later
copy `cabf4a7a` has the same stable patch id
`4996fee917e880dbee57f5a650297ef728477188`; the stage layer must not apply it
twice. No original worktree or commit was changed.

Existing metadata, dependency, workflow, stability, module and storage
contracts passed all 73 tests. Croquis consumption (32 files), consumer
migration (19 files), rule parity and storage summary checks passed without
regeneration. No derived cross-count change was necessary. The central record
retains current CI, Nuxt and SSR decisions and is 347 lines at this layer.

## Stage path layer

The stage move has 130 R100 paths in its own move-only commit. References,
authored playground import and guarded Rust test registration follow in
separate commits. The four patches preserve their full stable patch ids:

| Original    | Replay      | Stable patch id                            |
| ----------- | ----------- | ------------------------------------------ |
| `26c0ae4dd` | `11eb7d5c7` | `bcdfccea2de28de32cc21cb4f03deecd54af772e` |
| `985a9a81c` | `6edd18882` | `210773f81ddfc4ef9b932292393330d61f4e29be` |
| `9eeea29b1` | `e27928f33` | `ecc24ddc91212d231b9aa51009afd24bf748618c` |
| `9a0d6219`  | `081120632` | `085277b3ee7ff5ca50ee5e824c9f343e7ffa6882` |

Existing dependency, stage, workflow, stability, module and storage suites
passed all 73 tests on this layer. The rename driver's verify mode reports
zero remaining reference rewrites. All four inventory checks passed without
regeneration. Actual locked/offline metadata resolves 38 workspace packages
and 782 targets; the current main adds three regression targets compared
with the old stack's 779. The single `davinci_l2_filters` target still binds
`tests/l2_filters.rs` behind the original `legacy` feature; no duplicate
unguarded `l2_filters` target is registered. An initial manual probe guessed
the wrong feature; the corrected probe compares the original manifest.

Apart from the stability-table formatting described below, the final tree
differs from original #6907 only by repaired CI/main changes and this proof.
All other non-overlap files match the current CI parent byte-for-byte. Parent/child overlap is limited to the Check workflow
and central record. Parsed Check YAML equals the exact three-way semantic
composition of old CI `de61316b9`, original #6907 `9a0d6219` and repaired CI
`1065e2dcc`; stacked-PR handling, full merge-group suites and required gates
are preserved. Central decision details moved verbatim to this companion;
current main's Nuxt, SSR and CI records remain intact.

Final formatting found Markdown column alignment left by the shorter L3
name in the original stability correction. Formatting changes table padding
and separator widths only; parsed table cells and every other line remain
identical. The style correction is on the physical-L3 layer so its standalone PR can
pass formatting. Cargo formatting passed. Changed-file formatting covers 113 files,
and source line/whitespace ratchets remain enforced.

TODO: publish the three layers after CI priority is resolved, mirror their
decisions on #6831/#6832 and validate fresh exact-head Actions and actual
merge-group results. No local full production build, browser/WASM run or
new native acceptance is claimed. Three legacy exceptions remain. Naming,
serialized vocabulary, CLI dump and the later restructure stay unfinished.
