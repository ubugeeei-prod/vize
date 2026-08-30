# Motion behavior contract

Normative contract for the motion family (`@vizejs/ui/motion`): design tokens,
named easing curves, shared recipes, and platform adapters. The stylesheet
contract is proven on the packaged `dist/style.css` (the pack pipeline lowers
`src/motion.css` to the declared browser floor; see
`style-pipeline.behavior.md`) in `src/motion-stylesheet.test.ts`; runtime rows
are proven by the named test in `src/motion.test.ts` or
`src/motion-ssr.test.ts`; compile-only assertions live in
`src/motion.types.test-d.ts`.

| #   | State               | Input                                           | Outcome                                                                                                                     | Proven by                                                           |
| --- | ------------------- | ----------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------- |
| M1  | packaged stylesheet | consumer imports any motion entry               | duration, delay, and easing tokens ship in `@layer vize.ui` at zero specificity                                             | `ships layered zero-specificity motion tokens matching the mirrors` |
| M2  | packaged stylesheet | `data-vize-motion` recipe + presence/transition | enter and exit recipes animate through the published status attributes                                                      | `pairs enter and exit recipes with presence and transition hooks`   |
| M3  | packaged stylesheet | `data-vize-motion~="move"` or emphasis recipe   | move glides position properties; pulse and shake run token-timed animations                                                 | `ships move and emphasis recipes with token-driven timing`          |
| M4  | packaged stylesheet | `enter` or `reveal` recipe                      | `@starting-style` entry and scroll-driven reveal ship verbatim for engines newer than the floor                             | `ships the starting-style and scroll-driven recipes verbatim`       |
| M5  | packaged stylesheet | `prefers-reduced-motion: reduce`                | every duration and delay token zeroes, recipes hard-stop, and reveal stands down                                            | `zeroes packaged motion under reduced motion`                       |
| M6  | packaged stylesheet | `forced-colors: active`                         | recipe animations and transitions stand down                                                                                | `stands down under forced colors`                                   |
| M7  | any                 | `setMotionTokens(element, overrides)`           | overrides apply to the element inline and the restore callback reinstates values                                            | `applies and restores token overrides on a real element`            |
| M8  | any                 | unknown token name or empty value               | `motionTokenProperty`/`setMotionTokens` throw `VIZE_UI_MOTION_TOKEN`                                                        | `rejects unknown tokens and empty override values`                  |
| M9  | no native support   | `startViewTransition(update)`                   | update runs directly; handle resolves with `native: false`                                                                  | `falls back synchronously without native view transitions`          |
| M10 | reduced motion      | `startViewTransition(update)`                   | native transition is skipped unless `respectReducedMotion: false`                                                           | `skips the native transition under reduced motion`                  |
| M11 | native support      | `startViewTransition(update)`                   | native transition drives the update; promises and `skipTransition` pass through                                             | `drives the native view transition when supported`                  |
| M12 | any                 | non-function update or non-boolean option       | `startViewTransition` throws `VIZE_UI_MOTION_OPTION`                                                                        | `rejects invalid view transition input`                             |
| M13 | mounted             | `prefers-reduced-motion` changes                | `useReducedMotion` ref tracks the media query and releases its listener on dispose                                          | `tracks reduced motion reactively in a mounted consumer`            |
| M14 | any                 | feature probes                                  | `supportsStartingStyle`/`supportsScrollDrivenAnimations`/`supportsViewTransitions` report platform support without throwing | `probes platform support without throwing`                          |
| M15 | SSR                 | render with adapters in setup                   | byte-identical markup; no `matchMedia`/`document` access is required                                                        | `renders byte-identical SSR markup without platform globals`        |
| M16 | SSR                 | hydration                                       | hydrating a motion consumer emits no diagnostics and keeps server nodes                                                     | `hydrates a motion consumer without replacement or diagnostics`     |
| M17 | public types        | invalid token names or mutated records          | compilation rejects misuse                                                                                                  | `src/motion.types.test-d.ts`                                        |

## Extension contract

- Every token is a `--vize-ui-motion-*` custom property defined at zero
  specificity inside `@layer vize.ui`: any consumer rule outside the layer —
  or a `setMotionTokens` override on an ancestor — replaces curves, durations,
  delays, and recipe hooks without forking components.
- Recipe hooks (`enter-duration`, `enter-easing`, `exit-duration`,
  `exit-easing`, `move-duration`, `move-easing`, `emphasis-duration`,
  `emphasis-easing`, `slide-distance`, `scale-from`) retune one phase
  everywhere; the base scales stay untouched.
- `data-vize-motion` accepts space-separated recipe names. Enter/exit recipes
  (`fade`, `scale`, `slide`) activate through `data-vize-presence` /
  `data-vize-transition` status attributes, so the presence and transition
  primitives complete from real `animationend` events; `families/overlays/presence/presence.vue` and
  `transition.vue` publish the attribute through their `motion` prop.
