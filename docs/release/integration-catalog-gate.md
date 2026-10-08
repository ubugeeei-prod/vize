# Qualify pinned metadata before merging

A pinned release's generated metadata can remain valid while ordinary PRs
change its complete publication catalog. For example, v0.436.0's frozen
`vize` manifest lacked the later `tests/define-config.test.ts` entry in its
`scripts.test` value. The generated version delta passed, but the final
catalog differed at `/npm/vize/manifest/scripts/test`. Preserve that strict
catalog comparison; prepare a new source cut when it differs.

Check's `Release integration catalog` job authenticates the official metadata
PR's source/body, maintainer, durable pin, and exact candidate before merging.
For a PR it checks the current GitHub synthetic merge commit and its two
parents. For its own merge group it checks the event's exact candidate, single
parent, and `gh-readonly-queue/main/pr-PR-PARENT` branch. Ambiguous or batched
delivery projections fail closed. The existing required `test-report` consumes
this job's result; a failed, cancelled, or skipped job cannot authorize a merge.
Ordinary PRs and batched queue roots finish the lightweight event selection
without building a catalog or imposing release-only parent constraints.

The integration PR opens before source initialization finishes. A check that
observes an absent backlink or pin fails closed; rerun it after initialization.
Do not omit missing custody to make initial checks green.

The release operator may also collect a read-only receipt before admission.
Check out the current metadata PR's synthetic merge SHA, then run:

```sh
rust-script tools/commands/release/pr.rs verify-integration-candidate \
  INTEGRATION_PR CANDIDATE_SHA BASE_SHA INTEGRATION_HEAD_SHA
```

The JSON receipt binds those exact commits and the frozen source. It does not
admit the PR, replace its later own merge-group check, qualify source/build
artifacts, or prove publication. The existing post-merge signed-delivery and
immutable-tag verification remain mandatory.
