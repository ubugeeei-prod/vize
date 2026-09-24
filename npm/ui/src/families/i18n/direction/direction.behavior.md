# Direction behavior contract

Normative behavior for `@vizejs/ui/direction` (`direction-provider.vue`, `useResolvedDirection`).
Every row is proven by the named test.

| State x input                                          | Observable outcome                                                                                                            | Proven by                                                                      |
| ------------------------------------------------------ | ----------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------ |
| `DirectionProvider dir="rtl"`                          | A native wrapper (`as`, default `div`) carries `dir="rtl"`, `data-vize-ui="direction-provider"`, and the slot receives `dir`. | `provider renders a dir wrapper and publishes the direction to its slot`       |
| `useResolvedDirection(local)`                          | Resolves the local value first, then the nearest provided direction, then `"ltr"`; unknown values are ignored.                | `useResolvedDirection prefers the local value, then the provider, then ltr`    |
| `as=null` nested provider                              | Renders the slot without a wrapper while still inheriting and re-publishing the direction.                                    | `useResolvedDirection prefers the local value, then the provider, then ltr`    |
| direction-aware family without `dir` inside a provider | The family renders `dir` from the provider and flips horizontal arrow keys (Accordion shown).                                 | `direction-aware families inherit the provider and flip horizontal arrow keys` |
| family with an explicit `dir` prop                     | The prop overrides the provided direction.                                                                                    | `an explicit dir prop overrides the provided direction`                        |
| `LocaleProvider direction="rtl"`                       | LocaleProvider also publishes the direction context, so families inherit it.                                                  | `LocaleProvider publishes its resolved direction to direction-aware families`  |
| SSR                                                    | Inherited direction is resolved without reading `document.dir`; output is byte-identical across requests.                     | `server output resolves inherited direction without reading the document`      |
| hydration                                              | Server markup is reused with zero warnings.                                                                                   | `hydrates inherited direction without diagnostics`                             |
| public types                                           | `Direction` is a closed union and `as` accepts native tags or `null`.                                                         | `src/families/i18n/direction/direction.types.test-d.ts`                        |
| DOM/SSR/Vapor                                          | `direction-provider.vue` compiles in every renderer lane.                                                                     | `scripts/check-renderers.ts`                                                   |

## Direction-aware families

Accordion, Tabs, Tree, Carousel, NavigationMenu, Menu, Menubar, DropdownMenu, ContextMenu,
SplitterGroup, and Resizable default `dir` to `undefined` and resolve it with
`useResolvedDirection`. Explicit `dir="ltr" | "rtl"` keeps its previous meaning.

| Target | Public contract                                                                |
| ------ | ------------------------------------------------------------------------------ |
| Root   | native `as` element, `part="root"`, `data-vize-ui="direction-provider"`, `dir` |
