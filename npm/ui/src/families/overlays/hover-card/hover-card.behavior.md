# HoverCard behavior contract

Normative behavior for the `@vizejs/ui/hover-card` compound primitive. A hover card
previews supplementary content for sighted pointer and keyboard users; the trigger keeps
its native link semantics and the card is never the only way to reach its content. Every
row is proven by the named mounted-DOM, SSR, or compile-time test.

| State x input                                             | Observable outcome                                                                                                                                                     | Proven by                                                                        |
| --------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------- |
| closed, mouse or pen `pointerenter` on the trigger        | The card opens after `openDelay` (default 700 ms), never earlier; the trigger gains `aria-describedby` and content gets `data-reason="hover"`.                         | `hover opens after openDelay with link semantics and describedby wiring`         |
| open, `pointerleave` from the trigger                     | The card closes after `closeDelay` (default 300 ms); re-entering the trigger first cancels the close.                                                                  | `leaving closes after closeDelay unless the pointer returns or enters the card`  |
| open, pointer enters or leaves the card                   | Entering the card cancels a pending close; leaving the card schedules one.                                                                                             | `leaving closes after closeDelay unless the pointer returns or enters the card`  |
| open, pointer travels from the trigger toward the card    | Pointer moves inside the pointer-grace safe triangle cancel the close; moves outside schedule it again.                                                                | `pointer grace keeps the card open while travelling toward it`                   |
| closed, trigger `focus`                                   | The card opens after `openDelay` with `data-reason="focus"`.                                                                                                           | `focus opens the card and blur closes it unless focus moves into the card`       |
| open, trigger `blur`                                      | Focus moving into the card keeps it open; focus moving elsewhere schedules the close.                                                                                  | `focus opens the card and blur closes it unless focus moves into the card`       |
| open, Escape on the trigger or card; outside pointer-down | The card closes immediately through the dismissable layer; Escape is consumed and `dismiss` reports the reason.                                                        | `Escape and outside pointer-down dismiss immediately`                            |
| `touchBehavior="ignore"`, touch input                     | Touch never opens the card and taps keep native link activation.                                                                                                       | `touch is ignored by default and keeps native link activation`                   |
| `touchBehavior="long-press"`, touch held                  | Holding for `longPressDelay` opens the card (`data-reason="long-press"`), suppresses the context menu and the one follow-up click; moving past the touch slop cancels. | `long-press touch opens the card and suppresses the follow-up click`             |
| `disabled` root                                           | Pointer, focus, and touch intent are ignored and an open card closes.                                                                                                  | `disabled roots ignore intent and close an open card`                            |
| controlled `open`                                         | Intent emits `update:open` and `open-change` while rendered state follows the prop.                                                                                    | `controlled open follows the parent after emitting requests`                     |
| root expose                                               | `scheduleOpen`, `scheduleClose`, `cancelPending`, `setOpen`, normalized delays, ids, `reason`, and `state` are available.                                              | `root exposes delays and programmatic scheduling`                                |
| parts outside the root                                    | Mounting throws `VIZE_UI_CONTEXT_MISSING`.                                                                                                                             | `trigger and content require a HoverCard root`                                   |
| SSR, closed                                               | Isolated requests render byte-identical `span`-only markup that is valid inside `<p>`, with no document listeners or timers.                                           | `renders byte-identical closed hover-card markup that nests in phrasing content` |
| SSR, `defaultOpen`                                        | Content renders in place through the deferred portal and hydrates without warnings before teleporting.                                                                 | `renders default-open content in place and hydrates without diagnostics`         |
| public types                                              | Touch behavior, open reason, delays, and placement are closed, typed contracts.                                                                                        | `src/families/overlays/hover-card/hover-card.types.test-d.ts`                    |
| DOM/SSR/Vapor                                             | Root, trigger, and content compile in every renderer lane.                                                                                                             | `scripts/check-renderers.ts`                                                     |

## Components

| Component                | State x input                             | Outcome                                                                                                                   |
| ------------------------ | ----------------------------------------- | ------------------------------------------------------------------------------------------------------------------------- |
| `hover-card-root.vue`    | controlled or uncontrolled open, timers   | Owns open state, open/close delays, open reason, and emits `update:open`/`open-change`.                                   |
| `hover-card-trigger.vue` | hover, focus, blur, Escape, touch         | Schedules opening and closing, tracks the pointer-grace corridor, and handles long-press touch.                           |
| `hover-card-content.vue` | open, pointer enter/leave, focus, dismiss | Portals and positions the card, keeps it open while hovered or focused, and dismisses via Escape or outside pointer-down. |

## Public Root Props

| Prop             | Type                       | Default     | Contract                                                         |
| ---------------- | -------------------------- | ----------- | ---------------------------------------------------------------- |
| `id`             | `string \| null`           | `undefined` | Consumer-owned base id.                                          |
| `open`           | `boolean`                  | `undefined` | Controlled open state.                                           |
| `defaultOpen`    | `boolean`                  | `false`     | Initial uncontrolled open state.                                 |
| `disabled`       | `boolean`                  | `false`     | Ignore intent and close an open card.                            |
| `openDelay`      | `number`                   | `700`       | Milliseconds before hover or focus opens. Invalid values mean 0. |
| `closeDelay`     | `number`                   | `300`       | Milliseconds before departure closes. Invalid values mean 0.     |
| `touchBehavior`  | `"ignore" \| "long-press"` | `"ignore"`  | Touch policy.                                                    |
| `longPressDelay` | `number`                   | `500`       | Touch hold duration for `"long-press"`.                          |

## Parts And Data

| Target       | Public contract                                                                                                        |
| ------------ | ---------------------------------------------------------------------------------------------------------------------- |
| Root         | `<span>`, `part="root"`, `data-vize-ui="hover-card-root"`, `data-state`, `data-disabled`                               |
| Trigger      | `<a>` by default (`as`), `part="trigger"`, `data-vize-ui="hover-card-trigger"`, `data-state`, `data-disabled`          |
| Content host | `<span>`, `part="content-host"`, `data-vize-ui="hover-card-content-host"`                                              |
| Content      | `part="content"`, `data-vize-ui="hover-card-content"`, `data-state`, `data-placement`, `data-reason`, `data-top-layer` |
| Arrow        | `HoverCardArrow` is the shared `PositionerArrow` and must render inside `HoverCardContent`.                            |

HoverCard ships no stylesheet. Consumers own all visual styling.
