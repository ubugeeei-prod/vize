// Runs complete emitted and official modules against the actual pinned browser runtime.
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { createRequire } from "node:module";
import { vueVaporBrowserRuntime, vueVaporVersion } from "./vue-vapor-release.mjs";
const requireUi = createRequire(new URL("../../../npm/ui/package.json", import.meta.url));
const { Window } = await import(requireUi.resolve("happy-dom"));
const { transformSync } = requireUi("@babel/core");
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
const dataUrl = (code) => `data:text/javascript;base64,${Buffer.from(code).toString("base64")}`;
console.info = (...values) => process.stderr.write(values.join(" ") + "\n");
const vue = await import(dataUrl(readFileSync(vueVaporBrowserRuntime, "utf8")));
assert.equal(vue.version, vueVaporVersion);
for (const name of ["template", "defineVaporComponent", "createVaporApp", "createVaporSSRApp"])
  assert.equal(typeof vue[name], "function", `actual export ${name}`);
globalThis.__nativeVaporRuntime = vue;

const moduleDefaults = new WeakSet();
let sequence = 0;
let activeId = "";
async function load(code, component = false, helperCode = null) {
  if (helperCode !== null) {
    const helper = await import(dataUrl(helperCode));
    assert.equal(typeof helper.default, "function", "actual stock export-helper");
    globalThis.__nativeVaporExportHelper = helper.default;
  }
  // Resolve only parsed import declarations. Authored static strings/comments
  // can contain module-looking text without becoming loader instructions.
  const rewritten = transformSync(code, {
    configFile: false,
    babelrc: false,
    plugins: [
      ({ types: t }) => ({
        visitor: {
          ImportDeclaration(path) {
            const owner = path.node.source.value;
            const declarations = path.node.specifiers.map((specifier) => {
              if (owner === "\0plugin-vue:export-helper") {
                assert.ok(t.isImportDefaultSpecifier(specifier) && helperCode !== null);
                return t.variableDeclarator(
                  specifier.local,
                  t.memberExpression(
                    t.identifier("globalThis"),
                    t.identifier("__nativeVaporExportHelper"),
                  ),
                );
              }
              assert.equal(owner, "vue", "actual runtime module owner");
              assert.ok(t.isImportSpecifier(specifier));
              const name = specifier.imported.name ?? specifier.imported.value;
              assert.equal(typeof vue[name], "function", `module import ${name}`);
              return t.variableDeclarator(
                specifier.local,
                t.memberExpression(
                  t.memberExpression(
                    t.identifier("globalThis"),
                    t.identifier("__nativeVaporRuntime"),
                  ),
                  t.stringLiteral(name),
                  true,
                ),
              );
            });
            path.replaceWith(t.variableDeclaration("const", declarations));
          },
        },
      }),
    ],
  }).code;
  const module = await import(dataUrl(rewritten + `\n// complete module ${sequence++}`));
  if (!component) return module.render;
  assert.equal(module.default?.__vapor, true, "whole default component Vapor marker");
  assert.equal(typeof module.default.render, "function", "whole component render attachment");
  moduleDefaults.add(module.default);
  return module.default;
}

function observe(node) {
  if (node.nodeType === 3 || node.nodeType === 8) return [node.nodeType, node.data];
  assert.equal(node.nodeType, 1);
  return [
    1,
    node.localName,
    node.namespaceURI,
    [...node.attributes]
      .map((attribute) => [attribute.name, attribute.value])
      .sort(([left], [right]) => left.localeCompare(right)),
    [...node.childNodes].map(observe),
  ];
}

async function mount(render, configuration, serverHtml = null, wholeComponent = false) {
  const host = document.body.appendChild(document.createElement("div"));
  if (serverHtml !== null) host.innerHTML = serverHtml;
  const originalNodes = [];
  if (serverHtml !== null && wholeComponent) {
    const collect = (parent) => {
      for (const node of parent.childNodes) {
        originalNodes.push(node);
        collect(node);
      }
    };
    collect(host);
  }
  const component = wholeComponent
    ? render
    : vue.defineVaporComponent({
        inheritAttrs: configuration.inheritAttrs,
        setup: () => render({}),
      });
  if (wholeComponent) {
    assert.ok(moduleDefaults.has(component), "exact loaded default component passed");
    component.inheritAttrs = configuration.inheritAttrs;
  }
  const app = (serverHtml === null ? vue.createVaporApp : vue.createVaporSSRApp)(
    component,
    configuration.props,
  );
  if (wholeComponent) {
    assert.equal(
      app._component.render,
      component.render,
      "actual runtime consumes default render identity",
    );
    assert.equal(
      app._component.__multiRoot,
      component.__multiRoot,
      "actual runtime retains default root metadata",
    );
  }
  const diagnostics = [];
  app.config.warnHandler = (message) => diagnostics.push(message);
  app.config.errorHandler = (error) => diagnostics.push(String(error));
  try {
    app.mount(host);
    await vue.nextTick();
    assert.deepEqual(diagnostics, [], "mounted diagnostics");
    for (const node of originalNodes)
      assert.ok(host.contains(node), `${activeId} hydration retains original SSR node`);
    const tree = [...host.childNodes].map(observe);
    const html = host.innerHTML;
    const retained = [...host.childNodes];
    app.unmount();
    await vue.nextTick();
    assert.equal(
      host.childNodes.length,
      0,
      `${activeId} ${serverHtml === null ? "mounted" : "hydrated"} unmount leaves no nodes`,
    );
    assert.deepEqual(diagnostics, []);
    return { tree, retained, html, hydratedNodeCount: originalNodes.length };
  } finally {
    host.remove();
  }
}

const chunks = [];
for await (const chunk of process.stdin) chunks.push(chunk);
const inputs = JSON.parse(Buffer.concat(chunks).toString("utf8"));
const captured = [];
try {
  assert.ok(inputs.length > 0, "nonempty actual module runtime denominator");
  for (const input of inputs) {
    activeId = input.id;
    const native = await load(input.code, !!input.component);
    const reference = await load(input.upstreamCode, !!input.component, input.helperCode ?? null);
    if (input.component) {
      assert.notEqual(native, reference, "native and primary default component owners differ");
      assert.notEqual(
        native.render,
        reference.render,
        "native and primary default render identities differ",
      );
      assert.equal(native.__multiRoot, input.multiRoot, `${input.id} native root metadata`);
      assert.equal(reference.__multiRoot, input.multiRoot, `${input.id} primary root metadata`);
    }
    const configurations = input.configurations ?? [{ inheritAttrs: true, props: {} }];
    if (input.rootElement && !input.configurations)
      configurations.push(
        { inheritAttrs: true, props: { title: "inherited", "data-owner": "caller" } },
        { inheritAttrs: false, props: { title: "inherited", "data-owner": "caller" } },
      );
    const traces = [];
    for (const [configurationIndex, configuration] of configurations.entries()) {
      const actual = await mount(native, configuration, null, !!input.component);
      const expected = await mount(reference, configuration, null, !!input.component);
      assert.deepEqual(actual.tree, expected.tree, `${input.id} actual pinned runtime tree`);
      const repeated = await mount(native, configuration, null, !!input.component);
      assert.deepEqual(repeated.tree, actual.tree, `${input.id} template clone reuse`);
      for (const [index, node] of repeated.retained.entries())
        assert.notEqual(node, actual.retained[index]);
      let hydrationNodes = 0;
      if (input.hydrate) {
        const hydrated = await mount(
          native,
          configuration,
          input.serverHtmlByConfiguration?.[configurationIndex] ??
            input.serverHtml ??
            expected.html,
          !!input.component,
        );
        hydrationNodes = hydrated.hydratedNodeCount;
        const referenceHydrated = await mount(
          reference,
          configuration,
          input.serverHtmlByConfiguration?.[configurationIndex] ??
            input.serverHtml ??
            expected.html,
          !!input.component,
        );
        assert.deepEqual(
          hydrated.tree,
          referenceHydrated.tree,
          `${input.id} actual pinned hydration tree`,
        );
        assert.deepEqual(hydrated.tree, actual.tree, `${input.id} hydrated original output`);
      }
      traces.push({
        configuration,
        tree: actual.tree,
        hydrated: !!input.hydrate,
        unmounted: [],
        ...(input.component
          ? { hydrationNodes, defaultAuthority: "loaded-default/runtime-render-identity" }
          : {}),
      });
    }
    captured.push({ id: input.id, version: vue.version, traces });
  }
  process.stdout.write(JSON.stringify(captured));
} finally {
  delete globalThis.__nativeVaporRuntime;
  delete globalThis.__nativeVaporExportHelper;
  await window.happyDOM.close();
}
