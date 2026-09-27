# Foundation stack replay after CI repair

Issues: [#6831](https://github.com/ubugeeei-prod/vize/issues/6831) and
[#6832](https://github.com/ubugeeei-prod/vize/issues/6832). Preparation only;
publication and fresh Actions are controlled separately.

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
