// Execute the original generated module in a fresh real Vue Vapor process.
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { createRequire, registerHooks } from "node:module";
import { vueVaporBrowserRuntime, vueVaporVersion } from "./vue-vapor-release.mjs";

const chunks = [];
for await (const chunk of process.stdin) chunks.push(chunk);
const input = JSON.parse(Buffer.concat(chunks).toString("utf8"));
assert.equal(input.retainedLane, true);
assert.equal(typeof input.code, "string");
const fromUi = createRequire(new URL("../../../npm/ui/package.json", import.meta.url));
const { Window } = await import(fromUi.resolve("happy-dom"));
const window = new Window();
for (const name of [
  "window",
  "document",
  "Document",
  "Node",
  "Text",
  "Comment",
  "Element",
  "HTMLElement",
  "SVGElement",
  "Event",
  "ShadowRoot",
])
  globalThis[name] = name === "window" ? window : window[name];

const runtimeCode = readFileSync(vueVaporBrowserRuntime, "utf8");
const runtimeUrl = `data:text/javascript;base64,${Buffer.from(runtimeCode).toString("base64")}`;
registerHooks({
  resolve(specifier, context, nextResolve) {
    return specifier === "vue"
      ? { url: runtimeUrl, shortCircuit: true }
      : nextResolve(specifier, context);
  },
});
const runtimeInfo = [];
console.info = (...values) => runtimeInfo.push(values.join(" "));
const vue = await import(runtimeUrl);
for (const message of runtimeInfo)
  assert.ok(message.startsWith("You are running a development build of Vue."), message);
assert.equal(vueVaporVersion, "3.6.0-rc.9");
assert.equal(vue.version, vueVaporVersion);
const { render } = await import(
  `data:text/javascript;base64,${Buffer.from(input.code).toString("base64")}`
);
assert.equal(typeof render, "function");
const state = vue.reactive({ s: { backgroundColor: "blue" } });
const app = vue.createVaporApp(vue.defineVaporComponent({ setup: () => render(state) }));
const diagnostics = [];
app.config.warnHandler = (message) => diagnostics.push(message);
app.config.errorHandler = (error) => diagnostics.push(String(error));
const host = window.document.createElement("main");
window.document.body.append(host);
const colors = [];
const backgrounds = [];
let original;
try {
  app.mount(host);
  for (const [index, style] of [undefined, { color: "green" }, {}, null].entries()) {
    if (index > 0) state.s = style;
    await vue.nextTick();
    assert.deepEqual(diagnostics, [], "real Vue runtime diagnostics");
    const elements = host.querySelectorAll("div");
    assert.equal(elements.length, 1);
    const element = elements[0];
    if (index === 0) original = element;
    else assert.equal(element, original, "style update replaced the original element");
    assert.equal(element.textContent, "x");
    colors.push(element.style.color);
    backgrounds.push(element.style.backgroundColor);
  }
  assert.deepEqual(colors, ["red", "green", "red", "red"]);
  assert.deepEqual(backgrounds, ["blue", "", "", ""]);
  app.unmount();
  await vue.nextTick();
  assert.deepEqual(diagnostics, []);
  assert.equal(host.childNodes.length, 0);
  assert.equal(original.isConnected, false);
  console.log(
    JSON.stringify({
      runtime: vue.version,
      runtimeInfo,
      codeSha256: createHash("sha256").update(input.code).digest("hex"),
      colors,
      backgrounds,
      sameNode: true,
      unmounted: true,
    }),
  );
} finally {
  if (host.childNodes.length > 0) app.unmount();
  host.remove();
  await window.happyDOM.close();
}
