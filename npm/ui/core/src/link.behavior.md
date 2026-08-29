# Link behavior contract

Normative state × input → outcome table for `link-anchor.vue` (`@vizejs/ui/link`).
Every row is proven by the named mounted-DOM test in `src/link.test.ts`; a row
without a passing test is a contract violation.

| #   | State        | Input                 | Outcome                                                                                          | Proven by                                                              |
| --- | ------------ | --------------------- | ------------------------------------------------------------------------------------------------ | ---------------------------------------------------------------------- |
| L1  | idle, native | render                | native `<a href>`, deterministic `id`, accessible name, forwarded link attributes, current state | `renders a native link with navigation and accessibility attributes`   |
| L2  | idle         | Tab / `focus()`       | link receives focus through native tab order and the exposed focus method                        | `joins the tab order and focuses programmatically`                     |
| L3  | idle         | pointer click         | exactly one `navigate` emit carrying the `MouseEvent`; native click listeners still run          | `click emits navigate and preserves consumer click listeners`          |
| L4  | idle         | Enter / Space         | Enter follows native link activation; Space does not synthesize link activation                  | `Enter activates native links while Space remains non-activating`      |
| L5  | disabled     | render                | no `href`/navigation attributes, `aria-disabled="true"`, `tabindex="-1"`, `data-state=disabled`  | `disabled links remove navigation, tab focus, and activation`          |
| L6  | disabled     | click / Enter / Space | no `navigate` and no consumer click listener                                                     | `disabled links remove navigation, tab focus, and activation`          |
| L7  | inert        | render                | native `inert`, no `href`, `aria-disabled="true"`, `tabindex="-1"`, `data-state=inert`           | `inert links expose native inertness and suppress fallback activation` |
| L8  | inert        | click / Enter / Space | no `navigate`                                                                                    | `inert links expose native inertness and suppress fallback activation` |
| L9  | any          | default slot render   | slot receives live `disabled`, `inert`, and `unavailable` state                                  | `exposes disabled, inert, and unavailable to the default slot`         |
| L10 | non-current  | render                | `aria-current` is omitted for `false`                                                            | `renders a native link with navigation and accessibility attributes`   |

Disabled and inert states deliberately omit `href` so the anchor cannot be
activated through native navigation in runtimes that do not enforce inertness.
Styling remains entirely consumer-owned; the component only publishes semantic
attributes and state hooks.
