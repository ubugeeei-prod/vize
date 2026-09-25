# Responsive behavior contract

Normative state x input -> outcome table for `responsive-switch.vue`,
`responsive-show.vue`, and `useBreakpoint` (`@vizejs/ui/responsive`). Rows are
proven by `responsive.test.ts` and `responsive-ssr.test.ts`; compile-only
assertions live in `responsive.types.test-d.ts`.

| #   | State               | Input                             | Outcome                                                                                                                    | Proven by                                                                                        |
| --- | ------------------- | --------------------------------- | -------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------ |
| RS1 | breakpoint map      | resolve width                     | the largest breakpoint whose minimum is reached wins; unknown width or below all resolves `null`                           | `resolves the largest reached breakpoint deterministically`                                      |
| RS2 | component setup     | mount, then resize                | width starts at `ssrWidth`, adopts the real viewport after mount, and follows `resize` until unmount                       | `useBreakpoint starts at ssrWidth, reads the real width after mount, and follows resize`         |
| RS3 | effect scope        | custom map / immediate / no scope | custom names resolve, injected hosts are read immediately, and a missing scope throws `VIZE_UI_RESPONSIVE_SETUP`           | `useBreakpoint accepts custom maps, immediate reads, injected hosts, and rejects missing scopes` |
| RS4 | ResponsiveSwitch    | viewport crosses breakpoints      | the largest reached breakpoint slot renders, else `base` (above zero width), else `default`                                | `ResponsiveSwitch renders the largest reached breakpoint slot with fallbacks`                    |
| RS5 | ResponsiveShow      | outside `above`/`below` range     | content unmounts inside a `hidden` wrapper (`unmount`) or stays mounted under it (`hidden`); unknown breakpoint names hide | `ResponsiveShow unmounts or hides content outside its range`                                     |
| RS6 | SSR with `ssrWidth` | isolated requests                 | byte-identical markup chosen from `ssrWidth`                                                                               | `renders byte-identical breakpoint markup from ssrWidth across requests`                         |
| RS7 | hydration           | real width differs from SSR       | hydrates without mismatch warnings, then switches to the real viewport                                                     | `hydrates with the ssrWidth layout, then adopts the real viewport`                               |
| RS8 | DOM/SSR/Vapor       | compile                           | both SFCs compile in every renderer lane                                                                                   | `scripts/check-renderers.ts`                                                                     |

`ResponsiveShow` uses `display: contents`, so its wrapper never affects layout.
