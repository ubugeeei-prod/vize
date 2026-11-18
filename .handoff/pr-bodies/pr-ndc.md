## Summary

`html/no-duplicate-class` found the same duplicates as `vue/no-duplicate-class-names`, but reported a different range and split names into separate findings. Each finding therefore became a false positive plus a false negative in the Real Project Matrix (#12336; run 44093557816 had 55 FP and 46 FN across npmx, dbx, airi, vue-bits, primevue and others).

In a Vue file the rule now reports on the node eslint-plugin-vue reports, with one finding that lists every repeated name:

| Duplicate | Before | Now |
| --- | --- | --- |
| repeated names in a static `class` value | the whole attribute, one finding per name | the value `"..."`, one finding |
| repeated names in one `:class` string, object key or template quasi | that part, one finding per name | that node (a quasi spans its `` ` `` / `${` / `}` delimiters, as ESTree does) |
| a `:class` name that the static `class` also lists | the part | the element's start tag |
| a name two `:class` parts can both render | the later part | the node holding the first part: the binding value, array, conditional, `+` expression, property or template literal |

Which names count as duplicates is unchanged: possible class parts still come from `possible_static_parts`. For example, a fragment such as `badge--` in `` `badge--${variant}` `` is still not a class.

JSX and standalone HTML keep their current output: one finding per name on the whole attribute.

- New message key `html/no-duplicate-class.message_many` (en, ja, zh) for findings that list several names.
- Allocations: a value with fewer than two names allocates nothing, and the static class list is no longer collected into a set, so the `davinci_authored_markup_budget` zero-allocation budget holds.

## Tests

- `no_duplicate_class::tests`: exact `(start, end, message)` oracles for static values, start-tag intersections, string, key and quasi nodes, cross-part holders, and the non-colliding controls. All ten original literal fixtures are kept.
- `default_correctness_tests`: the exact-packet helper expects the value range and one listing finding in Vue files, and per-name attribute findings for JSX and HTML.
- `markup::tests::no_duplicate_class_tests`: two template counts become 1, one finding per value.
- Battery census: +1 literal fixture, +1 call, +20 trace lines, with accounting comments.
- Differential case `linter/no-duplicate-class-nodes-12336`.

## Local measurement

`lint-divergence-report --all-mapped-rules`, `html/no-duplicate-class`, FP/FN:

| Project | Run 44093557816 | This PR |
| --- | --- | --- |
| npmx | 21 | 1 FP |
| dbx | 13 | 0 |
| airi | 8 | 0 |
| vue-bits | 8 | 0 |
| primevue | 8 | 0 |
| hoppscotch | 7 | 1 FP / 2 FN |
| vue-vben-admin | 5 | 1 FP |
| habitica, frappe-builder, elk | 4, 4, 2 | 0 |
| element | 1 FN | 1 FN |

Both kinds of residual are semantic differences, not locations:
- Patina counts identifier object keys (`:class="{ hidden: cond }"` with a static `hidden`); upstream counts only string-literal keys.
- Upstream counts template fragments such as `-600` in `` `hover:text-${color}-600` ``; Patina does not.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
