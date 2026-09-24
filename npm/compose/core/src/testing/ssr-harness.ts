/**
 * Server-rendering harness shared by the composable SSR tests.
 *
 * Test-only module: it is not part of any published entry. It renders a
 * component that calls the composable under test through Vue's real server
 * renderer, twice, while every host capability that Node exposes globally
 * (Web Storage, `fetch`, sockets, channels, `navigator`) is replaced by a
 * trap. A composable that touches one of them during server rendering throws
 * `[ssr-trap]`, so the tests prove capability detection stays gated behind a
 * browser `window` and that server output is deterministic.
 */

import assert from "node:assert/strict";

import { createSSRApp, defineComponent, h, isRef, unref } from "vue";
import { renderToString } from "vue/server-renderer";

/** Globals replaced by throwing traps while the harness renders. */
export const trappedServerGlobals = [
  "BroadcastChannel",
  "EventSource",
  "Notification",
  "WebSocket",
  "Worker",
  "fetch",
  "indexedDB",
  "localStorage",
  "navigator",
  "sessionStorage",
] as const;

/** Name of one trapped server global. */
export type TrappedServerGlobal = (typeof trappedServerGlobals)[number];

function trapError(name: string): Error {
  return new Error(`[ssr-trap] ${name} was used during server rendering`);
}

function createTrap(name: string): unknown {
  const target = function trapped(): never {
    throw trapError(name);
  };
  return new Proxy(target, {
    apply() {
      throw trapError(name);
    },
    construct() {
      throw trapError(name);
    },
    get() {
      throw trapError(name);
    },
    has() {
      throw trapError(name);
    },
  });
}

/**
 * Run `callback` while every {@link trappedServerGlobals} entry throws on use.
 *
 * `typeof window` and `typeof document` remain `"undefined"` exactly as on a
 * real Node server. Original property descriptors are restored afterwards.
 *
 * @param callback Work to run with the traps installed.
 * @returns The callback result.
 */
export async function withTrappedServerGlobals<Result>(
  callback: () => Promise<Result>,
): Promise<Result> {
  assert.equal(typeof window, "undefined", "SSR tests must run without a window");
  assert.equal(typeof document, "undefined", "SSR tests must run without a document");
  const saved = new Map<string, PropertyDescriptor | undefined>();
  for (const name of trappedServerGlobals) {
    saved.set(name, Object.getOwnPropertyDescriptor(globalThis, name));
    const trap = createTrap(name);
    Object.defineProperty(globalThis, name, {
      configurable: true,
      get: () => trap,
    });
  }
  try {
    return await callback();
  } finally {
    for (const [name, descriptor] of saved) {
      if (descriptor === undefined) Reflect.deleteProperty(globalThis, name);
      else Object.defineProperty(globalThis, name, descriptor);
    }
  }
}

function serializeState(state: Readonly<Record<string, unknown>>): string {
  const snapshot: Record<string, unknown> = {};
  for (const [key, value] of Object.entries(state)) {
    const plain: unknown = isRef(value) ? unref(value) : value;
    if (typeof plain === "function") continue;
    snapshot[key] = plain instanceof Map || plain instanceof Set ? [...plain] : plain;
  }
  return JSON.stringify(snapshot, (_key, value: unknown) =>
    typeof value === "bigint" ? `${value.toString()}n` : value,
  );
}

/**
 * Render a component whose setup calls a composable, twice, on the server.
 *
 * The setup callback returns the reactive state to serialize (refs are
 * unwrapped, functions skipped). Both renders run with the server-global
 * traps installed and must produce byte-identical HTML.
 *
 * @param setup Composable invocation performed inside component setup.
 * @returns The serialized state embedded in the rendered HTML.
 */
export async function renderComposableOnServer(
  setup: () => Readonly<Record<string, unknown>>,
): Promise<string> {
  const render = async (): Promise<string> => {
    const component = defineComponent({
      setup() {
        const state = setup();
        return () => h("output", { "data-state": serializeState(state) });
      },
    });
    return renderToString(createSSRApp(component));
  };
  return withTrappedServerGlobals(async () => {
    const first = await render();
    const second = await render();
    assert.equal(second, first, "server rendering must be deterministic");
    const match = /data-state="([^"]*)"/.exec(first);
    assert.ok(match?.[1] !== undefined, `unexpected server output: ${first}`);
    return match[1].replaceAll("&quot;", '"').replaceAll("&amp;", "&");
  });
}
