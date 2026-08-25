# Live region behavior contract

Normative state × input → outcome table for `live-region.vue` (`@vizejs/ui/live-region`).
Every row is proven by the named test in `src/live-region.test.ts` or
`src/live-region-ssr.test.ts`; compile-only assertions live in
`src/live-region.types.test-d.ts`.

| #   | State        | Input                | Outcome                                      | Proven by                                      |
| --- | ------------ | -------------------- | -------------------------------------------- | ---------------------------------------------- |
| L1  | idle         | render               | polite status region is empty                | `renders an empty polite live region`          |
| L2  | idle         | `announce`           | message is published after a clear tick      | `announces text after clearing`                |
| L3  | announced    | identical `announce` | text is re-announced by clearing first       | `announces text after clearing`                |
| L4  | idle         | assertive politeness | role is `alert` and `aria-live` is assertive | `switches to an assertive alert region`        |
| L5  | announced    | `clear`              | message is emptied                           | `clears the current announcement`              |
| L6  | any          | render               | exposed `element` is the rendered node       | `exposes the rendered element for composition` |
| L7  | SSR          | default render       | byte-identical empty polite region           | SSR test                                       |
| L8  | public types | invalid politeness   | compilation rejects misuse                   | `src/live-region.types.test-d.ts`              |
