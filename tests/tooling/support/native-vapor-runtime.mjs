// Runs complete emitted and official modules against the actual pinned browser runtime.
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { createRequire } from "node:module";
import { vueVaporBrowserRuntime, vueVaporVersion } from "./vue-vapor-release.mjs";
const requireUi = createRequire(new URL("../../../npm/ui/package.json", import.meta.url));
const { Window } = await import(requireUi.resolve("happy-dom"));
const window = new Window();
for (const name of ["window", "document", "Document", "Node", "Text", "Comment", "Element", "HTMLElement", "SVGElement", "Event", "ShadowRoot"])
  globalThis[name] = name === "window" ? window : window[name];
const dataUrl = (code) => `data:text/javascript;base64,${Buffer.from(code).toString("base64")}`;
console.info = (...values) => process.stderr.write(values.join(" ") + "\n");
const vue = await import(dataUrl(readFileSync(vueVaporBrowserRuntime, "utf8")));
assert.equal(vue.version, vueVaporVersion);
for (const name of ["template", "defineVaporComponent", "createVaporApp", "createVaporSSRApp"])
  assert.equal(typeof vue[name], "function", `actual export ${name}`);
globalThis.__nativeVaporRuntime = vue;

let sequence = 0;
async function load(code) {
  const rewritten = code.replace(/import \{([^}]+)\} from ["']vue["'];?/g, (_, list) =>
    list.split(",").map((entry) => {
      const [name, local = name] = entry.trim().split(/\s+as\s+/u);
      assert.equal(typeof vue[name], "function", `module import ${name}`);
      return `const ${local} = globalThis.__nativeVaporRuntime[${JSON.stringify(name)}];`;
    }).join("\n"));
  assert.ok(!/\bimport\s/u.test(rewritten), "unresolved module import");
  return (await import(dataUrl(rewritten + `\n// complete module ${sequence++}`))).render;
}

function observe(node) {
  if (node.nodeType === 3 || node.nodeType === 8) return [node.nodeType, node.data];
  assert.equal(node.nodeType, 1);
  return [1, node.localName, node.namespaceURI,
    [...node.attributes].map((attribute) => [attribute.name, attribute.value]).sort(),
    [...node.childNodes].map(observe)];
}

async function mount(render, configuration, serverHtml = null) {
    const host = document.body.appendChild(document.createElement("div"));
    if (serverHtml !== null) host.innerHTML = serverHtml;
  const component = vue.defineVaporComponent({
    inheritAttrs: configuration.inheritAttrs,
    setup: () => render({}),
  });
  const app = (serverHtml === null ? vue.createVaporApp : vue.createVaporSSRApp)(component, configuration.props);
  const diagnostics = [];
  app.config.warnHandler = (message) => diagnostics.push(message);
  app.config.errorHandler = (error) => diagnostics.push(String(error));
  try {
    app.mount(host);
    await vue.nextTick();
    assert.deepEqual(diagnostics, [], "mounted diagnostics");
    const tree = [...host.childNodes].map(observe);
    const html = host.innerHTML;
    const retained = [...host.childNodes];
    app.unmount();
    await vue.nextTick();
    assert.equal(host.childNodes.length, 0, "unmount leaves no nodes");
    assert.deepEqual(diagnostics, []);
    return { tree, retained, html };
  } finally { host.remove(); }
}

const chunks = [];
for await (const chunk of process.stdin) chunks.push(chunk);
const inputs = JSON.parse(Buffer.concat(chunks).toString("utf8"));
const captured = [];
try {
  assert.ok(inputs.length > 0, "nonempty actual module runtime denominator");
  for (const input of inputs) {
    const native = await load(input.code);
    const reference = await load(input.upstreamCode);
    const configurations = [{ inheritAttrs: true, props: {} }];
    if (input.rootElement) configurations.push(
      { inheritAttrs: true, props: { title: "inherited", "data-owner": "caller" } },
      { inheritAttrs: false, props: { title: "inherited", "data-owner": "caller" } },
    );
    const traces = [];
    for (const configuration of configurations) {
      const actual = await mount(native, configuration);
      const expected = await mount(reference, configuration);
      assert.deepEqual(actual.tree, expected.tree, `${input.id} actual pinned runtime tree`);
      const repeated = await mount(native, configuration);
      assert.deepEqual(repeated.tree, actual.tree, `${input.id} template clone reuse`);
      for (const [index, node] of repeated.retained.entries()) assert.notEqual(node, actual.retained[index]);
      if (input.hydrate) {
        const hydrated = await mount(native, configuration, expected.html);
        const referenceHydrated = await mount(reference, configuration, expected.html);
        assert.deepEqual(hydrated.tree, referenceHydrated.tree, `${input.id} actual pinned hydration tree`);
        assert.deepEqual(hydrated.tree, actual.tree, `${input.id} hydrated original output`);
      }
      traces.push({ configuration, tree: actual.tree, hydrated: !!input.hydrate, unmounted: [] });
    }
    captured.push({ id: input.id, version: vue.version, traces });
  }
  process.stdout.write(JSON.stringify(captured));
} finally {
  delete globalThis.__nativeVaporRuntime;
  await window.happyDOM.close();
}
