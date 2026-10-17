# Consumer recovery authority and run timestamps (#6830)

Recovery run `38990889311`, attempt 2, at
`78b75fb151c115ab8f5043e99424c0e55a562081` completed all six consumer replays.
Its finalizer job `117274555191` then refused `original signed integration
changed`: GitHub returned newer `pushed_at` and `updated_at` values inside the
integration PR's `head.repo` and `base.repo`. These clocks describe activity in
the repository, not a change to the historical signed integration.

Prepare artifact `12085630438` and failed-finalizer artifact `12084743637`
retain the same complete `integration.json` (144,236 bytes, SHA-256
`794f583ecbc5cbba07013e0c6806153106e0197d1febf6b870d3df3eb95aeaf2`). The
fresh response that failed comparison was not separately saved. The log records
`pushed_at` changing from `01:30:03Z` to `01:34:38Z` and `updated_at` from
`01:25:44Z` to `01:33:25Z`, all on 2026-10-17. A timestamp-mutated copy of the
retained packet is a regression control, not a recovered final API response.

Main change #9325 at `d57cf724d2caea6cf47f814250d18801f6b75e15` supersedes
the initial integration timestamp comparison. Use its canonical
`integrationIdentity` and authenticate both stored and current observations
against the original tag with `verifyStoredIntegration`/`verifyIntegration`.
The compared authority includes PR/repository/head/base identities, exact pin
markers, merge SHA/tree/parents/signature, and every qualifying exact-main push
Check's identity, attempt and terminal result. The latest such Check must pass.
PR prose, repository activity metadata and unrelated Check runs are not delivery
authority. This is an explicit integration policy, not a general field filter.

Keep main's complete raw before/after observations, including
`integration-after.json`, in the signed file inventory. Verification checks
those stored packets and the freshly fetched integration again. Main's policy,
retained drift fixture and integration negative-control suite stay byte-exact;
the fixture labels its later packet as an independent live follow-up, not the
un-emitted failed-job response. Remove the superseded integration comparison
helper and its redundant assertions, including obsolete expectations that
non-authority PR prose or timestamps must compare exactly.

Original Release and replacement run rechecks retain the narrow comparison-only
copies: ignore only string values of `pushed_at`/`updated_at` directly inside a
run's `repository` and `head_repository`. Presence, types, all other repository
fields and non-repository timestamps remain exact. Run/source/attempt/event,
jobs, failed-job details, logs, policy, artifacts, tags and source receipts keep
their existing checks. Neither comparison rewrites raw custody packets or their
file hashes. No publication or source closure behavior changes; the original
Release remains failed.

The focused integration, run-custody, original recovery, archive, public identity,
JSR and installed-acceptance suites pass (187 tests), as do both hosted-operator
controls. Strict TypeScript and scoped formatting/lint checks pass. The original
500-line recovery test and its retained fixtures remain unchanged. Hosted
finalization and signing still require qualification after the merge lands;
the already-running Full Check at `2b0cf6c` is evidence for that earlier source.
