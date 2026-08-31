# Table behavior contract

Normative state x input -> outcome table for the semantic Table family
(`@vizejs/ui/table`). Every row is proven by the named mounted-DOM, SSR,
renderer, type, size, or tree-shaking gate. A row without a passing test is a
contract violation.

Source SFCs covered by this contract: `table.vue`, `table-caption.vue`,
`table-head.vue`, `table-body.vue`, `table-row.vue`, `table-header.vue`, and
`table-cell.vue`.

| #   | State           | Input             | Outcome                                                                                                  | Proven by                                                                 |
| --- | --------------- | ----------------- | -------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------- |
| T1  | default         | render / Tab      | renders native `<table>`, caption, head, body, rows, header cells, data cells, parts, hooks, and no grid | `renders native table composition without grid machinery or visual CSS`   |
| T2  | native attrs    | render            | forwards native `scope`, `abbr`, `headers`, `colspan`, and `rowspan` while mirroring data and CSS hooks  | `mirrors semantic hooks and native span/idref attributes`                 |
| T3  | root state      | slot/expose       | passes layout, density, and inline style to the slot and exposes live root state                         | `passes slot state and exposes live table state`                          |
| T4  | compound state  | slot/expose       | exposes caption side, section literals, row selected state, and header/cell metadata                     | `passes compound slot state and exposes native element contracts`         |
| T5  | SSR composition | isolated requests | renders byte-identical valid table markup without request-global state                                   | `renders byte-identical native table markup across isolated SSR requests` |
| T6  | SSR/hydration   | hydrate           | server table markup hydrates without warnings, node replacement, or parser repair drift                  | `hydrates semantic table markup without replacing SSR nodes`              |
| T7  | DOM/SSR/Vapor   | compile           | authored SFCs and a compound consumer compile in every renderer lane without warnings or fallback        | `scripts/check-renderers.ts`                                              |
| T8  | public types    | invalid contract  | TypeScript rejects unsupported layout, density, caption side, row state, scope, and alignment tokens     | `src/families/data/table/table.types.test-d.ts`                           |
| T9  | root/subpath    | consumer bundle   | root and subpath consumers retain only Table, emit no CSS, and stay within gzip budget                   | `scripts/check-tree-shaking.mjs`                                          |

## Props

| Component      | Prop      | Type                                         | Purpose                                                        | Default     |
| -------------- | --------- | -------------------------------------------- | -------------------------------------------------------------- | ----------- |
| `Table`        | `layout`  | `"auto" \| "fixed"`                          | Native CSS `table-layout` mode.                                | `"auto"`    |
| `Table`        | `density` | `"compact" \| "normal" \| "spacious"`        | Consumer density hook mirrored to `data-density`.              | `"normal"`  |
| `TableCaption` | `side`    | `"top" \| "bottom"`                          | Native CSS `caption-side` placement.                           | `"top"`     |
| `TableRow`     | `state`   | `"default" \| "selected"`                    | Consumer-owned row state hook.                                 | `"default"` |
| `TableHeader`  | `scope`   | `"col" \| "colgroup" \| "row" \| "rowgroup"` | Native header scope.                                           | `"col"`     |
| `TableHeader`  | `abbr`    | `string`                                     | Native abbreviated header text.                                | `undefined` |
| `TableHeader`  | `colspan` | `number`                                     | Native column span.                                            | `undefined` |
| `TableHeader`  | `rowspan` | `number`                                     | Native row span.                                               | `undefined` |
| `TableHeader`  | `align`   | `"start" \| "center" \| "end"`               | Logical cell alignment hook.                                   | `"start"`   |
| `TableCell`    | `headers` | `string`                                     | Space-separated native header ids that describe the data cell. | `undefined` |
| `TableCell`    | `colspan` | `number`                                     | Native column span.                                            | `undefined` |
| `TableCell`    | `rowspan` | `number`                                     | Native row span.                                               | `undefined` |
| `TableCell`    | `align`   | `"start" \| "center" \| "end"`               | Logical cell alignment hook.                                   | `"start"`   |

`TableHead` and `TableBody` accept no component props. They preserve ordinary
Vue fallthrough attributes on their native section roots.

## Emits

The Table family emits no custom events. Sorting, selection, pagination,
virtualization, editing, and resizing are deliberately outside this semantic
slice.

## Slots

| Component      | Slot      | Props                   | Purpose                              | Default |
| -------------- | --------- | ----------------------- | ------------------------------------ | ------- |
| `Table`        | `default` | `TableSlotState`        | Render caption and table sections.   | empty   |
| `TableCaption` | `default` | `TableCaptionSlotState` | Render native caption content.       | empty   |
| `TableHead`    | `default` | `TableHeadSlotState`    | Render one or more native rows.      | empty   |
| `TableBody`    | `default` | `TableBodySlotState`    | Render one or more native rows.      | empty   |
| `TableRow`     | `default` | `TableRowSlotState`     | Render native header and data cells. | empty   |
| `TableHeader`  | `default` | `TableHeaderSlotState`  | Render native header cell content.   | empty   |
| `TableCell`    | `default` | `TableCellSlotState`    | Render native data cell content.     | empty   |

## Expose

| Component      | Expose              | Type                                    | Purpose                         | Default  |
| -------------- | ------------------- | --------------------------------------- | ------------------------------- | -------- |
| `Table`        | `element`           | `HTMLTableElement \| null`              | Rendered native table.          | `null`   |
| `Table`        | `layout`, `density` | `TableLayout`, `TableDensity`           | Current root hook state.        | defaults |
| `Table`        | `style`             | `TableStyle`                            | Inline table style contract.    | object   |
| `TableCaption` | `element`           | `HTMLTableCaptionElement \| null`       | Rendered native caption.        | `null`   |
| `TableCaption` | `side`, `style`     | `TableCaptionSide`, `TableCaptionStyle` | Current caption hook state.     | defaults |
| `TableHead`    | `element`           | `HTMLTableSectionElement \| null`       | Rendered native table head.     | `null`   |
| `TableHead`    | `section`           | `"head"`                                | Stable section token.           | `"head"` |
| `TableBody`    | `element`           | `HTMLTableSectionElement \| null`       | Rendered native table body.     | `null`   |
| `TableBody`    | `section`           | `"body"`                                | Stable section token.           | `"body"` |
| `TableRow`     | `element`           | `HTMLTableRowElement \| null`           | Rendered native table row.      | `null`   |
| `TableRow`     | `state`, `selected` | `TableRowState`, `boolean`              | Current row hook state.         | defaults |
| `TableHeader`  | `element`           | `HTMLTableCellElement \| null`          | Rendered native header cell.    | `null`   |
| `TableHeader`  | cell metadata       | `TableHeaderSlotState`                  | Current header cell hook state. | defaults |
| `TableCell`    | `element`           | `HTMLTableCellElement \| null`          | Rendered native data cell.      | `null`   |
| `TableCell`    | cell metadata       | `TableCellSlotState`                    | Current data cell hook state.   | defaults |

## Data Attributes

| Component      | Attribute           | Values                                       | Purpose                         | Default     |
| -------------- | ------------------- | -------------------------------------------- | ------------------------------- | ----------- |
| all            | `data-vize-ui`      | `TableDataName`                              | Stable family selector.         | always      |
| `Table`        | `data-layout`       | `"auto"`, `"fixed"`                          | Native layout hook.             | `"auto"`    |
| `Table`        | `data-density`      | `"compact"`, `"normal"`, `"spacious"`        | Consumer density hook.          | `"normal"`  |
| `TableCaption` | `data-caption-side` | `"top"`, `"bottom"`                          | Native caption placement hook.  | `"top"`     |
| `TableHead`    | `data-section`      | `"head"`                                     | Stable section selector.        | `"head"`    |
| `TableBody`    | `data-section`      | `"body"`                                     | Stable section selector.        | `"body"`    |
| `TableRow`     | `data-state`        | `"default"`, `"selected"`                    | Consumer row state hook.        | `"default"` |
| `TableRow`     | `data-selected`     | `"true"`                                     | Present only for selected rows. | absent      |
| `TableHeader`  | `data-scope`        | `"col"`, `"colgroup"`, `"row"`, `"rowgroup"` | Native header scope hook.       | `"col"`     |
| cells          | `data-align`        | `"start"`, `"center"`, `"end"`               | Logical alignment hook.         | `"start"`   |
| cells          | `data-colspan`      | numeric string                               | Present when `colspan` is set.  | absent      |
| cells          | `data-rowspan`      | numeric string                               | Present when `rowspan` is set.  | absent      |

## ARIA Attributes

Table never sets `role`, `tabindex`, `aria-hidden`, `aria-live`, sorting
attributes, grid semantics, or keyboard behavior. Native table semantics come
from `<table>`, `<caption>`, `<thead>`, `<tbody>`, `<tr>`, `<th>`, and `<td>`.
Consumers may pass ordinary fallthrough attributes when their table needs an
accessible name, description, sort state, selection state, or region wrapper.

## CSS Custom Properties

| Property                       | Component      | Purpose                                 | Default   |
| ------------------------------ | -------------- | --------------------------------------- | --------- |
| `--vize-ui-table-layout`       | `Table`        | Mirrors the native `table-layout` prop. | `"auto"`  |
| `--vize-ui-table-caption-side` | `TableCaption` | Mirrors the native `caption-side` prop. | `"top"`   |
| `--vize-ui-table-cell-align`   | cells          | Mirrors the native `text-align` prop.   | `"start"` |

No stylesheet is emitted. These custom properties are authored as inline
native style hooks so consumers can override or cascade around the semantic
primitive without importing a preset.

## Parts

| Component      | Part      | Purpose             | Default |
| -------------- | --------- | ------------------- | ------- |
| `Table`        | `root`    | Native table root.  | always  |
| `TableCaption` | `caption` | Native caption.     | always  |
| `TableHead`    | `head`    | Native table head.  | always  |
| `TableBody`    | `body`    | Native table body.  | always  |
| `TableRow`     | `row`     | Native table row.   | always  |
| `TableHeader`  | `header`  | Native header cell. | always  |
| `TableCell`    | `cell`    | Native data cell.   | always  |
