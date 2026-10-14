# Native phase evidence ignores unrelated tooling changes

Issue: [#6830](https://github.com/ubugeeei-prod/vize/issues/6830).

## Decision

The `typechecker-native-phases.yml` PR path filter names the one tooling
module its steps load, `tests/tooling/davinci-storage-rust-syntax.ts`, instead
of `tests/tooling/**` and three exclusions. Every other source path, the
manual `source_sha` dispatch that release uses, and the workflow's checks are
unchanged.

## Reason

The workflow is not a required check, yet `tests/tooling/**` started its
13.7-minute job on almost every PR, because most PRs add or change a tooling
test. Only that one module is loaded by the workflow's steps and the scripts
they run. The tooling tier of the Check workflow still runs every tooling
test that reads this workflow.
