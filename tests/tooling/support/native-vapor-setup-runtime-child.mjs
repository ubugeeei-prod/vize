// Private child runner: genuine loaded defaults, no reconstructed setup/render.
import assert from "node:assert/strict";
import { createRequire } from "node:module";
import { pathToFileURL } from "node:url";
const fromUi = createRequire(new URL("../../../npm/ui/package.json", import.meta.url));
const runtimeVersion = createRequire(fromUi.resolve("vue-vapor-runtime/package.json"))(
  "./package.json",
).version;
const dataUrl = (code) => `data:text/javascript;base64,${Buffer.from(code).toString("base64")}`;
export async function runVaporSetupSfcRuntimeChild(packet, flavor, negativeAction = null) {
  assert.ok(["development", "production"].includes(flavor));
  const { Window } = await import(fromUi.resolve("happy-dom"));
  const { transformSync } = fromUi("@babel/core");
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
  const runtimePath = fromUi.resolve(
    `vue-vapor-runtime/dist/vue.runtime-with-vapor.esm-browser${flavor === "production" ? ".prod" : ""}.js`,
  );
  const runtimeUrl = pathToFileURL(runtimePath).href;
  const vue = await import(runtimeUrl);
  assert.equal(vue.version, runtimeVersion);
  const defaults = new WeakSet();
  let sequence = 0;
  const resolveOwners = (code, owners) => {
    const nodes = transformSync(code, { configFile: false, babelrc: false, ast: true, code: false })
      .ast.program.body;
    const edits = nodes
      .filter((node) => node.source)
      .map((node) => {
        assert.ok(
          ["ImportDeclaration", "ExportNamedDeclaration", "ExportAllDeclaration"].includes(
            node.type,
          ),
        );
        assert.ok(
          owners.has(node.source.value),
          `unsupported actual module owner ${node.source.value}`,
        );
        return {
          start: node.source.start,
          end: node.source.end,
          text: JSON.stringify(owners.get(node.source.value)),
        };
      })
      .sort((left, right) => right.start - left.start);
    for (const edit of edits) code = code.slice(0, edit.start) + edit.text + code.slice(edit.end);
    return code;
  };
  async function load(code, reference = null) {
    const owners = new Map([["vue", runtimeUrl]]);
    if (reference) {
      assert.equal(typeof reference.helperCode, "string");
      const helperUrl = dataUrl(reference.helperCode);
      assert.equal(
        typeof (await import(helperUrl)).default,
        "function",
        "actual stock helper default",
      );
      owners.set("\0plugin-vue:export-helper", helperUrl);
      for (const virtual of reference.virtualModules)
        owners.set(
          virtual.id,
          dataUrl(
            resolveOwners(virtual.transformed.code, owners) + `\n// actual virtual ${sequence++}`,
          ),
        );
    }
    const component = (
      await import(
        dataUrl(resolveOwners(code, owners) + `\n// actual complete module ${sequence++}`)
      )
    ).default;
    assert.equal(component?.__vapor, true, "actual loaded default Vapor marker");
    assert.equal(typeof component.setup, "function", "actual inline setup default");
    assert.equal(component.render, undefined, "no reconstructed/separate render");
    defaults.add(component);
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
    const originalTree = tree(host),
      originals = descendants(host);
    assert.ok(defaults.has(component), "exact loaded default owner");
    component.inheritAttrs = configuration.inheritAttrs;
    const app = (serverHtml === null ? vue.createVaporApp : vue.createVaporSSRApp)(
      component,
      structuredClone(configuration.props),
    );
    assert.equal(app._component.setup, component.setup, "loaded default setup function consumed");
    assert.equal(app._component.render, undefined);
    const diagnostics = [];
    app.config.warnHandler = (message) => diagnostics.push({ kind: "warn", message });
    app.config.errorHandler = (error) =>
      diagnostics.push({ kind: "error", name: error.name, message: error.message });
    try {
      const proxy = app.mount(host);
      await vue.nextTick();
      assert.equal(proxy, undefined, "original primitive inline setup exposes no public proxy");
      const instance = app._instance;
      if (flavor === "development") {
        assert.ok(instance, "actual dev instance available");
        assert.equal(instance.type.setup, component.setup);
        assert.equal(instance.type.render, undefined);
        assert.deepEqual(Object.keys(instance.setupState ?? {}), []);
        assert.deepEqual(Object.keys(instance.exposed ?? {}), []);
        assert.equal(instance.update, undefined);
        assert.equal(instance.scope.active, true);
        assert.ok(
          instance.block instanceof Node || Array.isArray(instance.block),
          "genuine setup-return Block",
        );
      } else assert.equal(instance, null, "production does not expose private root instance");
      const mountedTree = tree(host),
        nodes = descendants(host).map(({ node }) => node);
      const retainedIdentities = originals.map(({ node, path }, original) => ({
        original,
        path,
        retained: host.contains(node),
      }));
      if (serverHtml !== null)
        assert.ok(
          retainedIdentities.length > 0 && retainedIdentities.every((entry) => entry.retained),
          "positive original SSR node identities",
        );
      app.unmount();
      await vue.nextTick();
      assert.deepEqual(tree(host), [], "strict actual whole-component unmount leaves zero nodes");
      assert.deepEqual(diagnostics, [], "actual runtime diagnostics");
      if (instance) assert.equal(instance.scope.active, false);
      return {
        nodes,
        trace: {
          originalTree,
          tree: mountedTree,
          retainedIdentities,
          diagnostics,
          defaultAuthority: "actual-loaded-default/runtime-setup-function-identity",
          publicProxy: "undefined",
          render: "undefined",
          unmounted: [],
        },
        authority: {
          privateInstanceObserved: flavor === "development",
          setupReturnBlockObserved: flavor === "development",
          privateState:
            flavor === "development"
              ? "empty setupState/exposed; no update; scope stopped"
              : "unobserved: production app._instance is null",
          loadedSetupIdentity: true,
        },
      };
    } finally {
      host.remove();
    }
  }
  if (negativeAction) {
    assert.equal(packet.fixtures.length, 1, "one exact captured native module per absence frame");
    const fixture = packet.fixtures[0];
    const native = await load(fixture.code);
    const host = document.body.appendChild(document.createElement("div"));
    const app = vue.createVaporApp(native);
    assert.equal(app._component.setup, native.setup);
    const proxy = app.mount(host);
    await vue.nextTick();
    process.stdout.write(
      JSON.stringify({
        action: negativeAction,
        id: fixture.id,
        runtime: flavor,
        version: vue.version,
        loadedSetupIdentity: app._component.setup === native.setup,
        publicProxy: typeof proxy,
        render: typeof native.render,
        originalTree: tree(host),
        privateInstanceObserved: app._instance !== null,
      }),
    );
    try {
      if (negativeAction === "public-force-update") proxy.$forceUpdate();
      else if (negativeAction === "public-primitive-setter") proxy.count = 2;
      else if (negativeAction === "private-instance-update") app._instance.update();
      else assert.fail("unknown absence action");
      assert.fail("native missing API unexpectedly callable");
    } finally {
      app.unmount();
      await vue.nextTick();
      assert.deepEqual(tree(host), [], "absence control still unmounts cleanly");
      host.remove();
      await window.happyDOM.close();
    }
  }
  const fixtures = [],
    counts = {
      fixtureGroups: 0,
      configurations: 0,
      mounts: 0,
      clones: 0,
      hydratedConfigurations: 0,
      hydrationMounts: 0,
      retainedSSRNodes: 0,
      mountOnlyConfigurations: 0,
    };
  try {
    assert.ok(packet.fixtures.length > 0);
    for (const fixture of packet.fixtures) {
      const native = await load(fixture.code),
        primary = await load(fixture.reference.client.code, fixture.reference.client);
      assert.notEqual(native, primary, "native/primary loaded default owners distinct");
      assert.notEqual(
        native.setup,
        primary.setup,
        "native/primary setup function identities distinct",
      );
      const traces = [],
        authorities = [];
      for (const configuration of fixture.configurations) {
        const actual = await mount(native, configuration),
          expected = await mount(primary, configuration);
        assert.deepEqual(
          actual.trace.tree,
          expected.trace.tree,
          `${fixture.id} actual original output`,
        );
        const cloned = await mount(native, configuration),
          primaryClone = await mount(primary, configuration);
        assert.deepEqual(cloned.trace.tree, actual.trace.tree);
        assert.deepEqual(primaryClone.trace.tree, expected.trace.tree);
        assert.ok(
          cloned.nodes.every((node) => !actual.nodes.includes(node)),
          "native fresh clone positive distinct nodes",
        );
        assert.ok(
          primaryClone.nodes.every((node) => !expected.nodes.includes(node)),
          "primary fresh clone positive distinct nodes",
        );
        assert.ok(
          actual.nodes.every((node) => !expected.nodes.includes(node)),
          "native and primary DOM owners distinct",
        );
        const server = configuration.server;
        assert.deepEqual(server.serverDiagnostics ?? [], []);
        let hydration;
        if (server.serverHtml === null || server.serverHtml === "") {
          hydration = {
            status: "mount-only",
            reason:
              server.serverHtml === ""
                ? "empty SSR triggers full-mount fallback"
                : "stock SSR unsupported",
            serverUnsupported: server.serverUnsupported ?? null,
          };
          counts.mountOnlyConfigurations++;
        } else {
          assert.equal(typeof server.serverHtml, "string", "genuine configured SSR HTML");
          const hydrated = await mount(native, configuration, server.serverHtml),
            primaryHydrated = await mount(primary, configuration, server.serverHtml);
          assert.deepEqual(hydrated.trace.originalTree, primaryHydrated.trace.originalTree);
          assert.deepEqual(hydrated.trace.tree, primaryHydrated.trace.tree);
          assert.deepEqual(
            hydrated.trace.tree,
            hydrated.trace.originalTree,
            "genuine original SSR tree retained",
          );
          assert.deepEqual(hydrated.trace.tree, actual.trace.tree);
          hydration = {
            status: "hydrated",
            native: hydrated.trace,
            primary: primaryHydrated.trace,
          };
          counts.hydratedConfigurations++;
          counts.hydrationMounts += 2;
          counts.retainedSSRNodes +=
            hydrated.trace.retainedIdentities.length +
            primaryHydrated.trace.retainedIdentities.length;
        }
        traces.push({
          configuration: { props: configuration.props, inheritAttrs: configuration.inheritAttrs },
          tree: actual.trace.tree,
          native: actual.trace,
          primary: expected.trace,
          clone: { native: cloned.trace, primary: primaryClone.trace },
          serverHtml: server.serverHtml,
          serverAuthority: server.serverAuthority,
          hydration,
          unmounted: [],
        });
        authorities.push({ native: actual.authority, primary: expected.authority });
        counts.configurations++;
        counts.mounts += 2;
        counts.clones += 2;
      }
      fixtures.push({ id: fixture.id, traces, authorities });
      counts.fixtureGroups++;
    }
    return {
      version: vue.version,
      pluginVersion: packet.pluginVersion,
      runtime: flavor,
      runtimePath,
      fixtures,
      counts,
    };
  } finally {
    await window.happyDOM.close();
  }
}
