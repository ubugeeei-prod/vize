# Retire the unpublished v0.436.0 catalog candidate

Tracking: [#6239](https://github.com/ubugeeei-prod/vize/issues/6239) and
[#6830](https://github.com/ubugeeei-prod/vize/issues/6830).

The immutable v0.436.0 source passed its five qualification workflows, and
metadata PR [#8250](https://github.com/ubugeeei-prod/vize/pull/8250) passed its
protected checks and actually merged. The official resume still stopped before
publication: the integration catalog differed from the qualified source catalog.
Those successful checks therefore do not establish a successful release.

## Exact failure

The unchanged `tools/support/release/pr_pin_catalog.rs` comparator found exactly
one difference in the complete catalogs:
`/npm/vize/manifest/scripts/test`. The delivered n8n configuration fix
[#8260](https://github.com/ubugeeei-prod/vize/pull/8260) legitimately added
`tests/define-config.test.ts` to that script after the source was frozen.
Every other catalog field was equal. The comparator includes the complete
public npm manifest; shipping-code equality alone cannot satisfy that contract.

| Identity                            | Value                                                                                               |
| ----------------------------------- | --------------------------------------------------------------------------------------------------- |
| Source before versioning, C         | `6d26d84b4240e9356cf5078b6d277e181d311a42`                                                          |
| Frozen versioned source, H          | `3390cb6044d53d655a9d64e112d2618375cff5ca`                                                          |
| Signed delivered integration, V = G | `f9cfdaa0e84152ab9f11382e1b49e501d1ea44b6`                                                          |
| Release run, R                      | [37717489915, attempt 2](https://github.com/ubugeeei-prod/vize/actions/runs/37717489915/attempts/2) |
| Source PR                           | [#8249](https://github.com/ubugeeei-prod/vize/pull/8249)                                            |

Keep strict complete-catalog equality. Retain the valid n8n test, the frozen
source, durable pin, signed integration and historical failure. No manifest
field is removed from comparison, and no tag is moved or manually published.

## Terminal retirement

The release run became terminal cancelled at 2026-10-08 06:18:38 UTC. Its 25
successful qualification/build jobs and all 41 artifact identities, sizes and
retention records remain intact. All 21 publication dependents were cancelled
with zero executed steps.

The 06:23 UTC channel audit found all 26 planned npm versions and all 26 planned
crate versions absent. Marketplace had no v0.436.0; Open VSX and GitHub Release
had no v0.436.0. The release tag and operator reference were absent too. This is
an observed unpublished state, not a claim that the previous public release has
passed the newly added acceptance tests.

Source PR #8249 was closed draft and unmerged at 06:25:11 UTC with its H, body and
durable pin unchanged. Its
[retirement receipt](https://github.com/ubugeeei-prod/vize/pull/8249#issuecomment-6053803901)
records `ABANDONED_UNPUBLISHED`. Metadata already delivered on main stays in
history. Closing the abandoned source also releases the official version-owner
lock; it supplies no P0 completion or release-success credit.

## Next release

Add a required read-only integration guard before the next cut. It must compare
the unchanged complete H catalog against the authenticated integration PR merge
snapshot and its own protected merge-group candidate. Ordinary PRs must finish
the guard explicitly; the required report must consume its result. Cover this
exact test-script drift, a matching candidate and candidate-identity refusals.

After that guard actually merges, use the official minor release flow for
v0.437.0 from fresh main. Generate a new frozen source, full qualification and
build run; the retired H's artifacts cannot qualify the new source. Verify tag
identity, every publication channel and installed original issue reproductions
separately. Further catalog drift must fail before integration delivery.

These guard and publication requirements remain pending in this record. The
broader release, n8n adoption and P0 tracking issues remain open. Upstream n8n
remains read-only.
