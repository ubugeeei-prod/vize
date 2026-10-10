# Native slot Stack delivery

Tracker: [#8142](https://github.com/ubugeeei-prod/vize/issues/8142).
Producer: [#8386](https://github.com/ubugeeei-prod/vize/pull/8386).
Configured policy: [#8422](https://github.com/ubugeeei-prod/vize/pull/8422).

The installed CLI adapter in [#8357](https://github.com/ubugeeei-prod/vize/pull/8357)
merged as signed `195980929d776d0106b064d19ecae70f8d43cc0c`. The default-slot
producer in [#8359](https://github.com/ubugeeei-prod/vize/pull/8359) merged as
signed `d6610e63122ecb89bfdf72fb6bb8c73fe1f71c90`. Both identities are verified
GitHub merge commits. Their installed-public and remaining adoption obligations
continue independently of those source deliveries.

The dynamic-binding head `fb463308fd4c393f318a5b035a7ac01baa69d1bc` passed
[Check 38026773009](https://github.com/ubugeeei-prod/vize/actions/runs/38026773009).
The configured-policy head `55558136d9f099cfd118348235d1d8aa4928497b` passed
[n8n adoption 38028964548](https://github.com/ubugeeei-prod/vize/actions/runs/38028964548),
including all 75 CLI observations and 50 JSON-RPC diagnostic observations. The
raw wire, complete actual and expected packets, authored input hashes and
build-receipt identity are retained in that run's authored-editor artifact.
Its qualified packet has SHA-256
`cbced5314301a44f4c9e2d678c9233c4191bafbff53e085301d64bdeef5b8acf`;
artifact `11661097696` has GitHub archive digest
`dcf5e475597d77cd4cb5e402dbce266abd9f3dab4c3cae1f68773eb3f2987f6f`.
These observations qualify those historical source identities only.

## Recovering a merged-prefix base

Native Stack #8387 originally contained #8359, #8386 and #8422. After #8359
merged, #8386 retained its former branch as its physical base. The queue entry
was unmergeable and had no prospective merge head. A read-only merge-tree check
against its actual predecessor `042f24370b08031f0e281d0b7b5530ae8265b5e7`
identified real source conflicts. An unchanged rule body or an old green run
could not qualify that queue projection.

Fence all owning worktrees at their recorded clean heads, and verify that each
active member has neither a queue entry nor auto-merge enabled. Use an isolated
clean clone to check out the actual remote Stack. Official `gh stack sync
--remote origin` adjusts source ancestry for the merged prefix and uses leases
for the branch updates. Here it produced #8386 head
`07e250de2bdf1f50f882aa7ca7def7b0eec47ba6` and #8422 head
`7715be611bb3383ff1d03c4696be3b33e0667ff7`, based on actual main `042f2437`.
The source histories were reconciled, but the physical Stack base still named
the former #8359 branch.

The documented [GitHub Stack API](https://docs.github.com/en/rest/pulls/stacks)
supports unstacking the open members while retaining merged members as history.
With the same clean-head and queue fences, the recovery was:

1. `gh stack unstack 8387` and inspect the remote result. The closed historical
   Stack must retain merged #8359 alone, and both open PRs must have no Stack.
2. `gh pr edit 8386 --base main` after verifying that #8386 already contains
   actual main and that its exact source head has not changed.
3. `gh stack link --remote origin 8386 8422` to register the ordered open prefix.
4. Inspect every actual PR head, physical base, Stack number, position and queue
   state through GitHub. New native Stack #8426 has #8386 at position 1 on main
   and #8422 at position 2 on #8386; historical Stack #8387 retains #8359.
5. Remove obsolete local tracking with `gh stack unstack 8387 --local`, then
   `gh stack checkout 8426`. Keep the historical remote Stack intact.

An unstack warning can also accompany retained merged members. Treat the remote
member and queue receipts as the authority before taking the next action.
Never overwrite an active owner's branch or transfer source qualification to a
changed head. A base edit emits `edited`, which does not run workflows configured
for the default pull-request event types. This delivery record is an authored
source change on #8386; cascading it through #8422 with the official Stack rebase
and lease-checked push starts fresh `synchronize` checks on both exact heads.

Queue only the current-head-green contiguous prefix through `gh stack merge`.
Verify the actual signed merges and any server-side restack before qualifying a
successor head. The frozen 0.440.0 release source excludes these slot changes.
Installed CLI and LSP policy acceptance must use a later public source tuple that
actually includes them. n8n upstream remains read-only, and #8142 stays open for
its remaining whole-project and public-product requirements.
