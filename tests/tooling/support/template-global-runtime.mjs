// Load complete compiler modules; only parsed module-owner literals are resolved.
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import {
  dataUrl,
  resolveModuleOwners,
  stableProvider,
  vaporProvider,
} from "./template-global-stock.mjs";

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
const providers = new Map();
let providerSequence = 0;
function providerUrl(provider) {
  if (providers.has(provider)) return providers.get(provider);
  const key = `__templateGlobalProvider${providerSequence++}`;
  globalThis[key] = provider;
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

export async function loadWholeComponent(packet, target) {
  const runtime = target === "vapor" ? vapor : target === "ssr" ? serverVue : stable;
  const owners = new Map([
    ["vue", providerUrl(runtime)],
    ["vue/server-renderer", providerUrl(renderer)],
  ]);
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
  return { component: module.default, runtime, resolvedCode };
}

function describeError(error, info) {
  return { name: error.name, message: error.message, stack: error.stack, info: info ?? null };
}

export async function executeWholeComponent(loaded, target, click) {
  const { component, runtime } = loaded;
  const errors = [];
  const warnings = [];
  let emitted = 0;
  const props = { onPress: () => emitted++ };
  const app =
    target === "ssr"
      ? runtime.createSSRApp(component, props)
      : target === "vapor"
        ? runtime.createVaporApp(component, props)
        : runtime.createApp(component, props);
  assert.equal(app._component, component, "actual app consumes the exact emitted default");
  assert.equal(app._component.setup, component.setup, "actual setup identity survives loading");
  app.config.errorHandler = (error, _instance, info) => errors.push(describeError(error, info));
  app.config.warnHandler = (message) => warnings.push(message);
  if (target === "ssr") {
    let html = null;
    try {
      html = await renderer.renderToString(app);
    } catch (error) {
      errors.push(describeError(error));
    }
    return { html, emitted, errors, warnings };
  }
  const host = document.body.appendChild(document.createElement("div"));
  let beforeClick = null;
  let html = null;
  let focused = false;
  try {
    app.mount(host);
    await runtime.nextTick();
    beforeClick = host.innerHTML;
    if (click) {
      const button = host.querySelector("button");
      assert.ok(button, "actual mounted button exists");
      button.focus();
      focused = document.activeElement === button;
      assert.equal(focused, true, "the real document owns the focused button");
      button.dispatchEvent(new MouseEvent("click", { bubbles: true }));
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
