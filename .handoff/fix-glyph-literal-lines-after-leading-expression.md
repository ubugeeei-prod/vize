# Handoff: fix/glyph-literal-lines-after-leading-expression

Delete `.handoff/` before merge. This branch is stacked on `refactor/glyph-l1-format-profile` (PR #13407). Rebase it onto main once #13407 merges.

## What is wrong (latent bug, found while doing #6847 F3)

`vize_l1::embed::syntax::literal_lines::literal_line_starts` (moved from Glyph's private `template/literal_spans.rs` without behaviour change in #13407) reads a value with OXC `Parser::parse_expression`. That call parses the *leading* expression and silently ignores the rest. So for a handler value such as `track();\n  notify(\`one\n  two\`)`, only `track()` is read. The template literal in the second statement is not seen, its continuation line counts as code and gets re-indented, and that changes the runtime string. The repro is `.handoff/literal-lines-repro.vue`, a same-line `@click` value.

## Done (uncommitted work committed here as WIP; NOT built or tested yet: the disk was below 60 GB)

- `literal_lines.rs`: the leading expression is admitted only when its span reaches the end of the trimmed value. Otherwise the value is parsed as a Program: any clean Program for directive values, exactly one expression statement for interpolations (comments allowed).
- `literal_lines/tests.rs`: new `literals_after_a_leading_expression_are_owned`.

## Next steps

1. Build and run `cargo test -p vize_l1 --lib embed::syntax` and `cargo test -p vize_glyph`.
2. Measure the behaviour change.
   - Rerun the old-vs-new literal differential over the repo corpus. It is a temporary unit test comparing the #13407 provider with this one; the method is in #13407's body.
   - Run `vize fmt` parity, base vs head, over the corpus. Every changed output must be a literal-ownership fix; list the files.
3. Confirm the repro is mangled on main and preserved on this branch.
4. Pin the fix, following the pattern of `fcbff3741c` (#13276):
   - a `formatter/sfc/...` case in `tests/_fixtures/differential/formatter/manifest.json`, with `App.vue.txt` and `reference.expected.txt`;
   - a Rust witness test in `crates/vize_glyph/tests/` (full output bytes plus 3-pass idempotence);
   - a ledger row if the corpus changes.
5. Measure: release-profile `shipped-profile-measure.yml` (every row <= base), ci-opt `level-instruction-counts.yml` (informational), release size under the fleet lock, and the PR performance observation.
6. Open the PR as `fix(glyph): own literals after a value's leading expression (#6847)`. Its `Co-Authored-By` lines are the Claude line only; there is no external reporter.

No CI runs are in flight for this branch.
