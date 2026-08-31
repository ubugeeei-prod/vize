# Icon

| Case                 | Required behavior                                                                                                                                              |
| -------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `icon.vue` source    | Owns the canonical Icon SFC implementation and every row in this table.                                                                                        |
| Default icon         | Renders a native `svg` root with `data-vize-ui="icon"`, `part="root"`, no class/style output, `focusable="false"`, and decorative semantics.                   |
| Decorative semantics | Icons without an accessible name, or icons with `decorative`/`ariaHidden`, set `aria-hidden="true"` and omit `role`, labels, `title`, and `desc`.              |
| Accessible image     | Icons named by `ariaLabel`, `ariaLabelledby`, or `title` render `role="img"` and expose deterministic `title`/`desc` ids when inline text is used.             |
| SVG composition      | Consumers own paths through the default slot; the slot receives current `ariaState`, `decorative`, `size`, `titleId`, `descriptionId`, and `viewBox`.          |
| Styling hooks        | `size`, `fill`, `stroke`, `strokeWidth`, `strokeLinecap`, and `strokeLinejoin` are typed and mirrored to native SVG attributes or `data-*`; no CSS is emitted. |
| SSR and hydration    | Generated title and description ids are deterministic across isolated SSR requests and hydrate without replacing the root.                                     |
