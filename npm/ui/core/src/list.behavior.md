# List behavior contract

Normative state x input -> outcome table for `list.vue` (`@vizejs/ui/list`).
Every row is proven by the named mounted-DOM or SSR test. A row without a
passing test is a contract violation.

| #   | State              | Input             | Outcome                                                                                           | Proven by                                                                        |
| --- | ------------------ | ----------------- | ------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------- |
| L1  | default            | render / Tab      | renders native `<ul data-vize-ui="list">`, `part="root"`, default hooks, list items, and no focus | `renders a native unordered list by default without adding semantics or styling` |
| L2  | list hooks         | render            | renders the requested host while mirroring strict marker, spacing, and tone hooks                 | `mirrors list presentation hooks on an ordered host`                             |
| L3  | consumer semantics | fallthrough attrs | preserves consumer-owned role, label, and focus attributes without deriving custom semantics      | `keeps custom semantics and focus policy consumer owned through attrs`           |
| L4  | any                | slot/expose       | passes marker, spacing, and tone to the slot and exposes the rendered element with live props     | `passes slot state and exposes live list state`                                  |
| L5  | SSR default        | isolated requests | renders byte-identical native list markup without request-global state                            | `renders byte-identical native list markup across isolated SSR requests`         |
| L6  | SSR custom         | render            | renders custom host markup with consumer-owned semantics and no default `aria-hidden` or style    | `renders consumer-owned server semantics without implicit accessibility attrs`   |
| L7  | DOM/SSR/Vapor      | compile           | authored SFC compiles in every renderer lane without warnings or fallback                         | `scripts/check-renderers.ts`                                                     |
| L8  | SSR/hydration      | hydrate           | server list markup hydrates without warnings, node replacement, or accessibility drift            | `src/runtime-conformance.test.ts`                                                |
| L9  | root/subpath       | consumer bundle   | root and subpath consumers retain only List, emit no CSS, and stay within gzip budget             | `scripts/check-tree-shaking.mjs`                                                 |

## Props

| Prop      | Type                                                                     | Purpose                                                        | Default     |
| --------- | ------------------------------------------------------------------------ | -------------------------------------------------------------- | ----------- |
| `as`      | `PrimitiveAs`                                                            | Native element, custom element, or component rendered as host. | `"ul"`      |
| `marker`  | `"disc" \| "decimal" \| "none"`                                          | Consumer marker token mirrored to `data-marker`.               | `"disc"`    |
| `spacing` | `"compact" \| "normal" \| "loose"`                                       | Consumer spacing token mirrored to `data-spacing`.             | `"normal"`  |
| `tone`    | `"accent" \| "danger" \| "muted" \| "neutral" \| "success" \| "warning"` | Consumer color or semantic tone token mirrored to `data-tone`. | `"neutral"` |

## Slots

| Slot      | Props                                                          | Purpose                                    | Default |
| --------- | -------------------------------------------------------------- | ------------------------------------------ | ------- |
| `default` | `{ marker: ListMarker; spacing: ListSpacing; tone: ListTone }` | Render list items with presentation hooks. | none    |

## Expose

| Name      | Type                  | Purpose                                      | Default     |
| --------- | --------------------- | -------------------------------------------- | ----------- |
| `element` | `ListElement \| null` | Rendered host element or component instance. | `null`      |
| `marker`  | `ListMarker`          | Current marker token.                        | `"disc"`    |
| `spacing` | `ListSpacing`         | Current spacing token.                       | `"normal"`  |
| `tone`    | `ListTone`            | Current tone token.                          | `"neutral"` |

## Data Attributes

| Attribute      | Values                                                                   | Purpose                 | Default     |
| -------------- | ------------------------------------------------------------------------ | ----------------------- | ----------- |
| `data-vize-ui` | `"list"`                                                                 | Stable family selector. | always      |
| `data-marker`  | `"disc"`, `"decimal"`, `"none"`                                          | Consumer marker hook.   | `"disc"`    |
| `data-spacing` | `"compact"`, `"normal"`, `"loose"`                                       | Consumer spacing hook.  | `"normal"`  |
| `data-tone`    | `"accent"`, `"danger"`, `"muted"`, `"neutral"`, `"success"`, `"warning"` | Consumer tone hook.     | `"neutral"` |

## ARIA Attributes

List never sets `role`, `tabindex`, `aria-hidden`, `aria-live`, `aria-label`,
or `aria-labelledby` by default. Native `<ul>` and consumer-selected hosts keep
their own semantics. Consumers that need a labelled group, navigation region,
focusable checklist, or hidden decorative list pass ordinary Vue fallthrough
attributes themselves.

## CSS Custom Properties

List defines no CSS custom properties and ships no stylesheet. Consumers own
marker style, counter style, indentation, spacing, nesting rhythm, typography,
and color through ordinary CSS.

## Parts

| Part   | Purpose               | Default |
| ------ | --------------------- | ------- |
| `root` | Single rendered host. | always  |
