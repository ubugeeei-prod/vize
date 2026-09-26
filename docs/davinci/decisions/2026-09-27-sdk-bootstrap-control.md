# First-publish bootstrap controls — #6895

Tracked in [#6895](https://github.com/ubugeeei-prod/vize/issues/6895).

The failed [0.429.0 Release run](https://github.com/ubugeeei-prod/vize/actions/runs/36267966910)
reports both `name` and `display_title` as
`Release v0.429.0 PR #6894 @ f59e69c38ecbead394ba30f0fdacb5f9c1b9fd04`.
Bootstrap accepts the historical name `Release` or the exact candidate title
only after validating its release tag, positive PR number and full source SHA.
Event, workflow path, run ID, branch, repository and terminal failure checks
remain unchanged. A matching arbitrary name/title does not qualify.

Bootstrap controls run from the exact current main dispatch SHA. The immutable
production tag must remain on main's first-parent history, and current main
must still own the same workspace release version. A newer release version,
moved main, non-main dispatch or unrelated tag is rejected. Read control
`Cargo.toml` from the validated main SHA, not mutable checkout contents.

Read production workspace/package manifests from the immutable tag SHA.
Release run and downloaded artifact identity remain bound to that same tag,
run ID, branch and version. Preserve every successful prerequisite, target
publish failure, expected skipped job and unexpired unique artifact check.
Repairs do not retag, rebuild or substitute production artifacts, publish a
package, relax authentication permissions or bypass CI/merge-queue gates.

Recover using the ordinary reviewed repair PR, then dispatch the bootstrap
from its validated current main while holding other merges through preflight.
Use the original Release run and smoke-tested package artifact; follow the
[first-publish handoff procedure](../../release/supply-chain.md#first-publish-bootstrap).
