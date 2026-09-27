# Inherited workflow security findings

Tracked in [#6866](https://github.com/ubugeeei-prod/vize/issues/6866) and
[#6830](https://github.com/ubugeeei-prod/vize/issues/6830). Existing external
Canon PRs #6921 and #6922 exposed these findings without changing workflows.
This correction belongs to the existing CI stack, separately from product fixes.

## Shell arguments

`check-bench.yml` and `tool-benchmark.yml` expand `github.ref_name` directly in
three shell commands. Step environment variable `VIZE_BENCH_REF` carries that
value into the two quoted `--ref` arguments and the quoted push destination
`HEAD:refs/heads/$VIZE_BENCH_REF`. The publishing job already requires a branch
reference. The shell no longer parses expression contents as source code.

## Rust action identity

Full Zizmor runs 36310581727 and 36310840105 reported 44 `impostor-commit`
findings: 42 references to `29eef336d9b2848a0b548edc03f92a220660cdb8` and two
to `4360b52568e2003a75bf9bc1d59f33a8e3fc893c`. A commit API lookup can return
an object from GitHub's fork network and does not prove official reachability.
Both old commits diverge from the present official branch by one old commit.
The audit's request warnings concern `useblacksmith/stickydisk`, separately.

All 44 references use official stable branch commit
`6bed0761d98439e5a578e2877258200ad565ba87`, verified with GitHub's branch-ref
API. Existing explicit toolchain, component and target inputs stay unchanged.
The two no-input calls in title policy and release retain the action's stable
default. The new `action.yml` is byte-identical to the old `4360b525` version;
against `29eef336`, differences only name steps and remove shell no-op labels.
The current master action removes the stable default, so it is not substituted.

## Validation and limits

The correction uses mechanical replacement with asserted old-pin counts and
three exact step contexts. Focused workflow contract tests and shell argument
checks run locally. The final existing CI-stack head must execute the full
online Zizmor audit in Actions; no audit exemption or severity reduction is
introduced. Until that run succeeds, the inherited security gate remains open.

Future upstream branch movement can make a generated stable pin unreachable
again; an authentic pin must then be checked and refreshed through the same
workflow-security gate. This change does not claim a permanent upstream identity.
