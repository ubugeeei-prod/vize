# Sticky behavior contract

Normative behavior for `@vizejs/ui/sticky` (`Sticky`, alias `Affix`). `sticky.vue` pins with
native `position: sticky` — no scroll listeners move the element — and only observes geometry
to report whether it is currently pinned. Every row is proven by the named test.

| State x input                                           | Observable outcome                                                                                                           | Proven by                                                                |
| ------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------ |
| any render                                              | The native element (`as`) carries `position: sticky`, the side inset, `--vize-sticky-offset`, `data-side`, and `data-state`. | `renders native sticky positioning with offset variables and slot state` |
| mounted                                                 | An `IntersectionObserver` watches the element with a root margin that trims `offset + 1` px from the pinned edge.            | `renders native sticky positioning with offset variables and slot state` |
| box rests on its offset line                            | `data-stuck="true"`, `data-state="stuck"`, and `stuck-change` fires once with `true`.                                        | `reports stuck when the box rests on its offset line and releases after` |
| box sits below the line or scrolled away with its block | `data-stuck` is removed and `stuck-change` fires once with `false`.                                                          | `reports stuck when the box rests on its offset line and releases after` |
| `side="bottom"` with a custom `root`                    | Geometry is measured against the container bottom and the observer uses that root.                                           | `bottom stickiness measures against the container bottom edge`           |
| `disabled`                                              | Sticky positioning is removed and the observer disconnects; re-enabling reconnects with the current offset.                  | `disabled stickiness drops positioning and observation`                  |
| expose `refresh()`                                      | Re-reads geometry synchronously and returns the new stuck value.                                                             | `expose refresh re-reads geometry on demand`                             |
| geometry helpers                                        | `isStickyStuck` and `stickyRootMargin` describe the one-pixel pinned band on both edges.                                     | `geometry helpers describe the pinned band on both edges`                |
| SSR                                                     | Server markup is byte-identical, includes the sticky style, and never reports stuck.                                         | `renders byte-identical sticky markup without stuck state on the server` |
| hydration                                               | The server element is reused with zero warnings; observation starts after mount.                                             | `hydrates the server box without diagnostics`                            |
| public types                                            | Side, state, native tag, and numeric offset are closed contracts.                                                            | `src/families/layout/sticky/sticky.types.test-d.ts`                      |
| DOM/SSR/Vapor                                           | `sticky.vue` compiles in every renderer lane.                                                                                | `scripts/check-renderers.ts`                                             |

## Parts And Data

| Target | Public contract                                                                                                          |
| ------ | ------------------------------------------------------------------------------------------------------------------------ |
| Root   | `part="root"`, `data-vize-ui="sticky"`, `data-side`, `data-state`, `data-stuck`, `data-disabled`, `--vize-sticky-offset` |

Sticky ships no stylesheet beyond the inline sticky declaration required for the behavior.
