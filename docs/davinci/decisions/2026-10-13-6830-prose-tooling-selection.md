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

The `typechecker-native-release-inputs` ledger is retired with its guard and
the three path exclusions it guarded in `typechecker-native-phases.yml`. It
pinned the bytes of the planner and its configuration, so every planner change
had to re-pin it.

## Merge queue

A merge-queue candidate whose own change is only prose (Markdown under `docs/`
or `.changeset/`, or a Markdown file at the repository root) now gets the plan
the same PR gets: no Rust, JS package or playground lanes, and the prose
selection above for tooling tests. The candidate is built on the candidate
ahead of it, which ran every lane on the code, so the candidate's tree differs
from that verified tree by prose alone. Markdown inside crates and packages
can be compiled in as Rust or package docs and keeps every lane.

## Reason

A change to `AGENTS.md` or one guide page selected 1,048 to 1,087 of 1,247
tooling tests, which is the full tooling tier: Markdown outside `docs/` was an
unknown input, and every test without a declared scope used the whole tooling
input set, which includes `docs/**`. With this change the same edits select 40
and 130 tests.

## Tradeoff

A test that reads prose through a computed path it never spells out is no
longer selected on the PR; the merge queue still runs it on the candidate.
