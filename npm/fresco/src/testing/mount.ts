/**
 * Test-only mount harness for the Fresco JS layer.
 *
 * Fresco's renderer is pure JS up to the byte boundary that crosses into the
 * native module: `renderer.ts` imports only types from `@vizejs/fresco-native`,
 * and `app.ts` loads the native binding lazily and only in interactive mode.
 * This harness mounts components through the real custom renderer with the
 * same provide set `createApp`/`renderToString` install, so tests assert the
 * mounted `FrescoNode` output tree without a native build and without mocks.
 *
 * Not exported from the package entry points; `vp pack` does not ship it.
 */

import {
  h,
  ref,
  nextTick,
  type App as VueApp,
  type Component,
  type Ref,
  type VNodeChild,
} from "@vue/runtime-core";
import { SCREEN_READER_KEY } from "../accessibility.js";
import { lastKeyEvent, type KeyEvent } from "../app.js";
import { APP_KEY, createAppContext } from "../composables/useApp.js";
import { createCursorContext, CURSOR_KEY } from "../composables/useCursor.js";
import { createFocusManager, FOCUS_KEY, type FocusManager } from "../composables/useFocus.js";
import { createStreamsContext, STREAMS_KEY } from "../composables/useStreams.js";
import { createRenderer, type FrescoElement, type FrescoNode } from "../renderer.js";

/** Options for {@link mountFresco}. */
export interface MountFrescoOptions {
  /** Terminal width provided through the app context. Defaults to 80. */
  width?: number;
  /** Terminal height provided through the app context. Defaults to 24. */
  height?: number;
  /** Initial screen reader mode. Defaults to false. */
  screenReader?: boolean;
}

/** A mounted Fresco component tree under test. */
export interface MountedFresco {
  /** Root element the Vue app mounted into; children form the output tree. */
  root: FrescoElement;
  /** The Vue app instance (for `provide`-level introspection if needed). */
  app: VueApp<FrescoElement>;
  /** Focus manager provided to the tree, as the real app would. */
  focusManager: FocusManager;
  /** Screen reader flag provided to the tree; writable to flip modes. */
  screenReaderEnabled: Ref<boolean>;
  /** Unmount the tree and dispose all component watchers. */
  unmount(): void;
}

/**
 * Mount a component (or a plain render function) through the Fresco renderer.
 *
 * The returned {@link MountedFresco.root} is live: re-read it after reactive
 * updates plus `await nextTick()` to observe the patched output tree.
 */
export function mountFresco(
  root: Component | (() => VNodeChild),
  options: MountFrescoOptions = {},
): MountedFresco {
  const { createApp } = createRenderer();
  const component: Component = typeof root === "function" ? { setup: () => root } : root;
  const app = createApp(component);

  const focusManager = createFocusManager();
  const screenReaderEnabled = ref(options.screenReader ?? false);
  const noopWrite = () => {};
  const width = options.width ?? 80;
  const height = options.height ?? 24;
  // A detached stdout stand-in keeps createAppContext from attaching resize
  // listeners to the real process.stdout on every mounted test tree.
  const stdout = {
    isTTY: false,
    columns: width,
    rows: height,
    write: () => true,
  } as unknown as NodeJS.WriteStream;

  app.provide(APP_KEY, createAppContext({ width, height, stdout }));
  app.provide(FOCUS_KEY, focusManager);
  app.provide(SCREEN_READER_KEY, screenReaderEnabled);
  app.provide(CURSOR_KEY, createCursorContext(noopWrite));
  app.provide(
    STREAMS_KEY,
    createStreamsContext({
      interactive: false,
      writeToStdout: noopWrite,
      writeToStderr: noopWrite,
    }),
  );

  const rootElement: FrescoElement = {
    id: -1,
    type: "root",
    props: {},
    children: [],
    parent: null,
  };
  app.mount(rootElement);

  return {
    root: rootElement,
    app,
    focusManager,
    screenReaderEnabled,
    unmount: () => app.unmount(),
  };
}

/**
 * Dispatch a key event through the same `lastKeyEvent` ref the interactive
 * event loop writes, then wait for watchers to flush.
 *
 * Pass `char` for printable input or `key` for named keys ("left", "enter",
 * "backspace", ...). Modifiers default to false.
 */
export function dispatchKey(event: Partial<Omit<KeyEvent, "type">>): Promise<void> {
  lastKeyEvent.value = {
    type: "key",
    ctrl: false,
    alt: false,
    shift: false,
    meta: false,
    super: false,
    hyper: false,
    capsLock: false,
    numLock: false,
    ...event,
  };
  return nextTick();
}

/** Dispatch a sequence of printable characters via {@link dispatchKey}. */
export async function typeChars(text: string): Promise<void> {
  for (const char of text) {
    await dispatchKey({ char });
  }
}

/**
 * Serializable snapshot of a mounted output tree.
 *
 * Node ids come from a module-global counter and depend on mount order, and
 * `parent` back-references make trees cyclic, so both are omitted. Props are
 * kept only when defined, so snapshots stay inline-sized and deterministic.
 */
export interface FrescoNodeSnapshot {
  type: FrescoNode["type"];
  text?: string;
  props?: Record<string, unknown>;
  children?: FrescoNodeSnapshot[];
}

function definedProps(props: Record<string, unknown>): Record<string, unknown> | undefined {
  const entries = Object.entries(props).filter(([, value]) => value !== undefined);
  if (entries.length === 0) return undefined;
  return Object.fromEntries(entries);
}

/**
 * Convert a mounted node (or the harness root) into a plain snapshot object
 * suitable for `assert.deepEqual` against an inline expected structure.
 */
export function toTreeSnapshot(node: FrescoNode): FrescoNodeSnapshot {
  const snapshot: FrescoNodeSnapshot = { type: node.type };
  if (node.text !== undefined) snapshot.text = node.text;
  const props = definedProps(node.props);
  if (props) snapshot.props = props;
  if (node.children.length > 0) {
    snapshot.children = node.children.map((child) => toTreeSnapshot(child));
  }
  return snapshot;
}

/** Depth-first list of nodes matching a predicate, starting at `node`. */
export function findNodes(
  node: FrescoNode,
  predicate: (candidate: FrescoNode) => boolean,
): FrescoNode[] {
  const matches: FrescoNode[] = [];
  const visit = (current: FrescoNode) => {
    if (predicate(current)) matches.push(current);
    for (const child of current.children) visit(child);
  };
  visit(node);
  return matches;
}

/** First mounted child under the harness root, asserting it exists. */
export function firstChild(mounted: MountedFresco): FrescoNode {
  const child = mounted.root.children[0];
  if (!child) throw new Error("expected the mounted tree to have a root child");
  return child;
}

/** Convenience wrapper that renders `component` with `h` and mounts it. */
export function mountComponent(
  component: Component,
  props: Record<string, unknown> = {},
  slot?: () => VNodeChild,
  options: MountFrescoOptions = {},
): MountedFresco {
  return mountFresco(() => h(component, props, slot), options);
}
