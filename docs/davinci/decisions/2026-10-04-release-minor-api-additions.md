# Native API additions and the next public release

Issue: [#6239](https://github.com/ubugeeei-prod/vize/issues/6239).

The unpublished patch candidate [#7584](https://github.com/ubugeeei-prod/vize/pull/7584)
cannot ship the current native API additions as `0.430.2`. Its exact-head full
Check [37143771558](https://github.com/ubugeeei-prod/vize/actions/runs/37143771558)
rejects the new `NativeSfcCompileOptions.scope_id` field, exhaustive
`NativeSfcCompileError` variants, and
`UnsupportedReason::OriginalForProviderUnavailable`. Rust callers can construct
the former options with a struct literal or exhaustively match those enums.

Preserve the implemented native capabilities and prepare `0.431.0` with
`vp run release minor -y`. In the `0.x` line, a minor bump supplies the required
breaking-release classification; the SemVer gate, baseline, required jobs and
release identity checks remain unchanged. This decision removes no published
item, feature, command or export and does not waive the support policy's
deprecation window. Davinci remains experimental, and this release does not
claim the native products or their fix-history gates are complete.

The public release notes include the struct-literal and exhaustive-match
migration steps, alongside the existing community reporter acknowledgements.

The supported release command binds a candidate to its `release/vVERSION`
branch and tag. Supersede #7584 with the command-generated minor candidate and
link both PRs rather than editing its immutable version identity. Wait for the
current delivery batch to merge before starting promotion. If main advances,
the command must regenerate the candidate and repeat exact-head validation.

Completion requires successful full Check (including `test-scripts` and all
SemVer jobs), Miri and Docs evidence on the final candidate. Require successful
Fuzz replay and Real Project Matrix evidence; a version-only release commit may
reuse its parent's successful evidence for those two gates, as permitted by the
[release workflow contract](../../release/pr-workflow.md). Then promote main
and the tag atomically and complete terminal publication.
Verify the public GitHub Release, native/editor assets, exact npm and crates.io
versions, VS Code Marketplace and Open VSX before reporting publication. A
draft candidate, green build or existing tag alone grants no completion credit.
