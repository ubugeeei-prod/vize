# Collapsible behavior contract

Normative behavior for the `@vizejs/ui/collapsible` disclosure primitive.

| Area                 | Input                                                                        | Observable outcome                                                                                                                                     | Evidence                         |
| -------------------- | ---------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------ | -------------------------------- |
| State                | `collapsible-root.vue` receives `defaultOpen` without `open`                 | root owns uncontrolled state, trigger activation toggles content visibility, and `update:open` plus `open-change` report distinct requests             | `collapsible.test.ts`            |
| Controlled state     | `open` is provided                                                           | trigger activation emits the requested value while rendered state follows the prop until the parent accepts it                                         | `collapsible.test.ts`            |
| Disclosure semantics | `collapsible-trigger.vue` and `collapsible-content.vue` render inside a root | trigger is a native `button` with `aria-expanded` and `aria-controls`; content owns the controlled id, default `region` role, and trigger-backed label | `collapsible.test.ts`            |
| Native keyboard      | `collapsible-trigger.vue` receives Enter or Space                            | the native button activation path toggles once per key press with no custom roving focus or Accordion navigation                                       | `collapsible.test.ts`            |
| Disabled trigger     | root or trigger is disabled                                                  | trigger leaves tab order through the native `disabled` attribute and user activation does not change state                                             | `collapsible.test.ts`            |
| Preventable trigger  | trigger `click` handler calls `preventDefault()`                             | open state remains unchanged and no root state event is emitted                                                                                        | `collapsible.test.ts`            |
| SSR                  | isolated server requests render the same tree                                | generated root, trigger, and content ids are stable; closed content renders with `hidden`; hydration keeps ids and markup                              | `collapsible-ssr.test.ts`        |
| DOM/SSR/Vapor        | authored SFCs compile                                                        | root, trigger, and content compile in every renderer lane without handwritten render functions                                                         | `scripts/check-renderers.ts`     |
| Root/subpath         | consumer imports Collapsible                                                 | root and subpath bundles are byte-equivalent, CSS-free, and retain only Collapsible plus shared state/context/id helpers                               | `scripts/check-tree-shaking.mjs` |

## Public Root Props

| Prop          | Type             | Default     | Contract                                                                       |
| ------------- | ---------------- | ----------- | ------------------------------------------------------------------------------ |
| `id`          | `string \| null` | `undefined` | Consumer-owned base id. `null` and `undefined` use the deterministic fallback. |
| `open`        | `boolean`        | `undefined` | Controlled open value. `undefined` selects uncontrolled behavior.              |
| `defaultOpen` | `boolean`        | `false`     | Initial uncontrolled open value.                                               |
| `disabled`    | `boolean`        | `false`     | Disables trigger activation while preserving current state.                    |

## Public Trigger Props

| Prop             | Type                              | Default     | Contract                                                      |
| ---------------- | --------------------------------- | ----------- | ------------------------------------------------------------- |
| `type`           | `"button" \| "reset" \| "submit"` | `"button"`  | Native button submission behavior.                            |
| `disabled`       | `boolean`                         | `false`     | Disables this trigger in addition to any root disabled state. |
| `ariaLabel`      | `string`                          | `undefined` | Accessible trigger name when no visible label supplies one.   |
| `ariaLabelledby` | `string`                          | `undefined` | Space-separated ids that label the trigger.                   |

## Public Content Props

| Prop              | Type                          | Default     | Contract                                                             |
| ----------------- | ----------------------------- | ----------- | -------------------------------------------------------------------- |
| `role`            | `"group" \| "region" \| null` | `"region"`  | Content landmark role. `null` renders a plain `div`.                 |
| `ariaLabel`       | `string`                      | `undefined` | Accessible content name.                                             |
| `ariaLabelledby`  | `string \| null`              | `undefined` | Content label ids. `undefined` uses the trigger id; `null` omits it. |
| `ariaDescribedby` | `string`                      | `undefined` | Space-separated ids that describe the content.                       |

## Public Events

| Event         | Payload                                                           | Contract                                                                                        |
| ------------- | ----------------------------------------------------------------- | ----------------------------------------------------------------------------------------------- |
| `update:open` | `[value: boolean]`                                                | Emitted by `CollapsibleRoot` for every distinct requested open value.                           |
| `open-change` | `[value: boolean, previous: boolean, nativeEvent: Event \| null]` | Emitted by `CollapsibleRoot` after a distinct request.                                          |
| `click`       | `[nativeEvent: MouseEvent]`                                       | Emitted by `CollapsibleTrigger` before requesting `toggle`; prevent it to keep state unchanged. |

## Public Slots

| Component            | Slot      | Props                  |
| -------------------- | --------- | ---------------------- |
| `CollapsibleRoot`    | `default` | `CollapsibleSlotState` |
| `CollapsibleTrigger` | `default` | `CollapsibleSlotState` |
| `CollapsibleContent` | `default` | `CollapsibleSlotState` |

## Public Expose

| Component            | Exposed member                                                                   | Contract                                                                       |
| -------------------- | -------------------------------------------------------------------------------- | ------------------------------------------------------------------------------ |
| `CollapsibleRoot`    | `id`, `triggerId`, `contentId`, `open`, `disabled`, `state`                      | Read current compound ids and disclosure state.                                |
| `CollapsibleRoot`    | `setOpen(value, event?)`, `expand(event?)`, `collapse(event?)`, `toggle(event?)` | Programmatic state requests returning whether the requested value differs.     |
| `CollapsibleTrigger` | `element`, `focus(options?)`                                                     | Read and focus the native trigger button.                                      |
| `CollapsibleContent` | `element`, `open`, `disabled`, `state`, `focusContent(options?)`                 | Read state and focus the content element when the consumer makes it focusable. |

## Parts And Data

| Target  | Public contract                                                                                        |
| ------- | ------------------------------------------------------------------------------------------------------ |
| Root    | `part="root"`, `data-vize-ui="collapsible-root"`, `data-state`, `data-disabled`                        |
| Trigger | `part="trigger"`, `data-vize-ui="collapsible-trigger"`, `data-state`, `data-disabled`                  |
| Content | `part="content"`, `data-vize-ui="collapsible-content"`, `data-state`, `data-disabled`, native `hidden` |

Collapsible defines no CSS custom properties and ships no stylesheet. Consumers own all visual styling.
