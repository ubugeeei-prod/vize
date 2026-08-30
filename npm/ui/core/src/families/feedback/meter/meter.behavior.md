# Meter behavior contract

Normative state x input -> outcome table for `meter.vue` (`@vizejs/ui/meter`).
Every row is proven by the named mounted-DOM, SSR, or compile-time test.

| Surface    | Contract                                                                                                              |
| ---------- | --------------------------------------------------------------------------------------------------------------------- |
| Element    | `Meter` renders a native `<meter>` with deterministic id, optional ARIA labelling, and no custom focus behavior.      |
| Range      | `value`, `min`, and `max` are normalized to a finite native-safe range; `max <= min` is repaired to `min + 1`.        |
| Thresholds | Optional `low`, `high`, and `optimum` thresholds are clamped into range; reversed low/high input is sorted.           |
| State      | Slot/expose/data state publishes value, bounds, thresholds, percent, range, optimum match, invalid repair, and token. |
| Styling    | No component CSS is emitted beyond a scoped empty block; `data-vize-ui`, `part`, and `data-*` are the contract.       |
| SSR        | Server output is deterministic and contains only native meter attributes, data attributes, and slot text.             |
| Packaging  | Root and subpath consumers retain only Meter plus shared deterministic-id support with zero CSS.                      |

| State x input                  | Outcome                                                                                                   |
| ------------------------------ | --------------------------------------------------------------------------------------------------------- |
| finite value inside thresholds | Native `<meter>` receives the same numeric attributes and publishes `data-range` plus normalized percent. |
| optimum threshold in range     | `data-optimal="true"` and `state="optimum"` when the current value is in the same threshold range.        |
| value below min or above max   | Value is clamped to the native range and `invalid` is reported for styling and diagnostics.               |
| non-finite or reversed inputs  | Unsafe inputs are repaired before reaching the native element and `data-invalid="true"` is published.     |
| omitted thresholds             | Threshold attributes are omitted while range state falls back to `medium`, except `empty` and `full`.     |
