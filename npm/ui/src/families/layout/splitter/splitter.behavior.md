# Splitter Behavior Contract

Normative state x input -> outcome table for `splitter-group.vue`,
`splitter-panel.vue`, and `splitter-handle.vue` (`@vizejs/ui/splitter`), plus
the `useSplitterPersistence` hook and the pure layout helpers. Every row is
proven by the named test in `splitter.test.ts`, `splitter-layout.test.ts`, or
`splitter-ssr.test.ts`; compile-only guarantees live in
`splitter.types.test-d.ts`.

The handle follows the WAI-ARIA APG window splitter pattern: a focusable
`role="separator"` whose `aria-valuenow` is the size of the panel before it
(its primary pane, named by `aria-controls`) and whose `aria-orientation` is
perpendicular to the group axis. Sizes are percentages of the group and the
only inline styles are the flex declarations that carry them.

| ID  | State                    | Input                                      | Outcome                                                                                        | Evidence                                                                           |
| --- | ------------------------ | ------------------------------------------ | ---------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------- |
| S1  | default                  | render                                     | flex group, `flex-grow` sizes, separator value/min/max/controls, perpendicular orientation     | `renders the APG window splitter contract with flex-carried sizes`                 |
| S2  | focused handle           | ArrowLeft/Right, Home, End                 | resize by `keyboardStep` inside min/max; Home/End jump to the primary pane's bounds            | `arrow keys resize by the keyboard step within constraints; Home and End jump`     |
| S3  | vertical group / RTL     | ArrowUp/Down, ArrowLeft                    | vertical groups use block arrows; RTL flips inline arrows                                      | `vertical groups use ArrowUp/ArrowDown and RTL flips horizontal arrows`            |
| S4  | collapsible primary pane | Enter, Home, ArrowRight                    | Enter collapses and restores the remembered size; collapsed panes grow straight to `minSize`   | `collapsible panels snap closed past half their minimum and Enter toggles them`    |
| S5  | pointer drag             | pointerdown, move, up                      | sizes follow the pointer from the drag origin, clamp to constraints, emit start/end            | `pointer drags resize from the drag origin and report start and end`               |
| S6  | disabled group           | keys, pointer                              | no resize, separator leaves the tab order and reports `aria-disabled`                          | `disabled groups and handles ignore keyboard and pointer input`                    |
| S7  | controlled `layout`      | resize, invalid layout                     | update events fire while the parent owns sizes; wrong panel counts fall back to defaults       | `controlled layouts wait for the parent and invalid layouts fall back to defaults` |
| S8  | exposed instances        | setLayout, collapse, expand, resize, reset | typed imperative layout control with validation                                                | `exposes group and panel controls for programmatic layout changes`                 |
| S9  | nested groups            | keys on the inner handle                   | each group owns its own panels and handles                                                     | `nested groups resize independently`                                               |
| S10 | `useSplitterPersistence` | mount, resize, malformed storage           | defaults render first, the stored layout loads after mount, changes are written back           | `useSplitterPersistence loads after mount and writes later layouts`                |
| S11 | missing provider         | setup                                      | panels and handles fail closed with the shared context diagnostic                              | `compound parts require a matching group provider`                                 |
| S12 | layout helpers           | validate, default, clamp, resize           | remainder sharing, clamping, cascading resize, and midpoint collapse snapping                  | `splitter-layout.test.ts`                                                          |
| S13 | SSR and hydration        | isolated render/mount                      | byte-identical exact sizes, controlled layouts before registration, hydration without warnings | `splitter-ssr.test.ts`                                                             |

## Notes

- For exact server rendering give every panel a `defaultSize`, or pass
  `defaultLayout`/`layout` to the group; unsized panels share the remainder
  once all panels have registered.
- Resizing only moves the boundary between the handle's neighbours, cascading
  through further panels when a neighbour reaches its minimum.
