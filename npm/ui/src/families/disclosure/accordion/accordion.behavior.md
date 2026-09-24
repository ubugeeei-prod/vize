# Accordion behavior contract

Normative behavior for the `@vizejs/ui/accordion` compound primitive, following the
[WAI-ARIA APG Accordion pattern](https://www.w3.org/WAI/ARIA/apg/patterns/accordion/).
Every row is proven by the named mounted-DOM, SSR, or compile-time test.

Accordion is built on the Collapsible contract: each `AccordionItem` provides the
Collapsible context, so `CollapsibleTrigger` and `CollapsibleContent` also work inside an
item. Native `<details name>` exclusive groups are intentionally not used: they cannot
express controlled state, the non-collapsible single mode, heading structure around the
trigger, or arrow-key navigation. `hidden="until-found"` keeps find-in-page support.

| State x input                                                     | Observable outcome                                                                                                                            | Proven by                                                                        |
| ----------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------- |
| any render                                                        | Root, item, trigger, and content ids derive from the root id and item value; the trigger is a native `button` inside a native `h3` heading.   | `renders APG accordion semantics with deterministic ids and native headings`     |
| open item                                                         | Trigger reports `aria-expanded="true"` and `aria-controls`; content is a `region` labelled by the trigger and has no `hidden`.                | `renders APG accordion semantics with deterministic ids and native headings`     |
| `type="single"`, not `collapsible`, click on the open trigger     | State is unchanged; the open trigger reports `aria-disabled="true"` and `data-locked`.                                                        | `single accordion switches items and locks the open item unless collapsible`     |
| `type="single"`, click on a closed trigger                        | The clicked item opens, the previous one closes, and `update:modelValue` plus `value-change` report the scalar model and native event.        | `single accordion switches items and locks the open item unless collapsible`     |
| `type="single"`, `collapsible`, click on the open trigger         | The item closes and the model becomes `null`.                                                                                                 | `collapsible single accordion collapses the open item to null`                   |
| `type="multiple"`, clicks                                         | Items open and close independently; the model is a readonly list in opening order.                                                            | `multiple accordion keeps independent items open and emits readonly lists`       |
| controlled `modelValue`                                           | Requests emit the next model but rendered state follows the prop until the parent accepts it.                                                 | `controlled accordion waits for the parent to accept each request`               |
| Enter or Space on a trigger                                       | Native button activation toggles once per key press.                                                                                          | `native Enter and Space activation toggle the focused trigger once`              |
| ArrowDown / ArrowUp on a vertical trigger                         | Focus moves to the next or previous enabled trigger, skipping disabled items and wrapping when `loop`; horizontal arrows are ignored.         | `arrow keys, Home, and End move focus between enabled triggers with wrapping`    |
| Home / End on a trigger                                           | Focus moves to the first or last enabled trigger.                                                                                             | `arrow keys, Home, and End move focus between enabled triggers with wrapping`    |
| `orientation="horizontal"` with `dir="rtl"`; `loop=false` at edge | ArrowLeft moves forward in RTL; at a non-looping edge the key is not consumed and focus stays.                                                | `horizontal navigation follows reading direction and loop can stop at the edges` |
| disabled root or item                                             | Triggers use native `disabled`, leave the tab order, and user activation emits nothing.                                                       | `disabled roots and items block user activation with native disabled buttons`    |
| trigger `click` handler calls `preventDefault()`                  | The item does not toggle and no model event is emitted.                                                                                       | `trigger click is preventable before the item toggles`                           |
| `hiddenUntilFound`, closed panel after mount                      | Panel carries `hidden="until-found"`; a `beforematch` event expands the item (even when disabled) and re-closing restores the keyword.        | `hidden until-found keeps closed panels searchable and beforematch expands them` |
| `hiddenUntilFound=false`                                          | Closed panels use plain `hidden`.                                                                                                             | `hidden until-found keeps closed panels searchable and beforematch expands them` |
| open panel after mount                                            | `--vize-accordion-content-height` and `--vize-accordion-content-width` publish the measured panel size for consumer animations.               | `content publishes measured size variables and supports role opt-out`            |
| `AccordionContent` `role=null`                                    | Panel renders without a landmark role or default label.                                                                                       | `content publishes measured size variables and supports role opt-out`            |
| root `headingLevel`, header `level`                               | Header renders `h{level}` with `data-level`; numeric and non-id-safe values get stable, hashed id segments.                                   | `heading level follows the root default and per-header overrides`                |
| root expose                                                       | `expand`, `collapse`, `toggle`, `expandAll`, `collapseAll`, `setValue`, `focus`, `isOpen`, `value`, and `openValues` drive and read state.    | `root and item expose typed programmatic controls`                               |
| `type="single"` root expose                                       | `expandAll` returns `false`; `collapseAll` respects `collapsible`; `setValue` bypasses the collapsible guard.                                 | `single roots refuse expandAll and keep collapseAll behind collapsible`          |
| Collapsible parts inside an item                                  | `CollapsibleTrigger` and `CollapsibleContent` read the item's ids and state and drive the accordion model.                                    | `items publish the Collapsible contract to Collapsible parts`                    |
| parts outside their provider                                      | Mounting throws `VIZE_UI_CONTEXT_MISSING`.                                                                                                    | `accordion parts require their providers`                                        |
| SSR                                                               | Isolated requests render byte-identical markup with deterministic ids, native `hidden`, and no measured style variables.                      | `renders byte-identical accordion markup across isolated SSR requests`           |
| hydration with `hiddenUntilFound`                                 | Hydration reuses server nodes with zero warnings, then upgrades closed panels to `hidden="until-found"`.                                      | `hydrates without mismatches and upgrades closed panels to hidden until-found`   |
| generic typing                                                    | `type` selects `Value \| null` or `readonly Value[]`; item values are inferred from `modelValue`/`defaultValue`; invalid shapes are rejected. | `src/families/disclosure/accordion/accordion.types.test-d.ts`                    |
| DOM/SSR/Vapor                                                     | Every part compiles in each renderer lane without handwritten render functions.                                                               | `scripts/check-renderers.ts`                                                     |

## Components

| Component               | State x input                        | Outcome                                                                                                      |
| ----------------------- | ------------------------------------ | ------------------------------------------------------------------------------------------------------------ |
| `accordion-root.vue`    | controlled or uncontrolled model     | Owns open items, deterministic ids, keyboard navigation, and emits `update:modelValue`/`value-change`.       |
| `accordion-item.vue`    | registered `value`                   | Publishes item and Collapsible contexts, `data-state`, and `data-disabled`.                                  |
| `accordion-header.vue`  | root `headingLevel` or `level`       | Renders the native heading around the trigger.                                                               |
| `accordion-trigger.vue` | click, Enter/Space, arrows, Home/End | Toggles its item through native button activation and moves focus between enabled triggers.                  |
| `accordion-content.vue` | open, closed, `hiddenUntilFound`     | Shows or hides the labelled panel, applies `hidden="until-found"` after mount, and expands on `beforematch`. |

## Public Root Props

| Prop               | Type                                  | Default      | Contract                                                                   |
| ------------------ | ------------------------------------- | ------------ | -------------------------------------------------------------------------- |
| `type`             | `"single" \| "multiple"`              | required     | Selection mode and model shape.                                            |
| `id`               | `string \| null`                      | `undefined`  | Consumer-owned base id; `null`/`undefined` use the deterministic fallback. |
| `modelValue`       | `Value \| null` or `readonly Value[]` | `undefined`  | Controlled open item(s).                                                   |
| `defaultValue`     | `Value \| null` or `readonly Value[]` | `undefined`  | Initial uncontrolled open item(s).                                         |
| `disabled`         | `boolean`                             | `false`      | Disables user activation of every item.                                    |
| `collapsible`      | `boolean`                             | `false`      | Lets a single accordion collapse its open item.                            |
| `orientation`      | `"vertical" \| "horizontal"`          | `"vertical"` | Arrow-key axis.                                                            |
| `dir`              | `"ltr" \| "rtl"`                      | `"ltr"`      | Reading direction for horizontal navigation.                               |
| `loop`             | `boolean`                             | `true`       | Wrap arrow-key navigation.                                                 |
| `hiddenUntilFound` | `boolean`                             | `false`      | Closed panels use `hidden="until-found"` after mount.                      |
| `headingLevel`     | `1 \| 2 \| 3 \| 4 \| 5 \| 6`          | `3`          | Default AccordionHeader level.                                             |

## Events

| Event               | Payload                                  | Contract                                         |
| ------------------- | ---------------------------------------- | ------------------------------------------------ |
| `update:modelValue` | `[value]`                                | Emitted for every distinct model request.        |
| `value-change`      | `[value, previous, nativeEvent \| null]` | Emitted after every distinct model request.      |
| `click` (trigger)   | `[nativeEvent: MouseEvent]`              | Preventable before the trigger toggles its item. |

## Parts And Data

| Target  | Public contract                                                                                                                         |
| ------- | --------------------------------------------------------------------------------------------------------------------------------------- |
| Root    | `part="root"`, `data-vize-ui="accordion-root"`, `data-type`, `data-orientation`, `data-disabled`, `dir`                                 |
| Item    | `part="item"`, `data-vize-ui="accordion-item"`, `data-state`, `data-orientation`, `data-disabled`                                       |
| Header  | `part="header"`, `data-vize-ui="accordion-header"`, `data-level`, `data-state`, `data-orientation`, `data-disabled`                     |
| Trigger | `part="trigger"`, `data-vize-ui="accordion-trigger"`, `data-state`, `data-orientation`, `data-disabled`, `data-locked`                  |
| Content | `part="content"`, `data-vize-ui="accordion-content"`, `data-state`, `data-hidden-until-found`, `--vize-accordion-content-height/-width` |

Accordion ships no stylesheet. Consumers own all visual styling.
