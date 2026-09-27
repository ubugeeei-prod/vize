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

TODO: replay the physical L3 and stage-path layers in separate move-only and
reference commits; record their patch identities, metadata and inventory
checks here. Publish only after the CI priority is resolved, mirror decisions
on the owning issues, and validate fresh exact-head Actions and merge-group
results. Three legacy exceptions remain, so #6831 and #6832 stay open.

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

TODO: replay stage paths after this layer, publish only after the CI priority
is resolved, mirror decisions on the owning issues, and validate fresh
exact-head Actions and merge-group results. Three legacy exceptions remain;
#6831 and #6832 stay open. No full local production build ran.

## Independent dependency-gate coordination

The root reviewed a metadata-only split of the existing eight: #6905 becomes
a standalone main-base PR; the seven existing #6906 -> #6907 -> #6943 ->
#6944 -> #6945 -> #6946 -> #6947 members retain their order in a main-base
native stack. All are positively verified off queue with unchanged source
heads before this graph operation. This adds no PR, member or source edit.

The #6905 source head is `a722f0be290c0d76feb6012bc1608b1561cbdf46`.
Its thirteen owned paths contain dependency policy, workflow contracts, tests
and documentation, with no Rust product, Cargo, benchmark, harness or protocol
change. Current actual main is `f67358d0c283f6211e52080f455285fdb62c95f1`.
The root reviewed the generated gate-only transport tree `d146`; publication
and fresh exact-source acceptance are still pending. Check remains 690 lines.

The original actual queue candidate
`5d44add8574cd2ff496258052e51454325d0e1e9` on main `30c5af85` passed
[full Check 36319176710](https://github.com/ubugeeei-prod/vize/actions/runs/36319176710),
including measured instruction job `108619572354`. Its raw artifact
`10932275565` has SHA-256
`b3f600fa12bae3788be25db5d1083af7344665469c7af4f10a11a89872d5f944`.
The latest original-head required source contexts also pass. These historical
receipts do not accept a new source or queue candidate on advancing main.

TODO: perform the reviewed graph split with exact remote-head leases and
off-queue checks; independently transport, validate and actually merge #6905
through the protected queue. The remaining seven require concretely reviewed
production repairs and fresh full measured queue acceptance before their
actual merges. Their current failure artifacts and authored source remain
preserved; private future naming is separate. Three normal legacy exceptions
remain, so #6831 and #6832 stay open. No numeric, native or main credit is
granted by coordination alone.

## Actual independent first-member merge

The reviewed graph split is complete. [#6905](https://github.com/ubugeeei-prod/vize/pull/6905)
actually merged at `2026-09-27T14:33:05Z`, retaining authored source
`a722f0be290c0d76feb6012bc1608b1561cbdf46`. Its actual queue candidate and main
are `7553eca45a4ed6baec08da23a2b237d800e90ef8`, tree
`45c49999d0a16ba195deac90d90bff67b6b60972`, parent actual `ae3c16b`.
The thirteen owned paths add no Rust product or benchmark change.

Fresh [queue Check 36325334512](https://github.com/ubugeeei-prod/vize/actions/runs/36325334512)
passes the full required suites and measured 100 targets in three equal
executions each, under the unchanged ceilings and guest method. Instruction
artifact `10933549361` has SHA-256
`0fe8b545caacc3b9b03511933304bc682b808eddeef7686f96393c8fe1216e5e`.
This is new candidate evidence, distinct from historical `5d44` acceptance.
The source-bound tooling CLI records 5246 passes, zero failures/cancellations
and twelve existing skips. Named LSP document-link and CRLF formatting controls
pass; this does not establish whole LSP history coverage or native facts.

[Post-main Check 36326323322](https://github.com/ubugeeei-prod/vize/actions/runs/36326323322)
and all six same-head workflows succeed. The gate reports six roots, three
existing legacy entries, five derived witnesses, no unlisted or stale entries.
Physical L4 and resolved external transitive closure remain unavailable.
The immutable final evidence bundle's receipt SHA-256 is
`7c3c426c3b7f0723f0ec13da5ad6d8abf75088e00061068a5898d82841d33a3b`.

The seven unchanged original source heads now form native stack `6971`, in
#6906 -> #6907 -> #6943 -> #6944 -> #6945 -> #6946 -> #6947 order, with
verified main base, all off queue and no auto-merge requests. Their production
performance repair, fresh source checks, full measured queue and actual main
merges remain TODO. No tail performance or native acceptance follows from the
first member's merge; #6831, #6832 and #6826 remain open with no closing refs.
