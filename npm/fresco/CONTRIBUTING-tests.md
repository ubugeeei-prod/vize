# Fresco JS test baseline

`pnpm test` runs `node:test` suites (via `vp exec tsx --test 'src/**/*.test.ts'`).
`pnpm check` includes `tsgo --noEmit`, which gates the compile-only type tests.

## The three-part standard

Every suite in `npm/fresco/src` follows three rules (see #3228):

1. **Render-snapshot tests assert frames and mounted output trees.** Mount
   through `renderTui` from `@vizejs/fresco/testing`, then compare
   `lastFrame()`/`captureFrame()` output, serializable `FrescoNode` trees, or
   stable native protocol snapshots against inline expected structures with
   `assert.deepEqual`. Never assert on source text, and avoid giant snapshots:
   keep expected frames and trees small enough to read inline.
2. **Type-level tests cover public props/emits.** Compile-only `*.test-d.ts`
   files (see `src/components/types.test-d.ts`) pair positive cases with
   `@ts-expect-error` negatives; `tsgo` fails when either direction regresses.
3. **Behavior tests cover keyboard/focus ownership.** Drive input through
   `renderTui(...).input` or the lower-level `dispatchKey`/`typeChars` helpers
   (the same `last*Event` refs the interactive event loop writes) and assert
   focus against the `FocusManager` contract.

## The native seam

There is no mock of `@vizejs/fresco-native`. The JS layer is pure up to the
byte boundary that crosses into Rust: `renderer.ts` imports only types from the
native package, and `app.ts` loads it lazily and only in interactive mode.
Tests therefore run the shipped code end-to-end:

- `renderTui` (exported as `@vizejs/fresco/testing`) records plain frame
  snapshots, serializable tree snapshots, stable native protocol payloads, and
  input-injected frame history.
- `mountFresco` (in `src/testing/mount.ts`) mounts components with the same
  provide set `createApp` installs (app context, focus manager, screen reader
  flag, streams, cursor) and exposes the live mounted tree.
- `frameSnapshot().protocolNodes` returns the stable flat payload the native
  `renderTree` call would paint; assert on it for style/appearance mapping.
- `renderToString(root)` covers the plain-text output path.

If a future test needs a runtime native API, add a typed seam instead of
importing the binding: keep the fake at the narrowest boundary possible.

## Adding a component test

1. Create `src/components/YourComponent.test.ts` (colocated, `node:test`).
2. Mount with `renderTui(() => h(YourComponent, props, slot), options?)` when
   asserting frames or input-driven behavior. Use `mountComponent` only for
   targeted low-level tree assertions.
3. Assert the tree with `toTreeSnapshot(firstChild(mounted))` against an
   inline expected object, or read targeted props off `firstChild(mounted)`.
   Note: Boolean-declared props default to `false` and appear on emitted host
   nodes; assert them explicitly.
4. For interaction, pass `focused: true` (or use `useFocus` ids) and drive
   `await rendered.input.key({ key: "enter" })` /
   `await rendered.input.text("abc")`; reactive updates outside the input
   driver need `await nextTick()` before `captureFrame()`.
5. Always `mounted.unmount()` so composable lifecycles (focus unregister,
   watcher disposal) are exercised.
6. Extend `src/components/types.test-d.ts` with the component's public
   props/emits, including at least one `@ts-expect-error` negative per prop
   group.
