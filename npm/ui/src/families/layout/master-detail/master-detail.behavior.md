# Master-detail behavior contract

Normative state x input -> outcome table for `master-detail.vue`
(`@vizejs/ui/master-detail`). Rows are proven by `master-detail.test.ts` and
`master-detail-ssr.test.ts`; compile-only assertions live in
`master-detail.types.test-d.ts`.

| #   | State               | Input                    | Outcome                                                                                     | Proven by                                                                  |
| --- | ------------------- | ------------------------ | ------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------- |
| MD1 | wide viewport       | render, select           | split grid shows master and detail (or `empty`); selecting emits `update:selected`          | `split layout shows both panes and an empty placeholder`                   |
| MD2 | narrow viewport     | select, Escape, Back     | one pane at a time; focus moves to the detail on select and back to the list on Escape/Back | `stacked layout swaps panes, moves focus, and returns with Back or Escape` |
| MD3 | controlled / resize | click, set prop, resize  | controlled selection follows the prop; `layoutChange` reports split/stacked switches        | `controlled selection and layout changes are reported`                     |
| MD4 | SSR with `ssrWidth` | isolated requests        | byte-identical split or stacked markup; stacked is used without `ssrWidth`                  | `renders byte-identical split or stacked markup from ssrWidth`             |
| MD5 | hydration           | mount over server markup | no mismatch warnings                                                                        | `hydrates the ssrWidth layout without mismatch warnings`                   |
| MD6 | DOM/SSR/Vapor       | compile                  | the SFC compiles in every renderer lane                                                     | `scripts/check-renderers.ts`                                               |

Both panes are labelled `section` regions with `tabindex="-1"` so focus can be
moved to them programmatically without adding tab stops.
