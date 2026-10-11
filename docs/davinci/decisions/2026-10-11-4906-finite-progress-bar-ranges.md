# Finite ProgressBar percentages and CSS

Issue: [#4906](https://github.com/ubugeeei-prod/vize/issues/4906).

## Problem and reproduction

The public `getProgressBarState` normalizer divides `(value - min)` by
`(max - min)`. A finite signed range from `-Number.MAX_VALUE` to
`Number.MAX_VALUE` overflows that span: its midpoint incorrectly reports 0%
and its maximum produces `NaN`. `getProgressBarStyle` then publishes `NaN%`
and `NaN` CSS custom properties. An invalid upper bound repaired to
`min + 100` can also round back to a large finite minimum and divide by zero.

## Decision

Keep ordinary arithmetic, the authored finite value scale, and existing
completion and indeterminate policies. Half-scale both differences only when
the span overflows. When the default repair cannot advance the minimum, mark
the range invalid and report zero ratio/percent instead of dividing by zero.
The existing completion predicate still means that a determinate value has
reached the normalized maximum; it is unchanged in this invalid case.

[WAI-ARIA 1.2](https://www.w3.org/TR/wai-aria-1.2/#aria-valuemax) permits a maximum
equal to the minimum. Replacing authored bounds with an unrelated default scale
would lose the consumer's ARIA value semantics. Preserve those finite bounds
and values, and keep unknown values indeterminate with `aria-valuenow` omitted.
Public props, slot types, and the behavior contract document the precision
exception.

## Regression and integration gates

- `progress-bar-state.test.ts` proves the ordinary default/repair contracts,
  signed range endpoints and midpoint, collapsed bounds in both signs, an
  advancing negative large-number repair, and finite bounded state/CSS laws.
- `progress-bar.test.ts` updates props across wide and collapsed ranges and
  verifies ARIA, data, CSS, slot, and exposed state, then returns to unknown
  progress.
- `progress-bar-ssr.test.ts` renders isolated requests across the same numeric
  limits and checks deterministic ARIA and CSS.
- Four additive runtime cases check native-generated SSR, Vapor mount, and
  hydration. The existing ordinary runtime fixture and all original component
  assertions remain intact. Budgets and the known-failure ledger are unchanged.

Pure source controls reproduce three failing regression tests before the
normalizer fix and four passing tests afterward. They provide no mounted,
native compiler, SSR, browser, CI, or release execution credit.
An additional source control directly compares the original normalizer at
`588d2c29ceead01ca391ab10eaf59439c8619798` with this implementation: all 3,300
ordinary input combinations and five non-overflowing large-number boundary
cases retain identical complete state objects and CSS tokens. The original
normalizer is the oracle; it is not reimplemented by the control.

Source preparation uses the exact source head of
[PR #8553](https://github.com/ubugeeei-prod/vize/pull/8553), including its support
for multiple distinct runtime cases per SFC. The normalizers remain independent.
Before delivery, use the actual merged parent on fresh `main`, or register a
native Stack if publishing while that parent is still unmerged; parent links
alone are insufficient.

## Remaining work

Run the complete selected UI Actions suite on the published source, including
both Vue provider configurations and the retained native conformance ratchet.
Then follow the protected queue through actual merge and verify the official
public package's root and `progress-bar` subpath consumers on a release source
containing that merge. These are unfinished until their actual evidence exists.
This numeric slice does not complete the broader feedback/status family in
#4906.

## Release preparation control

The protected [#8555 candidate](https://github.com/ubugeeei-prod/vize/actions/runs/38134653350)
exposed a [#6830](https://github.com/ubugeeei-prod/vize/issues/6830) test-input collision:
replacing the first `8250` changed a real cut SHA before the integration marker.
Target the complete marker instead. Retain every original receipt, author,
parent, pin, deadline and exact error assertion; repeat the same negative
controls with harmless earlier cut-shaped text containing `8250`, retaining
the authentic Git fixture and whole non-marker bytes. Production,
runtime fixtures, budgets and the shared decision record remain byte-exact.
