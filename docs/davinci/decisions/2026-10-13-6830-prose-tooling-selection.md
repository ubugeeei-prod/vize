# PR tooling tests select prose by name

Issue: [#6830](https://github.com/ubugeeei-prod/vize/issues/6830).

## Decision

When a PR changes Markdown (`*.md`, `*.mdx`), the tooling test planner selects:

- tests with a declared scope whose inputs cover the file, as before, including
  the broad inputs a declared scope keeps when its import closure is incomplete;
- any other test whose resolvable sources name the file, a directory above it,
  or a glob that matches it.

Markdown no longer counts as an unknown input, and an undeclared test with an
unresolved import no longer runs for prose it never names. Code changes and the
merge tier are unchanged: the merge queue still runs every tooling test.

The `typechecker-native-release-inputs` ledger is retired with its guard. It
pinned the bytes of the planner and its configuration, so every planner change
had to re-pin it. The three path exclusions it guarded in
`typechecker-native-phases.yml` stay: they only keep three release tests from
starting that optional workflow.

## Merge queue

Every merge-group candidate selects the complete Rust, JS package, tooling
and playground suites, including candidates that change only prose. The
optimization applies to PR selection only. Each candidate must satisfy the
existing complete-Rust gate independently of its predecessor's qualification.

The initial prose exception caused the docs-only candidate for
[#8898](https://github.com/ubugeeei-prod/vize/pull/8898),
`273d14ed0a22fd4056d56ab7cd3d79353c5ef54a`, to select `rust=false`.
The [Rust source report](https://github.com/ubugeeei-prod/vize/actions/runs/38435064811/job/115430252462)
correctly rejected the incomplete merge-group plan. Restoring unconditional
merge-group selection fixes that defect without weakening the guard. The
regression coverage includes docs, MDX, changesets and root Markdown, plus a
real docs-only Git comparison through the planner CLI.

## Reason

A change to `AGENTS.md` or one guide page selected 1,048 to 1,087 of 1,247
tooling tests, which is the full tooling tier: Markdown outside `docs/` was an
unknown input, and every test without a declared scope used the whole tooling
input set, which includes `docs/**`. With this change the same edits select 40
and 130 tests.

## Tradeoff

A test that reads prose through a computed path it never spells out is no
longer selected on the PR; the merge queue still runs it on the candidate.
