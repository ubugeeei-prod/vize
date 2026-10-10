# Reachable Rust toolchain action pins (2026-10-10)

Issue: [#6830](https://github.com/ubugeeei-prod/vize/issues/6830).
Observed failure: the unmodified inherited workflow-security surface in
[#8506](https://github.com/ubugeeei-prod/vize/pull/8506) reports 88 high-confidence,
high-severity `impostor-commit` findings, including two new benchmark calls.
The current main source has 86 inherited calls using the same two generated pins.

## Decision

Pin each action to its corresponding immutable parent in the official
`dtolnay/rust-toolchain` master history, and explicitly pass the intended Rust
toolchain where the generated action previously supplied a default.

| Generated stable pin                       | Reachable official parent                  | Main calls |
| ------------------------------------------ | ------------------------------------------ | ---------: |
| `6bed0761d98439e5a578e2877258200ad565ba87` | `d1031067263f94b142dd6c0ce24c5eb9d02d52a0` |         76 |
| `89b12181fb390509a0842a86cc55eeb8eb928c1d` | `7e38f4b43b4db5c8dd498af069a4f6196df1d067` |         10 |

The official [first generated commit](https://github.com/dtolnay/rust-toolchain/commit/6bed0761d98439e5a578e2877258200ad565ba87)
and [second generated commit](https://github.com/dtolnay/rust-toolchain/commit/89b12181fb390509a0842a86cc55eeb8eb928c1d)
change only `inputs.toolchain.required` from `true` to `false` and add
`inputs.toolchain.default: stable` in `action.yml`. Full-file comparisons confirm
that every executable step, script, dependency and other input remains identical
to the corresponding parent. Both parents are ancestors of the official master
head audited at `e2a55d2ffb04f378e9626c28d38b36d230d1e12f`; the generated commits
are outside that history. The provider's [SHA pinning guidance](https://github.com/dtolnay/rust-toolchain#choice-of-full-length-commit-sha)
requires a commit from master history because generated revision commits can be
garbage-collected.

This correction retains the implementation of each installed action. The newer
master action has executable differences from both parents and is not part of
this change.

## Preserved configuration

At main source `7fbe0a714b2fc0c6a4602d84a5690812b841e1f5`, the 86 affected calls
span 41 workflows and five composite actions. Eighty calls omit `toolchain` and
receive explicit `stable`. The six existing explicit inputs remain unchanged:
`1.95.0` once, `1.99.0` twice, and `nightly` three times. All 12 component inputs,
all ten target inputs, job conditions, matrices, permissions, step order, action
outputs and executable commands retain their previous configuration. The Check
workflow remains within its unchanged 690-line budget.

The two benchmark calls introduced by #8506 remain that PR's responsibility.
This focused main-source correction does not rewrite that contribution's 86
inherited workflow surfaces. Rebasing the contribution after the correction
must preserve its intended benchmark toolchains explicitly.

## Validation and acceptance

Source preparation verifies complete parsed workflow/action equality after
normalizing only the corresponding pin replacements and newly explicit stable
inputs. This is checked for every affected call and every complete affected
workflow or composite action, with all other configuration preserved.

Existing workflow lint, full Check, normal PR checks, and the complete workflow
security audit must pass on the exact candidate. The trusted same-repository
Zizmor job uploads SARIF and can finish successfully while findings remain;
its success label alone is not security acceptance. Inspect the complete findings
or run the normal console audit at the configured confidence and severity.
Retain previous failed audit evidence. No finding ignore, waiver, provider
upgrade, rerun reclassification, or protected-branch bypass is authorized.
