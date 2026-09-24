# Image Behavior Contract

Normative state x input -> outcome table for `image-root.vue`, `image-content.vue`,
`image-placeholder.vue`, and `image-fallback.vue` (`@vizejs/ui/image`). Every row
is proven by the named test. A row without a passing test is a contract violation.

The lifecycle is `idle -> loading -> loaded | error`. `ImageRoot` owns the source
candidate chain and the lifecycle; `ImageContent` renders the native `<img>`,
`ImagePlaceholder` renders while pending, and `ImageFallback` renders after the
chain is exhausted. Every part mirrors the lifecycle through `data-status`.

| ID  | State             | Input                          | Outcome                                                                                                                              | Evidence                                                                                 |
| --- | ----------------- | ------------------------------ | ------------------------------------------------------------------------------------------------------------------------------------ | ---------------------------------------------------------------------------------------- |
| I1  | source present    | render                         | root is `loading`; native image receives the first safe candidate, `srcset`/`sizes`, `loading="lazy"`, `decoding="async"`, and hints | `renders a loading native image with placeholder, native attributes, and no fallback`    |
| I2  | loading           | native `load`                  | lifecycle becomes `loaded`, placeholder unmounts, `load` emits `(event, src)`, `statusChange` reports `load`                         | `load settles the lifecycle, hides the placeholder, and emits load`                      |
| I3  | loading           | native `error`                 | the next safe candidate is attached without `srcset`; after the last failure the image unmounts and the fallback renders             | `advances through the candidate chain and renders the fallback after the last failure`   |
| I4  | missing or unsafe | render                         | lifecycle is `error`; no `<img>` or unsafe URL is rendered; `http:` requires `allowInsecure`                                         | `missing and unsafe sources render the fallback without forwarding a source`             |
| I5  | settled           | `src` change / `retry()`       | a different chain restarts at the first candidate (`source`); an equal chain keeps state; `retry()` restarts or reports `false`      | `source replacement and retry restart the chain`                                         |
| I6  | `defer`           | render / intersect             | lifecycle stays `idle` without `src` until the root intersects (`rootMargin`), then loads and stops observing                        | `deferred images stay idle until visible and fall back to eager without observers`       |
| I7  | `defer`, no IO    | mount                          | without `IntersectionObserver` the source attaches on mount                                                                          | `deferred images stay idle until visible and fall back to eager without observers`       |
| I8  | `defer`           | `defer` becomes `false`        | the source attaches immediately                                                                                                      | `turning defer off attaches the source immediately`                                      |
| I9  | placeholder delay | pending longer / shorter       | positive `delay` renders the placeholder only after the delay and never after the image settles                                      | `delayed placeholders wait before rendering and never render after settling`             |
| I10 | hydration race    | image settled before hydration | a complete image with pixels is `loaded`; a complete image is decoded to settle `loaded` or `error`; events report `null`            | `reads images that settled before hydration attached listeners`                          |
| I11 | exposed instance  | read                           | root exposes status, attached source, candidate index/count, element, and `retry()`                                                  | `exposes typed lifecycle state and live parts`                                           |
| I12 | missing provider  | setup                          | compound parts fail closed with the shared context diagnostic                                                                        | `compound parts require a matching root provider`                                        |
| I13 | candidate policy  | `resolveImageCandidates()`     | trims, filters by the shared media-source policy, de-duplicates in order, and freezes the chain                                      | `resolves safe, de-duplicated candidate chains`                                          |
| I14 | SSR               | isolated requests              | loading, error, idle, and delayed-placeholder markup is byte-identical and never contains unsafe candidates                          | `renders byte-identical loading image markup across isolated SSR requests`               |
| I15 | SSR               | render                         | fallback, idle, and delayed-placeholder states are deterministic on the server                                                       | `renders fallback, idle, and delayed-placeholder states deterministically on the server` |
| I16 | SSR / hydration   | hydrate                        | server markup hydrates without warnings or node replacement                                                                          | `hydrates server image markup without warnings or node replacement`                      |
| I17 | types             | compile                        | lifecycle, reasons, candidate chains, slot state, and exposes are closed and read-only                                               | `image.types.test-d.ts`                                                                  |

Styling hooks: `data-status` on every part, `data-deferred` on the root, and
`data-candidate` on the image. No CSS ships with the primitive; hide the image
while `data-status="loading"` if a placeholder should occupy the same box.
