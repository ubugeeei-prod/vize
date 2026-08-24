# P2-9 Installment 9 — interpolation-filter text projection (2026-08-24)

> Part of the [P2-9 series record](../p2-9.md), split per installment
> under the 350-line source budget.

Installment 8 named the gap and left it out of the Vue 2 battery.
This installment measures it and closes it. The series box in
`phase-2.md` stays open (P1-9 residual still unmeasured).

## What was measured

Command: `cargo test -p vize_atelier_core --features legacy --test davinci_s2_transform_vue2`
(probe templates run through `compare_with(..., VueVersion::V2)` before
any comparator change). The text projection prints both unit lists on
TS-25 panic.

| template | legacy dynamic text | S2 dynamic text | result |
| --- | --- | --- | --- |
| `{{ msg \| cap }}` | `msg \| cap` | `_filter_cap(msg)` | diverge |
| `<div>{{ msg \| cap }}</div>` | `msg \| cap` | `_filter_cap(msg)` | diverge |
| `hello {{ msg \| cap }}` | `msg \| cap` | `msg \| cap` | **agree** (compound) |
| `{{ a \| f(b) }}` | `a \| f(b)` | `_filter_f(a,b)` | diverge |
| `{{ a \| f \| g }}` | `a \| f \| g` | `_filter_g(_filter_f(a))` | diverge |
| `{{ a \|\| b }}` | `a \|\| b` | `a \|\| b` | agree (not a filter) |
| `{{  msg \| cap  }}` | `msg \| cap` | `_filter_cap(msg)` | diverge (trim) |

The mixed run is the installment-7 compound-opaque family: the pipe is
not `ExprRef::Filter`, so `legacy-sugar` does not wrap it. Both
collectors publish the authored `msg | cap`. Not a silent skip — it
compares exactly and is now in the battery.

## Why agreement, not a skip class

House: agree if one side is reading the wrong stage; count if the two
trees genuinely represent different things.

The two *spellings* differ, but they name one rewrite. S2
`legacy-sugar` always wraps `ExprRef::Filter`. The comparator's legacy
transform uses default `prefix_identifiers: false`, so
`transform_interpolation` never calls `process_expression` and never
runs the filter rewrite that lives there. The legacy collector is
reading the pre-legalize interpolation; S2 publishes the post-legalize
one. Turning `prefix_identifiers` on would also prefix `_ctx.` on the
legacy side (`transform_expression` is a P2-9 non-goal) and would
diverge again.

So the check wrap-equals: `VueFilterExpr::parse_in` (the same
admission S2 lowering uses) plus the `legacy-sugar` wrap
(`a | f` → `_filter_f(a)`, `a | f(b)` → `_filter_f(a,b)`, `-` → `_`).
When wrap(authored) equals the S2 text, the parts agree.

A skip class (`filter_templates` like `entity_templates`) would have
been the honest move if the wrap were not reconstructible. It is.
Skipping would also have dropped the mixed run, which already agrees.

## Counted class (the ratchet, not a skip)

`parts_filter` counts wrap-normalized agreements only. Exact-equal
dynamic parts still increment `parts_dynamic` alone. Both wrap-equals
and exact-equal increment `parts_dynamic` (they agreed). The extra
counter is the pin that S2 still legalizes: if both sides started
reporting authored `msg | cap`, `parts_filter` would fall to 0 and
the witness would fail. TS-13: exact-equality on the pinned
`Counters`; no assertion-allowlist entry.

## Battery (exact-pinned)

Nine templates, zero divergence. New cases:

| name | what it pins |
| --- | --- |
| `filter` | lone `{{ msg \| cap }}` wrap-equals `_filter_cap(msg)` |
| `filter-mixed` | mixed run, authored pipe on both sides, `compound_units += 1` |
| `filter-args` | `{{ a \| f(b) }}` wrap-equals `_filter_f(a,b)` |

Text counters after the three: `units=6`, `parts_static=3`,
`parts_dynamic=4`, `compound_units=1`, `parts_filter=2`. Surface and
hoist counters unchanged (the new templates have no element owners).
Vue 3 battery / corpus witness gain `parts_filter: 0` only.

## What remains

P1-9 residual re-measure is still not this installment. Filter wrapping
happens on S2 `ExprRef`s after lowering and does not feed
`transform_expression/reparse.rs`. Inventing a 12.73% figure without
running the counters would violate the task. Mixed-run pipes stay
unwrapped on S2 (installment 7); they now compare, they are not a
new wrap. Bind-value filters (`:id="raw | formatId"`) are a surface
projection, not this text-projection installment.

## House

Every touched file ≤ 350. `s2_support` stays a test module. ricalco
`src/emit/**` untouched. Assertions are exact-equality on the pinned
`Counters` (TS-13).
