# Button behavior contract

Normative state × input → outcome table for `action-button.vue` (`@vizejs/ui/button`).
Every row is proven by the named mounted-DOM test in `src/button.test.ts`; a row
without a passing test is a contract violation.

| #   | State                | Input                       | Outcome                                                                         | Proven by                                                                     |
| --- | -------------------- | --------------------------- | ------------------------------------------------------------------------------- | ----------------------------------------------------------------------------- |
| B1  | idle, native         | render                      | `role=button`, accessible name from slot, `type="button"`, `data-state="idle"`  | `renders a native button with an accessible name`                             |
| B2  | idle                 | Tab                         | control receives focus                                                          | `joins the tab order and focuses programmatically`                            |
| B3  | idle                 | exposed `focus()`           | control receives focus                                                          | `joins the tab order and focuses programmatically`                            |
| B4  | idle                 | pointer click               | exactly one `press` carrying the `MouseEvent`                                   | `pointer click fires exactly one press`                                       |
| B5  | idle, native         | Enter                       | exactly one `press` (activation on keydown)                                     | `Enter and Space each fire exactly one press on a native button`              |
| B6  | idle, native         | Space                       | exactly one `press` (activation on keyup)                                       | `Enter and Space each fire exactly one press on a native button`              |
| B7  | idle, non-native     | Enter                       | the component itself clicks the element; exactly one `press`                    | `Enter and Space activate a non-native button through its own handlers`       |
| B8  | idle, non-native     | Space                       | keydown canceled (no page scroll); one `press` on keyup                         | `Enter and Space activate a non-native button through its own handlers`       |
| B9  | idle, non-native     | Escape                      | ignored: not canceled, no `press`                                               | `Enter and Space activate a non-native button through its own handlers`       |
| B10 | idle                 | click, Enter, Space         | three `press` emits in dispatch order, each indistinguishably a `MouseEvent`    | `keyboard and pointer presses are indistinguishable MouseEvents in order`     |
| B11 | disabled, native     | render                      | native `disabled` attribute, no `aria-disabled` mirror, `data-state="disabled"` | `disabled native button removes activation and keeps native semantics`        |
| B12 | disabled, native     | click / Enter / Space       | no `press`                                                                      | `disabled native button removes activation and keeps native semantics`        |
| B13 | disabled, native     | Tab                         | skipped by the tab order                                                        | `disabled native button removes activation and keeps native semantics`        |
| B14 | disabled, non-native | render                      | `tabindex="-1"`, `aria-disabled="true"`                                         | `disabled non-native button leaves the tab order and announces aria-disabled` |
| B15 | disabled, non-native | click / Enter / Space / Tab | no `press`; skipped by the tab order                                            | `disabled non-native button leaves the tab order and announces aria-disabled` |
| B16 | loading              | render                      | `aria-busy="true"`, `aria-disabled="true"`, `data-state="loading"`, focusable   | `loading button announces busy, stays focusable, and suppresses press`        |
| B17 | loading              | `focus()`                   | focus is accepted and retained                                                  | `loading button announces busy, stays focusable, and suppresses press`        |
| B18 | loading              | click / Enter / Space       | no `press`                                                                      | `loading button announces busy, stays focusable, and suppresses press`        |
| B19 | any                  | default slot render         | slot receives live `disabled`, `loading`, `unavailable` state                   | `exposes disabled, loading, and unavailable to the default slot`              |

Keyboard activation timing (`Enter` on keydown, `Space` on keyup, `Space` keydown
canceled) is additionally pinned as pure logic by `matches native keyboard
activation timing`.
