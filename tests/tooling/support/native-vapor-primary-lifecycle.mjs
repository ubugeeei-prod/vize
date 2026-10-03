// Dev-only primary controls. Fragment residue is evidence, never native lifecycle credit.
import assert from "node:assert/strict";
import { createRequire } from "node:module";
import { pathToFileURL } from "node:url";
import { pluginVersion } from "./native-vapor-sfc-oracle.mjs";
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
console.info = (...values) => process.stderr.write(values.join(" ") + "\n");
const runtimeUrl = pathToFileURL(vueVaporBrowserRuntime).href;
const vue = await import(runtimeUrl);
assert.equal(vue.version, vueVaporVersion);
assert.equal(vue.version, "3.6.0-rc.9", "audited primary lifecycle release");
const dataUrl = (code) => `data:text/javascript;base64,${Buffer.from(code).toString("base64")}`;
const moduleDefaults = new WeakSet();
let sequence = 0;

async function load(reference) {
  assert.equal(typeof reference.code, "string");
  assert.equal(typeof reference.helperCode, "string");
  const helperUrl = dataUrl(reference.helperCode);
  assert.equal(typeof (await import(helperUrl)).default, "function", "actual stock export-helper");
  const owners = new Map([
    ["vue", runtimeUrl],
    ["\0plugin-vue:export-helper", helperUrl],
  ]);
  const resolved = transformSync(reference.code, {
    configFile: false,
    babelrc: false,
    plugins: [
      () => ({
        visitor: {
          ImportDeclaration(path) {
            const owner = path.node.source.value;
            assert.ok(owners.has(owner), `unsupported primary module import: ${owner}`);
            path.node.source.value = owners.get(owner);
          },
        },
      }),
    ],
  });
  const component = (await import(dataUrl(resolved.code + `\n// primary component ${sequence++}`)))
    .default;
  assert.equal(component?.__vapor, true, "actual stock default component");
  assert.equal(typeof component.render, "function", "actual render installation");
  assert.equal(typeof reference.multiRoot, "boolean");
  assert.equal(component.__multiRoot, reference.multiRoot, "actual stock root metadata");
  moduleDefaults.add(component);
  return component;
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
const tree = (host) => [...host.childNodes].map(observe);
function descendants(host) {
  const nodes = [];
  const collect = (parent, parentPath) => {
    for (const [index, node] of [...parent.childNodes].entries()) {
      const path = [...parentPath, index];
      nodes.push({ node, path });
      collect(node, path);
    }
  };
  collect(host, []);
  return nodes;
}

async function mount(component, configuration, serverHtml = null) {
  const host = document.body.appendChild(document.createElement("div"));
  if (serverHtml !== null) host.innerHTML = serverHtml;
  const originalTree = tree(host);
  const originalRoots = [...host.childNodes];
  const originals = descendants(host);
  assert.ok(moduleDefaults.has(component), "exact primary loaded default passed");
  component.inheritAttrs = configuration.inheritAttrs;
  const app = (serverHtml === null ? vue.createVaporApp : vue.createVaporSSRApp)(
    component,
    configuration.props,
  );
  assert.equal(
    app._component.render,
    component.render,
    "actual primary runtime consumes default render identity",
  );
  assert.equal(app._component.__multiRoot, component.__multiRoot);
  const diagnostics = [];
  app.config.warnHandler = (message) => diagnostics.push(message);
  app.config.errorHandler = (error) => diagnostics.push(String(error));
  try {
    app.mount(host);
    await vue.nextTick();
    const mountedTree = tree(host);
    const nodes = descendants(host).map(({ node }) => node);
    const retainedIdentities = originals.map(({ node, path }, index) => ({
      original: index,
      path,
      retained: host.contains(node),
    }));
    assert.ok(
      retainedIdentities.every((identity) => identity.retained),
      "primary hydration retains every original SSR node",
    );
    app.unmount();
    await vue.nextTick();
    const residue = tree(host);
    assert.deepEqual(diagnostics, [], "primary lifecycle diagnostics");
    if (serverHtml !== null && component.__multiRoot) {
      assert.deepEqual(
        [originalTree[0], originalTree.at(-1)],
        [
          [8, "["],
          [8, "]"],
        ],
        "genuine stock SSR fragment boundaries",
      );
      assert.deepEqual(
        residue,
        [
          [8, "["],
          [8, "]"],
        ],
        "audited rc.9 primary fragment unmount residue",
      );
      assert.equal(host.childNodes[0], originalRoots[0], "original opening marker remains");
      assert.equal(host.childNodes[1], originalRoots.at(-1), "original closing marker remains");
    } else assert.deepEqual(residue, [], "primary non-fragment lifecycle leaves no nodes");
    return {
      nodes,
      trace: {
        defaultAuthority: "loaded-default/runtime-render-identity",
        originalTree,
        tree: mountedTree,
        diagnostics,
        retainedIdentities,
        unmounted: residue,
      },
    };
  } finally {
    host.remove();
  }
}

const chunks = [];
for await (const chunk of process.stdin) chunks.push(chunk);
const inputs = JSON.parse(Buffer.concat(chunks).toString("utf8"));
const traces = [];
const counts = {
  fixtureGroups: 0,
  configurations: 0,
  mountOnly: 0,
  hydrated: 0,
  cleanHydrated: 0,
  fragmentedHydrationWithRetainedMarkers: 0,
};
try {
  assert.ok(Array.isArray(inputs) && inputs.length > 0, "nonempty primary control denominator");
  assert.equal(
    new Set(inputs.map(({ id }) => id)).size,
    inputs.length,
    "unique ordered fixture ids",
  );
  for (const input of inputs) {
    assert.equal(typeof input.id, "string");
    const component = await load(input.reference);
    const configurations = input.configurations ?? [{ props: {}, inheritAttrs: true }];
    assert.ok(configurations.length > 0, "nonempty configuration denominator");
    if (input.serverHtmlByConfiguration)
      assert.equal(
        input.serverHtmlByConfiguration.length,
        configurations.length,
        "one genuine SSR control per caller configuration",
      );
    const entries = [];
    for (const [index, supplied] of configurations.entries()) {
      const configuration = {
        props: supplied.props ?? {},
        inheritAttrs: supplied.inheritAttrs ?? true,
      };
      assert.equal(typeof configuration.inheritAttrs, "boolean");
      const serverHtml =
        input.serverHtmlByConfiguration?.[index] ??
        supplied.serverHtml ??
        input.reference.serverHtml;
      assert.equal(typeof serverHtml, "string", "genuine stock SSR HTML control");
      if (!input.serverHtmlByConfiguration && supplied.serverHtml === undefined) {
        assert.equal(configurations.length, 1, "configured caller needs its own stock SSR control");
        assert.deepEqual(configuration.props, {}, "configured attrs need matching stock SSR HTML");
      }
      const mounted = await mount(component, configuration);
      const cloned = await mount(component, configuration);
      assert.deepEqual(cloned.trace.tree, mounted.trace.tree, "stock component clone output");
      assert.ok(
        cloned.nodes.every((node) => !mounted.nodes.includes(node)),
        "fresh mount positively creates distinct DOM nodes",
      );
      let hydration;
      if (serverHtml === "") {
        assert.equal(component.__multiRoot, false);
        assert.deepEqual(mounted.trace.tree, [], "empty SSR control matches empty primary output");
        hydration = {
          status: "mount-only",
          reason: "empty SSR container triggers full-mount fallback",
        };
        counts.mountOnly++;
      } else {
        const hydrated = await mount(component, configuration, serverHtml);
        assert.deepEqual(
          hydrated.trace.tree,
          hydrated.trace.originalTree,
          "raw genuine stock SSR tree survives primary hydration",
        );
        if (component.__multiRoot) {
          assert.deepEqual(
            hydrated.trace.tree.slice(1, -1),
            mounted.trace.tree,
            "raw hydrated fragment surrounds primary mounted content",
          );
          counts.fragmentedHydrationWithRetainedMarkers++;
        } else {
          assert.deepEqual(
            hydrated.trace.tree,
            mounted.trace.tree,
            "primary single-root mounted and hydrated raw trees",
          );
          counts.cleanHydrated++;
        }
        hydration = {
          status: component.__multiRoot ? "retained-fragment-markers" : "clean",
          ...hydrated.trace,
        };
        counts.hydrated++;
      }
      entries.push({
        configuration,
        serverHtml,
        mounted: mounted.trace,
        cloned: {
          ...cloned.trace,
          nodeCount: cloned.nodes.length,
          identityCheck: cloned.nodes.length === 0 ? "empty" : "distinct",
        },
        hydration,
      });
      counts.configurations++;
    }
    traces.push({ id: input.id, multiRoot: component.__multiRoot, traces: entries });
    counts.fixtureGroups++;
  }
  process.stdout.write(
    JSON.stringify({
      schema: "vize.native-vapor.primary-component-lifecycle",
      primaryOnly: true,
      version: vue.version,
      pluginVersion,
      counts,
      traces,
    }),
  );
} finally {
  await window.happyDOM.close();
}
