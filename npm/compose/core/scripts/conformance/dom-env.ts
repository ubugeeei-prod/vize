/**
 * Browser globals backed by happy-dom for the conformance lanes.
 *
 * Installs `window`, `document`, `navigator`, and every DOM constructor or
 * window function Node lacks. Globals Node already defines (for example
 * `fetch` or `URL`) keep their Node implementation except `navigator`,
 * which is replaced so capability detection sees a browser navigator.
 * `hideBrowserGlobals` temporarily removes the browser-only globals so a
 * server render in the same process sees a real server environment.
 */
import { Window } from "happy-dom";

/**
 * Node globals replaced by their happy-dom counterparts: events created with
 * Node's constructors cannot be dispatched on happy-dom targets.
 */
const overridden = ["Event", "CustomEvent", "EventTarget", "MessageEvent", "ErrorEvent"] as const;

/** Globals removed while server rendering. */
const browserOnly = ["window", "document", "navigator", "self"] as const;

/**
 * Every global added by {@link installDomGlobals} that Node itself lacks
 * (for example `ResizeObserver` or `HTMLElement`); hidden together with
 * {@link browserOnly} so server renders see exactly Node's globals.
 */
const installedGlobals = new Set<string>();

/** Install the happy-dom globals; returns the window for teardown. */
export function installDomGlobals(): Window {
  const window = new Window({ url: "https://conformance.vize.test/", width: 1024, height: 768 });
  const target: Record<string, unknown> = globalThis as unknown as Record<string, unknown>;
  for (const name of Object.getOwnPropertyNames(window)) {
    if (name in globalThis) continue;
    installedGlobals.add(name);
    const value: unknown = Reflect.get(window, name);
    target[name] = typeof value === "function" && !/^[A-Z]/.test(name) ? value.bind(window) : value;
  }
  for (const name of overridden) target[name] = Reflect.get(window, name);
  for (const name of browserOnly) {
    Object.defineProperty(globalThis, name, {
      configurable: true,
      writable: true,
      value: name === "window" || name === "self" ? window : Reflect.get(window, name),
    });
  }
  return window;
}

/** Run `render` with the browser-only globals removed, restoring them after. */
export async function withoutBrowserGlobals<Result>(
  render: () => Promise<Result>,
): Promise<Result> {
  const saved = new Map<string, PropertyDescriptor | undefined>();
  for (const name of new Set<string>([...browserOnly, ...installedGlobals])) {
    saved.set(name, Object.getOwnPropertyDescriptor(globalThis, name));
    Reflect.deleteProperty(globalThis, name);
  }
  try {
    return await render();
  } finally {
    for (const [name, descriptor] of saved) {
      if (descriptor) Object.defineProperty(globalThis, name, descriptor);
    }
  }
}

/** Collect `console.warn` / `console.error` output while `run` executes. */
export async function captureConsole<Result>(
  run: () => Promise<Result>,
): Promise<{ readonly result: Result; readonly messages: readonly string[] }> {
  const messages: string[] = [];
  const originalWarn = console.warn;
  const originalError = console.error;
  const record = (...values: unknown[]): void => {
    messages.push(
      values
        .map((value) => (value instanceof Error ? (value.stack ?? value.message) : String(value)))
        .join(" "),
    );
  };
  console.warn = record;
  console.error = record;
  try {
    return { result: await run(), messages };
  } finally {
    console.warn = originalWarn;
    console.error = originalError;
  }
}

/** Let effects, post-flush jobs, and short timers settle. */
export async function settle(): Promise<void> {
  for (let index = 0; index < 5; index += 1) await Promise.resolve();
  await new Promise((resolve) => setTimeout(resolve, 20));
  for (let index = 0; index < 5; index += 1) await Promise.resolve();
}
