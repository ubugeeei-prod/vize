# CI for stacked pull requests — #6830

The Check workflow previously accepted pull requests only when their base
was `main` or `davinci`. A native stack changes intermediate PR bases to the
preceding branch, which prevented fresh Check runs when those heads changed.

Accept every pull-request base in Check. Preserve the main merge-group event,
required aggregate, source selection, permissions and validation commands.
Every stack layer can now receive a check for its own diff and checked-out
merge tree; a successful parent run does not certify a changed child.

The title policy also accepts every base. It continues to check out only the
default branch under `pull_request_target`, so extending the trigger does
not execute candidate code with the policy's write token.

This enables validation of the CI and level-restructure stacks. It does not
establish GitHub automatic rebasing, remove any source suite, or satisfy the
T0 timing target. Keep squash auto-merge gated on successful PR checks and
verify the actual main merge-group result.
