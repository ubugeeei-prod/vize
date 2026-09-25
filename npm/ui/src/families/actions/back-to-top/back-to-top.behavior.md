# BackToTop behavior contract

Normative behavior for `@vizejs/ui/back-to-top` (`back-to-top.vue`). Every row is proven by
the named test.

| State x input                            | Observable outcome                                                                                                       | Proven by                                                             |
| ---------------------------------------- | ------------------------------------------------------------------------------------------------------------------------ | --------------------------------------------------------------------- |
| scroll offset below `threshold`          | The native `button` is `hidden` (out of the tab order and accessibility tree) with `data-state="hidden"`.                | `stays hidden until the container passes the threshold`               |
| scroll offset at or past `threshold`     | The button is shown with `data-state="visible"`.                                                                         | `stays hidden until the container passes the threshold`               |
| click                                    | The container scrolls to the top smoothly, focus moves to the container (given `tabindex="-1"`), and `scroll-top` fires. | `activation scrolls smoothly to the top and focuses the container`    |
| `focusTarget`                            | Focus moves to that element instead; a focused button stays visible below the threshold until it blurs.                  | `focusTarget receives focus and a focused button stays visible`       |
| `prefers-reduced-motion: reduce`         | `behavior="smooth"` is downgraded to an instant jump.                                                                    | `reduced motion downgrades smooth scrolling and click is preventable` |
| `click` handler calls `preventDefault()` | No scrolling and no `scroll-top`.                                                                                        | `reduced motion downgrades smooth scrolling and click is preventable` |
| `target=null`                            | The window's `scrollY` drives visibility.                                                                                | `window scrolling is the default container`                           |
| expose                                   | `refresh()`, `scrollToTop()`, `visible`, `state`, `scrollTop`, and `element` are available.                              | `expose reads offsets and scrolls on demand`                          |
| SSR                                      | Server markup is byte-identical and hidden; no listeners are attached during render.                                     | `renders a hidden, deterministic button on the server`                |
| hydration                                | The server button is reused with zero warnings; scroll listening starts after mount.                                     | `hydrates without diagnostics and starts listening after mount`       |
| public types                             | Behavior, target, and numeric threshold are closed contracts.                                                            | `src/families/actions/back-to-top/back-to-top.types.test-d.ts`        |
| DOM/SSR/Vapor                            | `back-to-top.vue` compiles in every renderer lane.                                                                       | `scripts/check-renderers.ts`                                          |

| Target | Public contract                                                            |
| ------ | -------------------------------------------------------------------------- |
| Root   | native `button`, `part="root"`, `data-vize-ui="back-to-top"`, `data-state` |

BackToTop ships no stylesheet.
