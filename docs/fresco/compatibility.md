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

Migration and extension contracts:

- [Migrate from Ink](./migration-ink.md)
- [Migrate from Vue TermUI](./migration-vue-termui.md)
- [Stability and extension support](./support-policy.md)

## Audit basis

| Surface          | Audited source                                                                                            |
| ---------------- | --------------------------------------------------------------------------------------------------------- |
| Fresco Vue layer | `npm/fresco/src` (`index.ts`, `app.ts`, `renderer.ts`, `accessibility.ts`, `components/`, `composables/`) |
| Native bindings  | `npm/fresco-native/index.d.ts` (checked-in declarations; node kind mirrors its Rust annotation)           |
| Rust renderer    | `crates/vize_fresco/src` (`component/`, `input/`, `layout/`, `render/`, `terminal/`, `text/`, `napi/`)    |
| Ink              | `vadimdemedes/ink` README + v6 type declarations, retrieved 2026-07-19                                    |
| OpenTUI          | opentui.com core-concepts/renderables documentation, retrieved 2026-07-19                                 |
| Vue TermUI       | `posva/vue-termui` `main` branch exports (OpenTUI-based rewrite), retrieved 2026-07-19                    |

Re-audited from main commit `c6ccf94043` (2026-11-09). External surfaces (Ink, OpenTUI, Vue
TermUI) remain pinned by retrieval date (2026-07-19) rather than by an upstream revision, since none
publish per-release documentation snapshots; re-audit against those sources as of that date. `—`
means the surface has no equivalent.

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
| `<Newline>`   | —                | `Newline`  | `Newline`   | Implemented | Canonical source is a typed `.vue` SFC.                                                 |
| `<Spacer>`    | —                | —          | `Spacer`    | Implemented | Canonical source is a typed `.vue` SFC.                                                 |
| `<Static>`    | —                | —          | `Static`    | Implemented | Typed SFC with append-only output above the live area and non-interactive/CI support.   |
| `<Transform>` | —                | —          | `Transform` | Implemented | Canonical typed SFC; transforms `(children, index)` and supports `accessibilityLabel`.  |

### Interaction primitives

Ink ships these as ecosystem packages (`ink-text-input`, `ink-select-input`, ...); OpenTUI and
Vue TermUI ship them in core. `TextInput` owns its keyboard and editing behavior. `Select`, `List`,
`Menu`, `Tabs`, and `Slider` own focus-scoped keyboard input when `focused` is true. The remaining
partial interactive components still retain the gaps listed below (M2).

| Ink ecosystem      | OpenTUI               | Vue TermUI  | Fresco                                      | Status      | Notes                                                                                                                                                                                                             |
| ------------------ | --------------------- | ----------- | ------------------------------------------- | ----------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `ink-text-input`   | `InputRenderable`     | `Input`     | `TextInput`                                 | Partial     | Grapheme-safe selection, word editing, undo/redo, controlled or default state, validation, bracketed paste, IME, and copy/cut callbacks. System clipboard writes remain host-owned (M2).                          |
| —                  | `TextareaRenderable`  | `Textarea`  | `TextArea`                                  | Implemented | Controlled or default state, grapheme-safe multiline navigation and selection, history, word editing, clipboard callbacks, bracketed paste, IME, validation, viewport scrolling, and Ctrl+Enter submission.       |
| `ink-select-input` | `SelectRenderable`    | `Select`    | `Select`                                    | Partial     | Focus-scoped Up/Down, Home/End, disabled-item skipping, wrapping, typeahead, Enter selection, and `v-model`; generic non-string values and bounded option viewports remain (M2).                                  |
| —                  | `TabSelectRenderable` | `TabSelect` | `Tabs`                                      | Implemented | Focus-scoped Left/Right, Home/End, disabled-tab skipping, wrapping, typeahead, manual Enter/Space activation, and `v-model`.                                                                                      |
| —                  | —                     | —           | `Checkbox`, `RadioGroup`, `Confirm`, `Form` | Partial     | Focus-scoped activation and navigation are implemented. Form adds required/custom validation, first-error focus, typed slot state, and valid/invalid submission. A shared accessible-state contract remains (M2). |
| —                  | —                     | —           | `List`, `Menu`, `Tree`                      | Partial     | All three own disabled-aware navigation, boundaries, typeahead, and activation; Tree adds controlled expansion and parent/child movement. Generic values and Menu/Tree bounded viewports remain (M2).             |
| —                  | —                     | —           | `Modal`, `Tooltip` (typed SFC)              | Partial     | Modal owns a nested focus scope, trap/restore, initial focus, and Escape policy. Overlay stacking plus Tooltip/Popover anchor placement remain (M2).                                                              |
| —                  | `SliderRenderable`    | —           | `Slider`                                    | Implemented | Canonical typed SFC with range-safe `v-model`, Arrow/Page/Home/End input, wheel/click/drag input, horizontal/vertical tracks, and accessible value state.                                                         |

### Display and rich components

| Ink ecosystem      | OpenTUI                 | Vue TermUI    | Fresco                      | Status      | Notes                                                                                                                                                                                                             |
| ------------------ | ----------------------- | ------------- | --------------------------- | ----------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `ink-spinner`      | —                       | —             | `Spinner`                   | Implemented | Canonical typed SFC with owned animation lifecycle.                                                                                                                                                               |
| `ink-progress-bar` | —                       | `ProgressBar` | `ProgressBar`               | Implemented |                                                                                                                                                                                                                   |
| `ink-table`        | —                       | —             | `Table`                     | Implemented | Typed rows/columns and cell slots, grapheme-safe alignment, controlled stable sorting, keyboard column resize, and bounded row virtualization.                                                                    |
| `ink-link`         | —                       | —             | `Link`                      | Partial     | Typed SFC with styled label and optional URL suffix; no OSC 8 hyperlink escape.                                                                                                                                   |
| —                  | `CodeRenderable`        | —             | [`Code`](./code.md)         | Implemented | Bounded streaming buffer, grapheme-safe controlled selection, diagnostics, synchronous syntax providers, line numbers, and large-file viewport culling.                                                           |
| —                  | `MarkdownRenderable`    | `Markdown`    | [`Markdown`](./markdown.md) | Implemented | Typed SFC backed by a lightweight CommonMark block/inline core, per-block providers, and a complete renderer replacement hook.                                                                                    |
| —                  | `LineNumberRenderable`  | —             | `LineNumber`                | Implemented | Standalone, aligned gutter primitive with configurable separator and active-line styling; `Code` composes the same public component.                                                                              |
| —                  | `DiffRenderable`        | —             | `Diff`                      | Implemented | Unified/split views, contextual hunks, line numbers, word-level highlights, and bounded viewport rendering for large files.                                                                                       |
| —                  | `ASCIIFontRenderable`   | —             | `AsciiFont`                 | Implemented | Provider-driven `@vizejs/fresco/ascii-font` entry; consumer glyph data stays outside the core bundle.                                                                                                             |
| —                  | `QRCodeRenderable`      | —             | `QrCode`                    | Implemented | Provider-driven `@vizejs/fresco/qr-code` entry; consumer QR encoders stay outside the core bundle.                                                                                                                |
| —                  | `ScrollBoxRenderable`   | `ScrollBox`   | `ScrollBox`                 | Partial     | Virtualized Vue facade with keyboard/wheel scrolling, sticky slots, imperative scrolling, bounded row retention, native ancestor clipping, and routed pointer targeting. A native scroll renderable remains (M1). |
| —                  | `ScrollBarRenderable`   | —             | `ScrollBar`                 | Partial     | Typed vertical/horizontal viewport indicator with bounded track clicks and thumb dragging; native renderable integration remains (M1).                                                                            |
| —                  | `FrameBufferRenderable` | —             | `FrameBuffer`               | Implemented | Public typed SFC accepts exact terminal rows and emits the native raw-node protocol without private renderer imports.                                                                                             |

### Vize-native components (no direct reference equivalent)

All are Implemented as display components and follow the same M2/M3 interaction and theming
polish track as the tables above.

| Group      | Components                                                            |
| ---------- | --------------------------------------------------------------------- |
| Layout     | `Card`, `Stack`/`HStack`/`VStack`, and `Divider` (typed SFCs); `Grid` |
| Feedback   | `Alert`, `Badge`, and `Avatar` (typed SFCs); `Timer`                  |
| Navigation | `Breadcrumb` (typed SFC), `Stepper` (`Menu`, `Tabs` above)            |
| Chrome     | `Header` (typed SFC), `StatusBar`, `KeyHint`                          |

## Composables

Fresco also exposes a typed theme contract with frozen zero-generation defaults. Nested
`provideFrescoTheme()` calls inherit the nearest resolved theme and reactively override only the
supplied color, spacing, border, or motion tokens; `useFrescoTheme()` consumes the resolved scope.

Fresco implements the full Ink v6 hook list under the same names, as Vue composables.

Configurable command input is available through [`useKeymap`](./keymaps.md), including exact
single chords, Emacs-style multi-chords, Vim-style sequences, reactive mode maps, and typed command
IDs.

| Ink                                | OpenTUI                           | Vue TermUI                   | Fresco                             | Status      | Notes                                                                                                                                                                                  |
| ---------------------------------- | --------------------------------- | ---------------------------- | ---------------------------------- | ----------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `useInput`                         | `onKeyDown` (per renderable)      | `onKeyDown`/`onKeyUp`        | `useInput` (+ `useKeyPress`)       | Partial     | Global keyboard subscription with Kitty press/repeat/release. Host nodes expose capture/target/bubble dispatch, but automatic keyboard targeting remains (M1).                         |
| —                                  | `onMouse`                         | —                            | `useMouse`                         | Partial     | Normalized down/up/drag/move/wheel actions and modifiers. The app hit-tests native layouts and routes pointer input through capture/target/bubble; enter/leave synthesis remains (M1). |
| —                                  | scroll viewport state             | —                            | `useVirtualViewport`               | Implemented | Bounded visible/overscan ranges plus scroll-to/by/into-view alignment; work scales with retained rows rather than collection size.                                                     |
| `usePaste`                         | `onPaste`                         | —                            | `usePaste`                         | Implemented | Bracketed paste.                                                                                                                                                                       |
| `useApp`                           | renderer lifecycle                | `useExit`                    | `useApp`                           | Partial     | `exit`, terminal `width`/`height` refs, `render`, `clear`; `waitUntilRenderFlush` waits for a pending mount and Vue updates, then commits the dirty frame synchronously.               |
| `useStdin`/`useStdout`/`useStderr` | —                                 | —                            | `useStdin`/`useStdout`/`useStderr` | Implemented | Backed by the app streams context; honors custom streams.                                                                                                                              |
| `useFocus`                         | `focus()`/`blur()` + focus events | `useCurrentFocusedElement`   | `useFocus`                         | Implemented | Ordered registry with reactive activation, safe descendant delegation, nested focus traps, descendant-focus state, and deterministic Tab/Shift+Tab traversal.                          |
| `useFocusManager`                  | renderer focus APIs               | `useFocusManager`            | `useFocusManager`                  | Implemented | `focus`, `focusNext`/`focusPrevious`, `hasFocusedDescendant`, `trapFocus`/`releaseFocusTrap`, `setDelegate`, enable/disable, `activeId`, `focusableIds`, `activeTrapId`.               |
| `useWindowSize`                    | `onResize` hook                   | `useTerminalSize`/`onResize` | `useWindowSize`                    | Implemented |                                                                                                                                                                                        |
| `useCursor`                        | cursor ownership                  | —                            | `useCursor`                        | Implemented | Position override + visibility via native cursor control.                                                                                                                              |
| `useAnimation`                     | `onUpdate(deltaTime)`             | `useInterval`/`useTimeout`   | `useAnimation`                     | Implemented | Shared timer; `frame`/`time`/`delta`/`reset`.                                                                                                                                          |
| `useBoxMetrics`                    | —                                 | —                            | `useBoxMetrics`                    | Implemented | Reads last-render layout results.                                                                                                                                                      |
| `useIsScreenReaderEnabled`         | —                                 | —                            | `useIsScreenReaderEnabled`         | Implemented |                                                                                                                                                                                        |
| —                                  | —                                 | —                            | `useIme`                           | Implemented | Fresco-specific: IME mode, preedit, and candidate state.                                                                                                                               |

## Top-level API and render options

| Ink                                                             | Fresco                                        | Status      | Notes                                                                                                                                           |
| --------------------------------------------------------------- | --------------------------------------------- | ----------- | ----------------------------------------------------------------------------------------------------------------------------------------------- |
| `render(tree, options?)` → instance                             | `render(root, options?)` → instance           | Implemented | Also accepts a bare writable stream as `options`, like Ink.                                                                                     |
| instance `rerender`/`unmount`/`waitUntilExit`/`cleanup`/`clear` | same                                          | Implemented |                                                                                                                                                 |
| instance `waitUntilRenderFlush`                                 | `waitUntilRenderFlush`                        | Implemented | Waits for pending Vue updates, then commits the dirty native or plain-text frame synchronously.                                                 |
| `renderToString`                                                | `renderToString`                              | Partial     | Plain-text projection (no ANSI styling, no flex layout beyond column stacking); retained as a low-level snapshot helper.                        |
| `measureElement`                                                | `measureElement`                              | Implemented | Backed by native last-render layouts.                                                                                                           |
| —                                                               | `createApp(root, options?)` + `mount`         | Implemented | Vue-native entry point underneath the Ink-compatible `render`.                                                                                  |
| —                                                               | [`App.use`, plugins, providers](./plugins.md) | Implemented | Mount-before-use enforcement, declarative global SFC registration, typed injection keys and fallbacks, plus an external component fixture.      |
| Testing: `ink-testing-library`                                  | [`@vizejs/fresco/testing`](./testing.md)      | Implemented | Vitest and plain Node harness with semantic queries, `renderTui`, frame/protocol snapshots, input injection, and public tree/layout inspection. |

Render options: `stdout`, `stdin`, `stderr`, `debug`, `exitOnCtrlC`, `patchConsole`, `onRender`,
`isScreenReaderEnabled`, `maxFps`, `interactive`, `alternateScreen`, and `kittyKeyboard` are
implemented with Ink semantics. `incrementalRendering` and `concurrent` are accepted for API
parity but are intentional no-ops (see "Intentional differences"). Fresco adds `mouse` and
`onError`.

## Runtime and renderer capabilities

| Capability                              | OpenTUI expectation                                                         | Fresco today                                                                                                                                                                                                                                                                                                                         | Status      |
| --------------------------------------- | --------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ | ----------- |
| Renderable lifecycle                    | `add`/`remove`/`destroyRecursively`, `onUpdate`/`onResize`/`onRemove` hooks | Vue vnode lifecycle owns a stable JS registry with idempotent recursive disposal and create/update/declared-resize/remove observers. The full node list is still resent to Rust every dirty frame; a native incremental protocol remains (M1).                                                                                       | Partial     |
| Stable IDs and lookup                   | `getRenderable(id)`, `findDescendantById`                                   | Renderer-assigned process-unique ids survive keyed moves; `getNode` and subtree-bounded `findDescendantById` expose read-only host views. Native-side lookup remains an M1 target.                                                                                                                                                   | Partial     |
| Input routing                           | Per-renderable keyboard/mouse events with bubbling                          | Stable host nodes expose typed `dispatchInput` routing with capture/target/bubble phases, generic `onFrescoInput` and event-specific handlers, default prevention, and propagation cancellation. Mouse packets are targeted automatically from the matching native layout snapshot; automatic keyboard/focus targeting remains (M1). | Partial     |
| Mouse                                   | `onMouseDown`/`Over`/`Scroll`/`Drag`                                        | Native capture preserves action kind and modifiers through the JS protocol. The app selects the deepest topmost host node inside ancestor overflow clips and routes every mouse action; `useMouse` remains available globally. Enter/leave synthesis remains (M1).                                                                   | Partial     |
| Focus model                             | `focus`/`blur`, focus events, `hasFocusedDescendant`                        | Ordered registry, enable/disable, autoFocus, parent IDs, descendant state, safe delegation, nested focus traps, and deterministic scoped Tab order. Native focus-event targeting remains tied to the input-routing and hit-testing slices (M1).                                                                                      | Partial     |
| Scrolling and culling                   | ScrollBox with scroll-to/sticky edges and viewport culling                  | `ScrollBox`/`ScrollBar` provide scroll-to/by/into-view, keyboard/wheel control, sticky slots, tail following, viewport+overscan row retention, and matching native paint/input clips. A native translated scroll viewport remains (M1).                                                                                              | Partial     |
| Clipping, z-index, translation, opacity | `visible`, `opacity`, `zIndex`, `translateX/Y`                              | Hidden/scroll overflow clips descendant paint at absolute terminal coordinates, including complete wide graphemes, and constrains hit-testing. Stable sibling `zIndex` orders both paint and pointer targets; translation and opacity remain (M1).                                                                                   | Partial     |
| Custom drawing                          | `renderSelf`, `renderBefore`/`renderAfter`, frame buffers                   | Public `FrescoRawRenderNode` and typed `FrameBuffer` SFC carry exact rows through Rust `NodeKind::Raw`; per-cell drawing and before/after hooks remain (M1).                                                                                                                                                                         | Partial     |
| Diff and paint                          | double-buffered native painting                                             | Double-buffered cell diff, border painting, Unicode-aware width/wrap (grapheme, CJK, emoji) in Rust.                                                                                                                                                                                                                                 | Implemented |
| Terminal control                        | raw mode, alternate screen                                                  | crossterm backend: raw mode, alternate screen, bracketed paste, mouse capture, cursor shape/visibility, truecolor detection.                                                                                                                                                                                                         | Implemented |
| Idle efficiency                         | no frame work when idle                                                     | Host-tree mutations mark the next native frame dirty; an unchanged Vue tree does no layout, paint, or terminal flush work. Input polling remains active while the application is interactive; dirty-subtree rendering remains an M1 target.                                                                                          | Partial     |
| IME                                     | —                                                                           | Full pipeline: platform backends (macOS/Windows/Linux), preedit/candidates, composition events, `TextInput` integration.                                                                                                                                                                                                             | Implemented |
| Kitty keyboard                          | supported                                                                   | Flag/modifier helpers plus press/repeat/release event types.                                                                                                                                                                                                                                                                         | Implemented |
| Streams and stdio                       | —                                                                           | Custom `stdin`/`stdout`/`stderr`, console patching, external writes repaint cleanly, non-interactive/CI plain-text mode with `Static` semantics.                                                                                                                                                                                     | Implemented |
| Accessibility                           | —                                                                           | `aria-role`/`aria-state` props produce screen-reader text frames; `INK_SCREEN_READER`/`FRESCO_SCREEN_READER` env toggles; no semantic output contract or snapshots yet (M2).                                                                                                                                                         | Partial     |
| Testing                                 | —                                                                           | [`@vizejs/fresco/testing`](./testing.md) provides Vitest/plain Node semantic queries, `renderTui`, frame/protocol snapshots, input injection, public tree/layout inspection, and type-contract fixtures.                                                                                                                             | Implemented |
| Devtools/diagnostics                    | —                                                                           | [`createFrescoDevtools`](./devtools.md) joins stable host nodes to native layouts, records bounded lifecycle/input/render traces and metrics, and formats an optional terminal debug overlay without scheduling idle work.                                                                                                           | Implemented |
| Typed render protocol                   | —                                                                           | JS exports a closed five-variant `FrescoRenderNode` plus shared style, appearance, and event contracts; camel/snake aliases normalize to the canonical payload. Generated NAPI declarations include the raw kind and exact row payload.                                                                                              | Implemented |
| SFC / Vite / HMR                        | — (Vue TermUI: SFC + dedicated Vite plugin with HMR)                        | `@vizejs/create-fresco` generates strict SFC/Vite projects with watch-mode reloads, typecheck/build commands, safe overwrite policy, and executable delivery guidance.                                                                                                                                                               | Implemented |
| Rust test/bench coverage                | —                                                                           | Unit tests across `input`/`layout`/`render`/`terminal`/`text`; Criterion `render` bench; JS Fresco has focused renderer/component/testing suites. The `fresco` workflow re-runs the JS check/build/test tasks and `cargo test -p vize_fresco` on Linux/macOS/Windows for every Fresco change (M0).                                   | Partial     |

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
