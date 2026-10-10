# CRLF fixture filter-warning setup

Tracking issue: #6830.

## Failure and decision

PR #8461 source `c936b7a22582ef306fe1d1e2ad6b134b38aa5f20`, based on
actual main `cb9afd9298bc74a852b0ea6a14e45d5b723db054`, failed the existing
hydration law in Check `38033941359`, tooling shard 2 job `114160793900`.
The exact CRLF bytes already matched, but `git diff --name-only` returned
an empty string instead of the required `authored.json` filter-warning path.
The original law reproduces locally with Git 2.55.0 and Node 24.14.0.

Changing `.gitattributes` does not change the authored file's cached stat.
Git can reuse that clean entry without hashing its bytes through the new
`text eol=lf` filter. A racy index timestamp can instead force the content
check, making the fixture's warning path depend on setup timing.

Give the authored file a fixed old modification time before the original
commit, then clear only its cached stat before the original dirty-path
expectation: reinsert the exact committed `100644` mode and `HEAD` blob
with `git update-index --cacheinfo`. The test proves the indexed mode/blob
remain identical and the index still has no staged differences.

The original CRLF bytes, dirty-path expectation and production checkout
validator are unchanged. Preserve every existing refusal for changed raw
bytes, a staged change, executable mode, missing path and symbolic link.
No corpus input, fixture exclusion, production Git policy or gate changes.

## Verification and remaining work

With the same old timestamp but no stat-cache invalidation, the unchanged
`authored.json` expectation fails. With invalidation, all three original
hydration laws pass, including the raw-byte and clean-index guards.
The original Actions failure remains retained; a retry is not the fix.

Fresh exact-head Actions, protected merge-queue qualification and actual
merge remain required before this CI repair is delivered.
