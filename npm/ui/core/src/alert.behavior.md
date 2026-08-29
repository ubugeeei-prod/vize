# Alert behavior contract

Normative state x input -> outcome table for `alert.vue` (`@vizejs/ui/alert`).
Every row is proven by the named mounted-DOM test in `src/alert.test.ts`,
runtime conformance in `src/runtime-conformance.test.ts`, or compile-only
assertions in `src/alert.types.test-d.ts`; a row without a passing test is a
contract violation.

| #   | State         | Input                | Outcome                                                                                                                | Proven by                                                                               |
| --- | ------------- | -------------------- | ---------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------- |
| A1  | open, alert   | render               | native `<div role="alert">`, assertive live region, atomic announcements, accessible labelling, and `data-state`       | `renders an assertive alert with labelling, description, variant, and open state`       |
| A2  | open, status  | render               | `role="status"` switches to polite live-region semantics and preserves consumer-owned atomic policy                    | `switches to a polite status region without forcing atomic announcements`               |
| A3  | closed        | render / Tab         | the root remains mounted for hydration stability, carries `hidden`, mirrors `data-state="closed"`, and is not tabbable | `closed alerts stay mounted but are hidden from user agents`                            |
| A4  | any           | slot / exposed state | slot consumers receive the live role, variant, visibility state, and boolean; exposed `element` is the rendered root   | `exposes slot state for application-owned chrome and dismissal`                         |
| A5  | SSR/hydration | isolated requests    | server markup is byte-identical per request and hydrates without replacing the root                                    | `renders stable, accessible markup across isolated SSR requests`; hydration conformance |
| A6  | public types  | invalid role/variant | compilation rejects unsupported live-region roles and variants                                                         | `src/alert.types.test-d.ts`                                                             |

The primitive intentionally ships no built-in dismiss button or CSS. Dismissal
chrome is application-owned in this slice and can consume slot state while the
subpath remains tree-shakable with zero packaged CSS.
