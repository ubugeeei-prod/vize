// Load complete compiler modules; only parsed module-owner literals are resolved.
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import {
  dataUrl,
  resolveModuleOwners,
  stableProvider,
  vaporProvider,
} from "./template-global-stock.ts";
import type { Target, WholePacket } from "./template-global-stock.ts";

export type RuntimeError = { name: string; message: string; stack?: string; info: string | null };
export type RuntimeResult = {
  html: string | null;
  emitted: number;
  errors: RuntimeError[];
  warnings: string[];
  beforeClick?: string | null;
  focused?: boolean;
  phase?: string;
};
type Component = { setup: unknown; render?: unknown };
type HostElement = HTMLElement;
type RuntimeApp = {
  _component: Component;
  config: {
    errorHandler?: (error: unknown, instance: unknown, info: string) => void;
    warnHandler?: (message: string) => void;
  };
  mount(host: HostElement): unknown;
  unmount(): void;
};
type RuntimeProvider = Record<string, unknown> & {
  version: string;
  createSSRApp(component: Component, props: unknown): RuntimeApp;
  createApp(component: Component, props: unknown): RuntimeApp;
  createVaporApp(component: Component, props: unknown): RuntimeApp;
  nextTick(): Promise<void>;
};
type LoadedComponent = { component: Component; runtime: RuntimeProvider; resolvedCode: string };
const dom = globalThis;

const stable = await import(
  dataUrl(readFileSync(stableProvider.resolve("vue/dist/vue.runtime.esm-browser.js"), "utf8"))
);
const vapor = await import(
  dataUrl(
    readFileSync(vaporProvider.resolve("vue/dist/vue.runtime-with-vapor.esm-browser.js"), "utf8"),
  )
);
const serverVue = stableProvider("vue");
const renderer = stableProvider("vue/server-renderer");
assert.equal(stable.version, "3.5.41");
assert.equal(serverVue.version, "3.5.41");
assert.equal(vapor.version, "3.6.0-rc.9");
assert.equal(stableProvider("@vue/server-renderer/package.json").version, "3.5.41");

// These provider modules forward the actual runtime exports without replacing helpers.
const providers = new Map<Record<string, unknown>, string>();
let providerSequence = 0;
function providerUrl(provider: Record<string, unknown>): string {
  if (providers.has(provider)) return providers.get(provider)!;
  const key = `__templateGlobalProvider${providerSequence++}`;
  Reflect.set(globalThis, key, provider);
  const names = Object.keys(provider).filter(
    (name) => /^[A-Za-z_$][\w$]*$/.test(name) && name !== "default",
  );
  const url = dataUrl(
    names
      .map(
        (name) =>
          `export const ${name}=globalThis[${JSON.stringify(key)}][${JSON.stringify(name)}];`,
      )
      .join("\n"),
  );
  providers.set(provider, url);
  return url;
}

export async function loadWholeComponent(
  packet: WholePacket,
  target: Target,
  authoredModules: Readonly<Record<string, string>> = {},
): Promise<LoadedComponent> {
  const runtime = target === "vapor" ? vapor : target === "ssr" ? serverVue : stable;
  const owners = new Map([
    ["vue", providerUrl(runtime)],
    ["vue/server-renderer", providerUrl(renderer)],
    ["@vue/server-renderer", providerUrl(renderer)],
  ]);
  for (const [id, code] of Object.entries(authoredModules)) {
    assert.ok(
      id.startsWith("./") && !owners.has(id),
      "authored module keeps its own relative owner",
    );
    owners.set(id, dataUrl(resolveModuleOwners(code, owners)));
  }
  if (packet.helperCode) owners.set("\0plugin-vue:export-helper", dataUrl(packet.helperCode));
  for (const virtual of packet.virtualModules ?? []) {
    owners.set(virtual.id, dataUrl(resolveModuleOwners(virtual.transformed.code, owners)));
  }
  const resolvedCode = resolveModuleOwners(packet.code, owners);
  const module = await import(dataUrl(resolvedCode));
  assert.equal(
    typeof module.default?.setup,
    "function",
    "the complete compiled default owns setup",
  );
  return {
    component: module.default as Component,
    runtime: runtime as RuntimeProvider,
    resolvedCode,
  };
}

function describeError(error: unknown, info?: string): RuntimeError {
  assert.ok(error instanceof Error);
  return { name: error.name, message: error.message, stack: error.stack, info: info ?? null };
}

export async function executeWholeComponent(
  loaded: LoadedComponent,
  target: Target,
  click: boolean,
): Promise<RuntimeResult> {
  const { component, runtime } = loaded;
  const errors: RuntimeError[] = [];
  const warnings: string[] = [];
  let emitted = 0;
  const onPress = () => emitted++;
  // Vapor raw props are getters; event listener values must be returned.
  const props = target === "vapor" ? { onPress: () => onPress } : { onPress };
  const app =
    target === "ssr"
      ? runtime.createSSRApp(component, props)
      : target === "vapor"
        ? runtime.createVaporApp(component, props)
        : runtime.createApp(component, props);
  // Vue createApp copies the options object; the emitted setup/render owners survive.
  assert.equal(app._component.render, component.render, "actual render identity survives loading");
  assert.equal(app._component.setup, component.setup, "actual setup identity survives loading");
  app.config.errorHandler = (error, _instance, info) => errors.push(describeError(error, info));
  app.config.warnHandler = (message) => warnings.push(message);
  if (target === "ssr") {
    let html: string | null = null;
    try {
      html = await renderer.renderToString(app);
    } catch (error) {
      errors.push(describeError(error));
    }
    return { html, emitted, errors, warnings };
  }
  const host = dom.document.body.appendChild(dom.document.createElement("div"));
  let beforeClick: string | null = null;
  let html: string | null = null;
  let focused = false;
  try {
    app.mount(host);
    await runtime.nextTick();
    beforeClick = host.innerHTML;
    if (click) {
      const button = host.querySelector("button");
      assert.ok(button, "actual mounted button exists");
      button.focus();
      focused = dom.document.activeElement === button;
      assert.equal(focused, true, "the real document owns the focused button");
      button.dispatchEvent(new dom.MouseEvent("click", { bubbles: true }));
      await runtime.nextTick();
    }
    html = host.innerHTML;
  } catch (error) {
    errors.push(describeError(error));
  } finally {
    app.unmount();
    await runtime.nextTick();
    host.remove();
  }
  return { beforeClick, html, focused, emitted, errors, warnings };
}
