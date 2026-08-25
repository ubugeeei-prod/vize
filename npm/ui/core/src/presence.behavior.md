# Presence behavior contract

Normative state × input → outcome table for `presence.vue` (`@vizejs/ui/presence`).
Every row is proven by the named test in `src/presence.test.ts` or
`src/presence-ssr.test.ts`; compile-only assertions live in
`src/presence.types.test-d.ts`.

| #   | State        | Input                                     | Outcome                                             | Proven by                                      |
| --- | ------------ | ----------------------------------------- | --------------------------------------------------- | ---------------------------------------------- |
| P1  | unmounted    | `present=false` render                    | slot stays unmounted                                | `keeps unmounted content out of the tree`      |
| P2  | unmounted    | `present` becomes true                    | host mounts in `entering` until completion          | `enters through an explicit completion step`   |
| P3  | entering     | `completeAnimation` or host animationend  | status becomes `present`                            | `enters through an explicit completion step`   |
| P4  | present      | `present` becomes false                   | status becomes `exiting` and the host stays mounted | `exits through an explicit completion step`    |
| P5  | exiting      | `completeAnimation`                       | host unmounts                                       | `exits through an explicit completion step`    |
| P6  | entering     | `present` becomes false before completion | enter is canceled and the host unmounts             | `cancels an in-flight enter`                   |
| P7  | any          | reduced motion                            | enter and exit skip to the terminal phase           | `skips motion when the user prefers it`        |
| P8  | unmounted    | `forceMount`                              | slot stays mounted with `unmounted` status          | `force-mounts hidden content`                  |
| P9  | present      | render                                    | exposed `element` is the rendered node              | `exposes the rendered element for composition` |
| P10 | SSR          | `present=true`                            | byte-identical markup starts in `present`           | SSR test                                       |
| P11 | public types | invalid status or mutating readonly refs  | compilation rejects misuse                          | `src/presence.types.test-d.ts`                 |
