# Scroll Spy Behavior Contract

Normative state x input -> outcome table for `@vizejs/ui/scroll-spy`
(`createScrollSpy` / `useScrollSpy`). Every row is exercised by
`scroll-spy.test.ts` or `scroll-spy-ssr.test.ts`; compile-only assertions live
in `scroll-spy.types.test-d.ts`.

| #   | State                         | Input                      | Outcome                                                                                  | Proven by                                                                        |
| --- | ----------------------------- | -------------------------- | ---------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------- |
| P1  | targets below and above line  | measure, scroll            | the last target whose top crossed `offset` is active; scroll work is batched per frame   | `activates the last target whose top crossed the activation line`                |
| P2  | before measuring / above all  | setup, measure             | `initialActiveId` is used until measured; no crossed target means `null`                 | `keeps null above the first target and honours initialActiveId before measuring` |
| P3  | container scrolled to the end | scroll                     | the last visible target activates even when its top never crosses the line               | `scrolling a container to its end activates the last visible target`             |
| P4  | any                           | `scrollTo`, reactive `ids` | navigation activates immediately with reason `navigation`; id changes rebind tracking    | `scrollTo navigates immediately and reactive ids rebind tracking`                |
| P5  | disabled / scope disposal     | scroll, scope stop         | disabled spies keep their value; `useScrollSpy` disposes with its scope and fails closed | `disabled spies keep the last value and useScrollSpy disposes with its scope`    |
| P6  | SSR and hydration             | concurrent render, hydrate | no DOM reads during render; the initial id renders and hydrates without warnings         | `scroll-spy-ssr.test.ts`                                                         |
