# Native verification after release metadata finalization

Paired issue: [#8335](https://github.com/ubugeeei-prod/vize/issues/8335), with the
[same decision](https://github.com/ubugeeei-prod/vize/issues/8335#issuecomment-6086337052).
The [qualification update](https://github.com/ubugeeei-prod/vize/issues/8335#issuecomment-6086426334)
preserves the failed old cut as historical evidence and requires a repaired new cut.

## Failure and decision

Historical source PR #8341 retains immutable H
`f58ba3592f378a6a40eea75b3d8040c6ad26ea0f`, cut C
`fbb31120ae0c6213f4d3ed2dbead67321e5a39a3`, and integration PR #8342.
Native workflow run `37967292545`, attempts 1 and 2, fails before native
tests with `Missing/ambiguous vize-release-integration`. Its opened event
body predates the final integration marker; rerunning retains that payload.

Add `edited` to the native workflow's existing `opened`, `synchronize` and
`reopened` PR event types. An actual final body update can then deliver the
complete metadata to the unchanged authenticated source-recipe selector.
All paths, source/head expressions, steps, permissions, helper bytes, native
cases and budgets remain unchanged. No incomplete release falls back to an
ordinary PR recipe, and no job is skipped to obtain success.

The existing same-PR concurrency may cancel an older run when a title/body
edit starts its replacement. Each replacement executes the same full checks;
cancellation is not successful qualification. Other PRs and manual dispatches
retain their separate groups. An edit using `GITHUB_TOKEN` does not create a
new workflow run: delivery must use the approved real user authentication and
must be observed on a newly prepared repaired immutable cut. The separately
failed old source remains immutable and is not refreshed for qualification.

## Regression and delivery

The TypeScript tooling regression uses the existing real-Git version-cut
fixture and unchanged selector. The same source/cut/pin rejects the stale
opened payload twice; a fresh finalized edited payload selects the complete
original recipe, hashes and environment. Ordinary PR events retain their
inline path. Missing/duplicate integration metadata, pin mismatch, foreign
head and insufficient permission remain rejected. These are reconstructed
fixture identities, not relabelled historical H or native runtime evidence.

Before the event change the new regression fails because `edited` is not
delivered. After it, both new laws and all seven existing source-recipe laws
pass (9/9). The repository formatter/linter reports no warnings or errors.
Fresh exact-source Actions, independent review, protected merge and the actual
edited native run on the repaired new source cut remain required. Root owns
queue admission and the user-authenticated body update after official fresh-cut
preparation. Historical H, release run and pin are retained; #8335 and third-party
delivery remain open until successful publication.
