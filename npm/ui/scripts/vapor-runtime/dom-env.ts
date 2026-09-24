/**
 * happy-dom browser globals for the Vapor lane: `window`, `document`,
 * `navigator`, and every DOM constructor Node lacks. Event constructors are
 * replaced as well, because events built with Node's classes cannot be
 * dispatched on happy-dom targets.
 */
import { Window } from "happy-dom";

const overridden = ["Event", "CustomEvent", "EventTarget", "MessageEvent", "ErrorEvent"] as const;
const browserOnly = ["window", "self", "document", "navigator"] as const;

/** Every global installed by {@link installDomGlobals} that Node lacks. */
const installedGlobals = new Set<string>(browserOnly);

/** Install the globals and return the window. */
export function installDomGlobals(): Window {
  const window = new Window({
    url: "https://vapor-conformance.vize.test/",
    width: 1024,
    height: 768,
  });
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

/**
 * Run `render` with every installed browser global removed (window, document,
 * navigator, and DOM constructors such as `HTMLElement`), so it sees exactly
 * Node's globals, as a real server render does.
 */
export async function withoutBrowserGlobals<Result>(
  render: () => Promise<Result>,
): Promise<Result> {
  const saved = new Map<string, PropertyDescriptor | undefined>();
  for (const name of installedGlobals) {
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
