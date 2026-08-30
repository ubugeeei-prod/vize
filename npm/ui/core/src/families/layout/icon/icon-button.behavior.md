# IconButton

| Case                     | Required behavior                                                                                                                                                    |
| ------------------------ | -------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `icon-button.vue` source | Owns the canonical IconButton SFC implementation and every row in this table.                                                                                        |
| Accessible name          | Public `IconButtonProps` require `ariaLabel` or `ariaLabelledby`; rendered markup forwards the resolved name and marks `data-name`.                                  |
| Native button            | Defaults to a native `button` with `type="button"`, `data-vize-ui="icon-button"`, `part="root"`, no class/style output, and exact once-per-activation `press` emits. |
| Non-native button        | Non-native hosts render `role="button"`, enter the tab order, and emulate native Enter/Space activation timing.                                                      |
| Unavailable states       | `disabled` removes activation and sequential focus; `loading` announces `aria-busy`, mirrors `aria-disabled`, preserves focus, and suppresses `press`.               |
| Styling hooks            | `size`, `tone`, and `variant` are strict tokens exposed through slot state, `defineExpose`, and `data-*`; no CSS is emitted.                                         |
| SSR and hydration        | Server markup is stable and hydrates without diagnostics or root replacement.                                                                                        |
