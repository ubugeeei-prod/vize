# Fresco Compatibility Matrix

This is the reference compatibility matrix for
[#3113 — feat(fresco): build the Vue-native OpenTUI component platform](https://github.com/ubugeeei-prod/vize/issues/3113).
It records what `@vizejs/fresco`, `@vizejs/fresco-native`, and the `vize_fresco` crate implement
today against the three reference surfaces:

- [React Ink](https://github.com/vadimdemedes/ink) — API-compatibility baseline (`Box`, `Text`,
  `Static`, `Transform`, `render`, the hook set, and stream/render options).
- [OpenTUI](https://opentui.com/) — capability baseline for the native rendering model
  (renderables, lifecycle, focus, input routing, scrolling, culling, custom drawing).
- [Vue TermUI](https://github.com/posva/vue-termui) — Vue-native ergonomics baseline (SFC,
  Composition API composables). Vue TermUI's current `main` branch renders through OpenTUI.

Statuses reflect audited source, not intentions. A PR that changes a status must update the
affected rows in the same PR.

## Audit basis

| Surface          | Audited source                                                                                            |
| ---------------- | --------------------------------------------------------------------------------------------------------- |
| Fresco Vue layer | `npm/fresco/src` (`index.ts`, `app.ts`, `renderer.ts`, `accessibility.ts`, `components/`, `composables/`) |
| Native bindings  | `npm/fresco-native/index.d.ts` (checked-in declarations; node kind mirrors its Rust annotation)           |
| Rust renderer    | `crates/vize_fresco/src` (`component/`, `input/`, `layout/`, `render/`, `terminal/`, `text/`, `napi/`)    |
| Ink              | `vadimdemedes/ink` README + v6 type declarations, retrieved 2026-07-19                                    |
| OpenTUI          | opentui.com core-concepts/renderables documentation, retrieved 2026-07-19                                 |
| Vue TermUI       | `posva/vue-termui` `main` branch exports (OpenTUI-based rewrite), retrieved 2026-07-19                    |

Audited at commit `628ea1e7d` (2026-07-19). External surfaces (Ink, OpenTUI, Vue TermUI) are
pinned by retrieval date (2026-07-19) rather than by an upstream revision, since none publish
per-release documentation snapshots; re-audit against those sources as of that date. `—` means the
surface has no equivalent.

## Status legend

| Status      | Meaning                                                                           |
| ----------- | --------------------------------------------------------------------------------- |
| Implemented | Present in Fresco and owns the reference behavior end to end.                     |
| Partial     | Present, but a defined part of the reference behavior is missing (noted per row). |
| Planned     | Not present yet; scheduled in a #3113 milestone (noted per row).                  |
| Not planned | Intentionally out of scope; see "Intentional differences".                        |

## Component surface

### Ink core components

| Ink           | OpenTUI          | Vue TermUI | Fresco      | Status      | Notes                                                                                   |
| ------------- | ---------------- | ---------- | ----------- | ----------- | --------------------------------------------------------------------------------------- |
| `<Box>`       | `BoxRenderable`  | `Box`      | `Box`       | Implemented | Taffy flexbox, `borderStyle`, colors, padding/margin/gap, absolute positioning.         |
| `<Text>`      | `TextRenderable` | `Text`     | `Text`      | Implemented | Styling plus `wrap`/`truncate`/`truncate-start`/`truncate-middle`/`truncate-end` modes. |
| `<Newline>`   | —                | `Newline`  | `Newline`   | Implemented |                                                                                         |
| `<Spacer>`    | —                | —          | `Spacer`    | Implemented |                                                                                         |
| `<Static>`    | —                | —          | `Static`    | Implemented | Append-only output above the live area; also used by the non-interactive/CI path.       |
| `<Transform>` | —                | —          | `Transform` | Implemented | `(children, index) => string`; supports `accessibilityLabel` in screen-reader output.   |

### Interaction primitives

Ink ships these as ecosystem packages (`ink-text-input`, `ink-select-input`, ...); OpenTUI and
Vue TermUI ship them in core. Only `TextInput` currently owns its keyboard behavior; the other
interactive components render selection/focus state from props and `v-model` but expect the host
app to wire `useInput` (their internal movement/select handlers are not connected yet — M2).

| Ink ecosystem      | OpenTUI               | Vue TermUI  | Fresco                                      | Status  | Notes                                                                                                                   |
| ------------------ | --------------------- | ----------- | ------------------------------------------- | ------- | ----------------------------------------------------------------------------------------------------------------------- |
| `ink-text-input`   | `InputRenderable`     | `Input`     | `TextInput`                                 | Partial | Owns keyboard via `useInput`: cursor, mask, submit, IME composition. No selection/history/word movement/clipboard (M2). |
| —                  | `TextareaRenderable`  | `Textarea`  | `TextArea`                                  | Partial | Renders multiline value/cursor state; editing behavior not owned by the component (M2).                                 |
| `ink-select-input` | `SelectRenderable`    | `Select`    | `Select`                                    | Partial | Display + `v-model`; keyboard navigation handlers exist but are not wired (M2).                                         |
| —                  | `TabSelectRenderable` | `TabSelect` | `Tabs`                                      | Partial | Display + `v-model`; tab switching keys not owned (M2).                                                                 |
| —                  | —                     | —           | `Checkbox`, `RadioGroup`, `Confirm`, `Form` | Partial | State rendering + `v-model`/emits; keyboard, validation, and focus semantics pending (M2).                              |
| —                  | —                     | —           | `List`, `Menu`, `Tree`                      | Partial | Typed items, selection/expansion rendering; navigation, typeahead, and scrolling pending (M2).                          |
| —                  | —                     | —           | `Modal`, `Tooltip`                          | Partial | Overlay rendering; stacking, focus trap/restore, escape policy, anchor placement pending (M2).                          |
| —                  | `SliderRenderable`    | —           | —                                           | Planned | Keyboard/mouse slider with range-safe typed model (M2).                                                                 |

### Display and rich components

| Ink ecosystem      | OpenTUI                 | Vue TermUI    | Fresco                      | Status      | Notes                                                                                                      |
| ------------------ | ----------------------- | ------------- | --------------------------- | ----------- | ---------------------------------------------------------------------------------------------------------- |
| `ink-spinner`      | —                       | —             | `Spinner`                   | Implemented | Animated via the shared `useAnimation` timer.                                                              |
| `ink-progress-bar` | —                       | `ProgressBar` | `ProgressBar`               | Implemented |                                                                                                            |
| `ink-table`        | —                       | —             | `Table`                     | Partial     | Typed rows/columns, alignment; sorting hooks, resize, and virtualization pending (M3).                     |
| `ink-link`         | —                       | —             | `Link`                      | Partial     | Styled label with optional URL suffix; no OSC 8 hyperlink escape.                                          |
| —                  | `CodeRenderable`        | —             | `Code`                      | Partial     | Static block with line numbers, highlight lines, border; no syntax provider, streaming, or selection (M3). |
| —                  | `MarkdownRenderable`    | `Markdown`    | —                           | Planned     | Lightweight CommonMark core with provider extension points (M3).                                           |
| —                  | `LineNumberRenderable`  | —             | —                           | Planned     | Standalone gutter primitive (M3); `Code` embeds its own line numbers today.                                |
| —                  | `DiffRenderable`        | —             | —                           | Planned     | Unified/split views, hunks, word-level highlights, large-file culling (M3).                                |
| —                  | `ASCIIFontRenderable`   | —             | —                           | Planned     | Optional tree-shakeable package (M3).                                                                      |
| —                  | `QRCodeRenderable`      | —             | —                           | Planned     | Optional package/provider (M3).                                                                            |
| —                  | `ScrollBoxRenderable`   | `ScrollBox`   | —                           | Planned     | Native scroll viewport + culling (M1). `overflow: "scroll"` is layout metadata only today.                 |
| —                  | `ScrollBarRenderable`   | —             | —                           | Planned     | Ships with the ScrollBox model (M1).                                                                       |
| —                  | `FrameBufferRenderable` | —             | — (`NodeKind::Raw` in Rust) | Partial     | Raw line content exists in `vize_fresco` but has no public JS API (M1).                                    |

### Vize-native components (no direct reference equivalent)

All are Implemented as display components and follow the same M2/M3 interaction and theming
polish track as the tables above.

| Group      | Components                                           |
| ---------- | ---------------------------------------------------- |
| Layout     | `Stack`/`HStack`/`VStack`, `Grid`, `Card`, `Divider` |
| Feedback   | `Alert`, `Badge`, `Timer`, `Avatar`                  |
| Navigation | `Breadcrumb`, `Stepper` (`Menu`, `Tabs` above)       |
| Chrome     | `StatusBar`, `Header`, `KeyHint`                     |

## Composables

Fresco implements the full Ink v6 hook list under the same names, as Vue composables.

| Ink                                | OpenTUI                           | Vue TermUI                   | Fresco                             | Status      | Notes                                                                                                                                                      |
| ---------------------------------- | --------------------------------- | ---------------------------- | ---------------------------------- | ----------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `useInput`                         | `onKeyDown` (per renderable)      | `onKeyDown`/`onKeyUp`        | `useInput` (+ `useKeyPress`)       | Partial     | Global subscription with Kitty press/repeat/release; not routed per element, no capture/bubble (M1).                                                       |
| `usePaste`                         | `onPaste`                         | —                            | `usePaste`                         | Implemented | Bracketed paste.                                                                                                                                           |
| `useApp`                           | renderer lifecycle                | `useExit`                    | `useApp`                           | Partial     | `exit`, terminal `width`/`height` refs, `render`, `clear`; `waitUntilRenderFlush` resolves immediately with no real flush barrier yet (see top-level API). |
| `useStdin`/`useStdout`/`useStderr` | —                                 | —                            | `useStdin`/`useStdout`/`useStderr` | Implemented | Backed by the app streams context; honors custom streams.                                                                                                  |
| `useFocus`                         | `focus()`/`blur()` + focus events | `useCurrentFocusedElement`   | `useFocus`                         | Partial     | Flat ordered registry with `isActive`/`autoFocus`; global Tab/Shift+Tab traversal; no traps, delegation, or descendant-focus state (M1).                   |
| `useFocusManager`                  | renderer focus APIs               | `useFocusManager`            | `useFocusManager`                  | Implemented | `focus`, `focusNext`/`focusPrevious`, enable/disable, `activeId`, `focusableIds`.                                                                          |
| `useWindowSize`                    | `onResize` hook                   | `useTerminalSize`/`onResize` | `useWindowSize`                    | Implemented |                                                                                                                                                            |
| `useCursor`                        | cursor ownership                  | —                            | `useCursor`                        | Implemented | Position override + visibility via native cursor control.                                                                                                  |
| `useAnimation`                     | `onUpdate(deltaTime)`             | `useInterval`/`useTimeout`   | `useAnimation`                     | Implemented | Shared timer; `frame`/`time`/`delta`/`reset`.                                                                                                              |
| `useBoxMetrics`                    | —                                 | —                            | `useBoxMetrics`                    | Implemented | Reads last-render layout results.                                                                                                                          |
| `useIsScreenReaderEnabled`         | —                                 | —                            | `useIsScreenReaderEnabled`         | Implemented |                                                                                                                                                            |
| —                                  | —                                 | —                            | `useIme`                           | Implemented | Fresco-specific: IME mode, preedit, and candidate state.                                                                                                   |

## Top-level API and render options

| Ink                                                             | Fresco                                | Status      | Notes                                                                                                                       |
| --------------------------------------------------------------- | ------------------------------------- | ----------- | --------------------------------------------------------------------------------------------------------------------------- |
| `render(tree, options?)` → instance                             | `render(root, options?)` → instance   | Implemented | Also accepts a bare writable stream as `options`, like Ink.                                                                 |
| instance `rerender`/`unmount`/`waitUntilExit`/`cleanup`/`clear` | same                                  | Implemented |                                                                                                                             |
| instance `waitUntilRenderFlush`                                 | `waitUntilRenderFlush`                | Partial     | Resolves immediately; no real flush barrier yet.                                                                            |
| `renderToString`                                                | `renderToString`                      | Partial     | Plain-text projection (no ANSI styling, no flex layout beyond column stacking); retained as a low-level snapshot helper.    |
| `measureElement`                                                | `measureElement`                      | Implemented | Backed by native last-render layouts.                                                                                       |
| —                                                               | `createApp(root, options?)` + `mount` | Implemented | Vue-native entry point underneath the Ink-compatible `render`.                                                              |
| Testing: `ink-testing-library`                                  | `@vizejs/fresco/testing`              | Partial     | Initial `renderTui`, `lastFrame`, frame/protocol snapshots, and input injection; layout inspection remains planned (M0/M4). |

Render options: `stdout`, `stdin`, `stderr`, `debug`, `exitOnCtrlC`, `patchConsole`, `onRender`,
`isScreenReaderEnabled`, `maxFps`, `interactive`, `alternateScreen`, and `kittyKeyboard` are
implemented with Ink semantics. `incrementalRendering` and `concurrent` are accepted for API
parity but are intentional no-ops (see "Intentional differences"). Fresco adds `mouse` and
`onError`.

## Runtime and renderer capabilities

| Capability                              | OpenTUI expectation                                                         | Fresco today                                                                                                                                                                                                                                                                                       | Status      |
| --------------------------------------- | --------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ----------- |
| Renderable lifecycle                    | `add`/`remove`/`destroyRecursively`, `onUpdate`/`onResize`/`onRemove` hooks | Vue vnode lifecycle drives a JS tree; the full node list is resent to Rust every frame; Rust `RenderTree` has dirty flags but no incremental protocol (M1).                                                                                                                                        | Partial     |
| Stable IDs and lookup                   | `getRenderable(id)`, `findDescendantById`                                   | Renderer-assigned numeric ids; no lookup or registry API (M1).                                                                                                                                                                                                                                     | Partial     |
| Input routing                           | Per-renderable keyboard/mouse events with bubbling                          | Global `last*Event` refs consumed by composables; no targeting, capture/bubble, or cancellation (M1).                                                                                                                                                                                              | Partial     |
| Mouse                                   | `onMouseDown`/`Over`/`Scroll`/`Drag`                                        | Native mouse capture surfaces raw events (`lastMouseEvent`); no hit-testing or per-element handlers (M1).                                                                                                                                                                                          | Partial     |
| Focus model                             | `focus`/`blur`, focus events, `hasFocusedDescendant`                        | Flat ordered registry, enable/disable, autoFocus, global Tab order; no traps, delegation, or descendant state (M1).                                                                                                                                                                                | Partial     |
| Scrolling and culling                   | ScrollBox with scroll-to/sticky edges and viewport culling                  | `overflow` (`visible`/`hidden`/`scroll`) maps to Taffy layout only; no viewport, scrollbars, or culling (M1).                                                                                                                                                                                      | Planned     |
| Clipping, z-index, translation, opacity | `visible`, `opacity`, `zIndex`, `translateX/Y`                              | Paint order is tree order; none of these controls exist yet (M1).                                                                                                                                                                                                                                  | Planned     |
| Custom drawing                          | `renderSelf`, `renderBefore`/`renderAfter`, frame buffers                   | Rust `NodeKind::Raw` line content only; no public typed extension API (M1).                                                                                                                                                                                                                        | Partial     |
| Diff and paint                          | double-buffered native painting                                             | Double-buffered cell diff, border painting, Unicode-aware width/wrap (grapheme, CJK, emoji) in Rust.                                                                                                                                                                                               | Implemented |
| Terminal control                        | raw mode, alternate screen                                                  | crossterm backend: raw mode, alternate screen, bracketed paste, mouse capture, cursor shape/visibility, truecolor detection.                                                                                                                                                                       | Implemented |
| Idle efficiency                         | no frame work when idle                                                     | JS event loop polls (~16 ms) and repaints at the frame interval even when nothing changed; Rust diff limits terminal writes (M1 target: dirty-subtree scheduling).                                                                                                                                 | Partial     |
| IME                                     | —                                                                           | Full pipeline: platform backends (macOS/Windows/Linux), preedit/candidates, composition events, `TextInput` integration.                                                                                                                                                                           | Implemented |
| Kitty keyboard                          | supported                                                                   | Flag/modifier helpers plus press/repeat/release event types.                                                                                                                                                                                                                                       | Implemented |
| Streams and stdio                       | —                                                                           | Custom `stdin`/`stdout`/`stderr`, console patching, external writes repaint cleanly, non-interactive/CI plain-text mode with `Static` semantics.                                                                                                                                                   | Implemented |
| Accessibility                           | —                                                                           | `aria-role`/`aria-state` props produce screen-reader text frames; `INK_SCREEN_READER`/`FRESCO_SCREEN_READER` env toggles; no semantic output contract or snapshots yet (M2).                                                                                                                       | Partial     |
| Testing                                 | —                                                                           | `@vizejs/fresco/testing` provides `renderTui`, `lastFrame`, frame/protocol snapshots, input injection, and type-contract fixtures; layout inspection remains planned.                                                                                                                              | Partial     |
| Devtools/diagnostics                    | —                                                                           | `debug` JSON tree dump, `onRender` timing, `getLastRenderLayouts`; no tree inspector or event trace (M4).                                                                                                                                                                                          | Partial     |
| Typed render protocol                   | —                                                                           | JS exports a closed four-variant `FrescoRenderNode` plus shared style, appearance, and event contracts; camel/snake aliases normalize to the canonical payload. Generated NAPI declaration parity remains (M0).                                                                                    | Partial     |
| SFC / Vite / HMR                        | — (Vue TermUI: SFC + dedicated Vite plugin with HMR)                        | SFC example builds with `@vizejs/vite-plugin`; dev runs re-execute via `vite-node`; no HMR pipeline (M4).                                                                                                                                                                                          | Partial     |
| Rust test/bench coverage                | —                                                                           | Unit tests across `input`/`layout`/`render`/`terminal`/`text`; Criterion `render` bench; JS Fresco has focused renderer/component/testing suites. The `fresco` workflow re-runs the JS check/build/test tasks and `cargo test -p vize_fresco` on Linux/macOS/Windows for every Fresco change (M0). | Partial     |

## Intentional differences

Per the product principles in #3113, Fresco keeps Ink's names where they help and drops React's
constraints:

- **Vue is the API.** State is Vue reactivity, not reconciler scheduling: `v-model` on inputs
  instead of controlled `value`/`onChange` pairs, typed emits instead of callback props, slots
  instead of render props, `provide`/`inject` instead of React context, and composables that are
  plain functions without hook-order rules.
- **No React reconciler semantics.** `concurrent` and `incrementalRendering` render options are
  accepted for Ink signature parity but stay no-ops: batching is Vue's scheduler plus the native
  diff, so there is nothing for those flags to switch. This is Not planned, not a gap.
- **Native pipeline.** Layout, diffing, and painting run in Rust (`vize_fresco`), not in
  JS Yoga + string diffing. JS owns component state and the render protocol only.
- **Template-first element names.** The renderer maps lowercase intrinsic tags (`box`, `div`,
  `view` → box; `text`, `span` → text; `input` → input), so SFC templates need no imports for
  primitives, while the exported components keep Ink-style PascalCase names rather than
  Vue TermUI's legacy `Tui*` prefixes or OpenTUI's `*Renderable` suffixes.
- **IME and Kitty input are first-class.** Composition events, preedit state, and key
  press/repeat/release reporting exist in the core event model rather than as add-ons.

## Updating this matrix

- Every #3113 child PR that adds, completes, or intentionally rejects a surface must update the
  matching row (and the audit commit above) in the same PR.
- Rows may only move to Implemented together with the behavior/type tests required by the issue's
  non-functional acceptance criteria.
- Performance-sensitive #3113 changes follow the
  [Fresco performance and size policy](./performance-policy.md).
