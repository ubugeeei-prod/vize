# Sticky stack behavior contract

Normative state x input -> outcome table for `sticky-stack.vue` and
`sticky-stack-item.vue` (`@vizejs/ui/sticky-stack`). Rows are proven by
`sticky-stack.test.ts` and `sticky-stack-ssr.test.ts`; compile-only assertions
live in `sticky-stack.types.test-d.ts`.

| #   | State         | Input                    | Outcome                                                                                        | Proven by                                                                           |
| --- | ------------- | ------------------------ | ---------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------- |
| SS1 | heights, base | compute offsets          | each item's `top` is the base plus earlier enabled heights; invalid values count as `0`        | `stacks offsets and skips disabled heights`                                         |
| SS2 | registrations | sort                     | items follow document order; detached items follow in registration order                       | `orders entries by document position and keeps detached ones in registration order` |
| SS3 | mounted stack | measure, scroll, remove  | measured heights replace estimates, `data-stuck`/`stuckChange` follow scroll, removals restack | `items stick below the base offset plus measured heights of earlier items`          |
| SS4 | disabled item | render                   | the item is not sticky and adds no offset to later items                                       | `disabled items scroll normally and add no offset`                                  |
| SS5 | no provider   | mount an item            | stable `VIZE_UI_CONTEXT_MISSING: StickyStack` diagnostic                                       | `items outside a stack throw the context diagnostic`                                |
| SS6 | SSR           | isolated requests        | byte-identical markup with offsets from `estimatedHeight`                                      | `renders byte-identical estimated offsets across SSR requests`                      |
| SS7 | hydration     | mount over server markup | no mismatch warnings                                                                           | `hydrates estimated offsets without mismatch warnings`                              |
| SS8 | DOM/SSR/Vapor | compile                  | both SFCs compile in every renderer lane                                                       | `scripts/check-renderers.ts`                                                        |

The stack exposes `--vize-ui-sticky-stack-height` for `scroll-margin-top` and
similar consumer CSS once its items have registered.
