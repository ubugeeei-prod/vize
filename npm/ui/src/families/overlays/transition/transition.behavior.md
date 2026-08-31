# Transition behavior contract

Normative state × input → outcome table for `transition.vue` (`@vizejs/ui/transition`).
Every row is proven by the named test in
`src/families/overlays/transition/transition.test.ts` or
`src/families/overlays/transition/transition-ssr.test.ts`; compile-only assertions live in
`src/families/overlays/transition/transition.types.test-d.ts`.

| #   | State        | Input                                     | Outcome                                                                    | Proven by                                                       |
| --- | ------------ | ----------------------------------------- | -------------------------------------------------------------------------- | --------------------------------------------------------------- |
| T1  | unmounted    | `present=false` render                    | slot stays unmounted                                                       | `keeps unmounted content out of the tree`                       |
| T2  | unmounted    | `present` becomes true                    | host mounts in `entering` until completion                                 | `enters through an explicit completion step`                    |
| T3  | entering     | computed motion duration elapses          | status becomes `present` without a manual complete                         | `auto-completes when CSS motion duration is 0`                  |
| T4  | present      | `present` becomes false                   | status becomes `exiting` and the host stays mounted                        | `exits through an explicit completion step`                     |
| T5  | any          | reduced motion                            | enter and exit skip to the terminal phase                                  | `skips motion when the user prefers it`                         |
| T6  | unmounted    | `forceMount`                              | slot stays mounted with `unmounted` status                                 | `force-mounts hidden content`                                   |
| T7  | present      | render                                    | exposed `element` is the rendered node                                     | `exposes the rendered element for composition`                  |
| T8  | SSR          | `present=true`                            | byte-identical markup starts in `present`                                  | SSR test                                                        |
| T9  | public types | invalid padding or mutating readonly refs | compilation rejects misuse                                                 | `src/families/overlays/transition/transition.types.test-d.ts`   |
| T10 | any          | `motion` recipe prop                      | `data-vize-motion` publishes the recipe; omitted prop renders no attribute | `publishes the named motion recipe for the packaged stylesheet` |
