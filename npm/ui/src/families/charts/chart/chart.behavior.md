# Chart Behavior Contract

Normative state x input -> outcome table for the `@vizejs/ui/chart` parts:
`chart-root.vue`, `chart-axis.vue`, `chart-grid.vue`, `chart-line.vue`,
`chart-area.vue`, `chart-bars.vue`, `chart-pie.vue`, `chart-points.vue`,
`chart-tooltip.vue`, `chart-crosshair.vue`, `chart-legend.vue`, and
`chart-data-table.vue`. Every row is proven by the named test in
`chart.test.ts` or `chart-ssr.test.ts`; compile-only guarantees live in
`chart.types.test-d.ts`.

The chart is a `<figure>` holding an SVG with `role="group"`,
`aria-roledescription="chart"`, and a `<title>`/`<desc>` pair. Axes, grids,
lines, and areas are `aria-hidden`. Data are reachable in two ways:

- **Navigable marks.** `ChartPoints`, and `ChartBars`/`ChartPie` when given a
  `label`, render focusable `role="img"` marks. Roving focus moves between
  them, and the chart's polite live region announces each one.
- **Data table fallback.** `ChartDataTable` renders a real `<table>` with a
  caption and column and row headers.

Every part is generic over the row type `T`, so accessors, tooltip slots, and
table columns infer the row type from `data`.

| ID  | State                         | Input                           | Outcome                                                                                              | Evidence                                                                               |
| --- | ----------------------------- | ------------------------------- | ---------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------- |
| C1  | default                       | render                          | labelled SVG group, plot translation, centered band ticks, grid lines, d3-identical line/area paths  | `renders a labelled SVG chart with plot geometry, axes, grid, and series paths`        |
| C2  | points                        | focus, arrows, Home/End, Escape | one tab stop, focus follows arrows, live announcements, tooltip and crosshair follow the active mark | `points use roving focus, arrow keys, announcements, tooltip, and crosshair`           |
| C3  | pointer over the plot         | pointermove, pointerleave       | the nearest point by x activates without focus or announcement and clears on leave                   | `pointer hover activates the nearest point without moving focus`                       |
| C4  | legend                        | toggle click, controlled prop   | `aria-pressed` toggles hide series and remove hidden marks from the tab order; controlled waits      | `the legend toggles series visibility with pressed buttons`                            |
| C5  | data table                    | render                          | captioned, visually hidden table with column and row headers and localized cells                     | `the data table fallback exposes every row with headers and localized cells`           |
| C6  | labelled bars and pie         | arrows, focus                   | bars and slices become navigable marks; unlabelled ones stay decorative; one active mark per chart   | `bars and pie slices join keyboard navigation when labelled`                           |
| C7  | responsive root               | ResizeObserver                  | renders `defaultWidth` first, then follows the container width and recomputes the plot size          | `responsive charts render the default width and follow ResizeObserver`                 |
| C8  | time, formatted, slotted axes | render                          | zone-aware `Intl` time labels, custom formatters, `tick` slot, orientation transforms                | `time axes format ticks in the scale's zone and support custom formats and tick slots` |
| C9  | missing provider              | setup                           | parts fail closed with the shared context diagnostic                                                 | `chart parts require a ChartRoot provider`                                             |
| C10 | SSR on hosts in other zones   | render twice                    | byte-identical markup, closed tooltip, first point as tab stop                                       | `renders byte-identical chart markup across requests and host time zones`              |
| C11 | hydration                     | mount over server HTML          | every part hydrates without warnings                                                                 | `hydrates the full chart without mismatch warnings`                                    |

No styles ship with the family. Marks expose `data-active`, `data-series`,
`data-hidden`, and `data-index`. The tooltip exposes
`--vize-chart-tooltip-x` and `--vize-chart-tooltip-y` for positioning.
