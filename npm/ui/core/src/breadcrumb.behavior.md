# Breadcrumb Behavior Contract

`Breadcrumb` provides a headless landmark and ordered-list shell for route
hierarchies. It owns the accessibility invariants while leaving link rendering,
router integration, separators, truncation, and visual styling to consumers.

| Contract            | Observable behavior                                                                                                                                                                          |
| ------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Landmark            | `breadcrumb.vue` renders a `nav` by default, applies `data-vize-ui="breadcrumb"` and `part="root"`, and mirrors `label` to `aria-label`.                                                     |
| List semantics      | `breadcrumb-list.vue` renders an `ol` by default, applies `data-vize-ui="breadcrumb-list"` and `part="list"`, and does not add roles or styles.                                              |
| Item state          | `breadcrumb-item.vue` renders an `li` by default, applies `data-vize-ui="breadcrumb-item"` and `part="item"`, and mirrors current state to `data-current="true"` only when current.          |
| Link state          | `breadcrumb-link.vue` renders an `a` by default, forwards `href`, applies `data-vize-ui="breadcrumb-link"` and `part="link"`, and maps `current=true` to `aria-current="page"`.              |
| Link safety         | `BreadcrumbLink` trims safe `href` values and suppresses empty, control-character, `data:`, `javascript:`, and `vbscript:` destinations.                                                     |
| Route-aware current | `BreadcrumbLink current` accepts the literal `aria-current` route states `page`, `step`, `location`, `date`, and `time`, preserving strict type feedback for router-derived state.           |
| Separator semantics | `breadcrumb-separator.vue` renders a `span` by default, applies `data-vize-ui="breadcrumb-separator"` and `part="separator"`, and is always `aria-hidden="true"` with `role="presentation"`. |
| Slots               | Root, item, link, and separator slots receive typed state objects for label, current state, resolved `aria-current`, and decorative separator state.                                         |
| Expose              | Every part exposes its rendered element; root exposes `label`, item exposes `current`, link exposes `current`, `ariaCurrent`, and `focus`, and separator exposes `decorative`.               |
| Styling             | The primitive emits no class names, inline styles, CSS custom properties, or package CSS; spacing, glyphs, collapse, ellipsis, color, and focus rings are consumer-owned.                    |
| SSR                 | Repeated isolated SSR requests emit byte-identical markup, and hydration preserves the server root without warnings or node replacement.                                                     |
