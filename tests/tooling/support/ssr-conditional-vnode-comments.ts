// #9455: whole SSR modules, official client hydration and conditional updates.
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { createRequire } from "node:module";
import type { Component } from "vue";
import type { Window } from "happy-dom";

type Runtime = typeof import("vue");
type Compiler = typeof import("vue/compiler-sfc");
type Renderer = typeof import("vue/server-renderer");
type Compiled = { code: string; helper?: string };
type Packet = {
  fixture: string;
  build: "development" | "production";
  files: {
    name: string;
    source: string;
    current: Compiled & { map: unknown; errors: unknown[]; warnings: unknown[] };
  }[];
};
type Seams = {
  reference(
    source: string,
    name: string,
    target: "dom" | "ssr",
    provider: { compiler: Compiler; vapor: false; development: boolean; comments: true },
  ): Promise<Compiled>;
  evaluate(
    code: string,
    runtime: Runtime,
    dependencies: Record<string, unknown>,
    helper: string | undefined,
    renderer: Renderer,
  ): Promise<Component>;
};
type Document = Window["document"];
type Node = NonNullable<ReturnType<Document["createTreeWalker"]>["currentNode"]>;
declare global {
  var __vizeConditionalSet: ((value: boolean) => void) | undefined;
  var __vizeConditionalLifecycle: string[];
}

function nodes(host: Node): Node[] {
  const walker = host.ownerDocument!.createTreeWalker(host, 0xffffffff);
  const result: Node[] = [];
  for (let node = walker.nextNode(); node; node = walker.nextNode()) result.push(node);
  return result;
}

function graph(host: Node, before: Node[]) {
  const after = nodes(host);
  return {
    nodes: after.map((node) => ({
      type: node.nodeType,
      name: node.nodeName,
      value: node.nodeValue,
      parent: node.parentNode === host ? -1 : after.indexOf(node.parentNode!),
    })),
    adopted: before.map((node) => after.indexOf(node)),
  };
}

export async function observeConditionalComments(input: Packet, seams: Seams) {
  assert.equal(process.env.NODE_ENV, input.build);
  const fixture = readFileSync(
    new URL(
      "../../_fixtures/differential/compiler/ssr-conditional-vnode-comments.json",
      import.meta.url,
    ),
    "utf8",
  );
  assert.equal(input.fixture, fixture);
  const source = JSON.parse(fixture);
  assert.equal(source.schema, 1);
  assert.equal(source.issue, 9455);
  assert.deepEqual(
    input.files.map((file) => file.name),
    ["Inner", "App"],
  );
  const root = createRequire(new URL("../../../package.json", import.meta.url));
  const fromVue = createRequire(root.resolve("vue-computed-inlay-oracle/package.json"));
  const vue: Runtime = fromVue("vue");
  const compiler: Compiler = fromVue("vue/compiler-sfc");
  const renderer: Renderer = fromVue("vue/server-renderer");
  assert.equal(vue.version, source.vue);
  assert.equal(compiler.version, source.vue);
  assert.equal(fromVue("@vue/server-renderer/package.json").version, source.vue);
  const { document } = globalThis as unknown as { document: Document };
  const observations: unknown[] = [];
  const state: {
    stage: string;
    build: string;
    vue: string;
    observations: unknown[];
    current?: Record<string, unknown>;
  } = {
    stage: "observing",
    build: input.build,
    vue: vue.version,
    observations,
  };

  async function components(target: "dom" | "ssr", actual: boolean, mode: string) {
    const result: Record<string, Component> = {};
    const codes: Record<string, string> = {};
    if (state.current) state.current[`${target}-${actual ? "native" : "primary"}-codes`] = codes;
    for (const file of input.files) {
      assert.equal(file.source, source.files[file.name]);
      assert.deepEqual(file.current.errors, []);
      assert.deepEqual(file.current.warnings, []);
      assert.ok(file.current.map);
      const stock = await seams.reference(file.source, file.name, target, {
        compiler,
        vapor: false,
        development: input.build === "development",
        comments: true,
      });
      const selected = actual && target === "ssr" ? file.current : stock;
      codes[file.name] = selected.code;
      result[file.name] = await seams.evaluate(
        selected.code,
        vue,
        {
          "./Inner.vue": { default: result.Inner },
        },
        selected.helper,
        renderer,
      );
      if (file.name === "Inner" && mode === "vnode") {
        // The same ordinary VDOM-only slot consumer forces SSR's VNode arm.
        result.Inner = {
          props: ["label"],
          render() {
            return vue.h("section", { "data-label": this.label }, this.$slots.default?.());
          },
        };
      }
    }
    return { root: result.App!, codes };
  }

  async function observe(mode: string, initial: boolean, actual: boolean) {
    const diagnostics: unknown[] = [];
    const current = { mode, initial, actual, diagnostics } as Record<string, unknown>;
    state.current = current;
    const originalWarn = console.warn;
    const originalError = console.error;
    const diagnosticsHandler = (...args: unknown[]) => diagnostics.push(args.map(String));
    console.warn = diagnosticsHandler;
    console.error = diagnosticsHandler;
    const host = document.createElement("div");
    document.body.append(host);
    let app: ReturnType<Runtime["createSSRApp"]> | undefined;
    globalThis.__vizeConditionalLifecycle = [];
    globalThis.__vizeConditionalSet = undefined;
    try {
      const server = await components("ssr", actual, mode);
      const client = await components("dom", false, mode);
      current.server = server.codes;
      current.client = client.codes;
      const serverApp = vue.createSSRApp(server.root, { initial });
      serverApp.config.warnHandler = diagnosticsHandler;
      serverApp.config.errorHandler = diagnosticsHandler;
      const html = await renderer.renderToString(serverApp);
      current.html = html;
      host.innerHTML = html;
      const before = nodes(host);
      const parents = before.map((node) => node.parentNode);
      current.before = graph(host, before);
      const tail = host.querySelector("[data-tail]");
      assert.ok(tail);
      app = vue.createSSRApp(client.root, { initial });
      app.config.warnHandler = diagnosticsHandler;
      app.config.errorHandler = diagnosticsHandler;
      app.mount(host);
      await vue.nextTick();
      const hydrated = graph(host, before);
      current.hydrated = hydrated;
      let previous = -1;
      before.forEach((node, index) => {
        const position = nodes(host).indexOf(node);
        assert.ok(position > previous, "every server node adopted in order");
        assert.equal(node.parentNode, parents[index], "physical parent retained");
        previous = position;
      });
      const updates: { show: boolean; html: string; graph: ReturnType<typeof graph> }[] = [];
      current.updates = updates;
      let priorBranch = host.querySelector("b");
      for (const show of [!initial, initial, !initial]) {
        assert.equal(typeof globalThis.__vizeConditionalSet, "function");
        globalThis.__vizeConditionalSet!(show);
        await vue.nextTick();
        const branch = host.querySelector("b");
        updates.push({ show, html: host.innerHTML, graph: graph(host, before) });
        assert.equal(Boolean(branch), show);
        if (priorBranch) assert.equal(priorBranch.isConnected, false, "old branch removed");
        if (show) assert.notEqual(branch, priorBranch, "new branch is a new physical node");
        assert.equal(host.querySelector("[data-tail]"), tail, "tail retained across updates");
        priorBranch = branch;
      }
      app.unmount();
      app = undefined;
      assert.equal(host.innerHTML, "");
      assert.deepEqual(globalThis.__vizeConditionalLifecycle, ["mounted", "unmounted"]);
      assert.deepEqual(diagnostics, []);
      return {
        html,
        hydrated,
        updates,
        diagnostics,
        lifecycle: [...globalThis.__vizeConditionalLifecycle],
        serverCodes: server.codes,
        clientCodes: client.codes,
      };
    } finally {
      app?.unmount();
      host.remove();
      console.warn = originalWarn;
      console.error = originalError;
      globalThis.__vizeConditionalSet = undefined;
    }
  }
  try {
    for (const mode of ["push", "vnode"])
      for (const initial of [false, true]) {
        const reference = await observe(mode, initial, false);
        const actual = await observe(mode, initial, true);
        observations.push({ mode, initial, reference, actual });
        const { serverCodes: _referenceCodes, ...expected } = reference;
        const { serverCodes: _actualCodes, ...observed } = actual;
        assert.deepEqual(observed, expected, "whole HTML, graph, lifecycle and updates");
      }
    state.stage = "complete";
    delete state.current;
    return state;
  } catch (error) {
    throw Object.assign(error instanceof Error ? error : new Error(String(error)), {
      conditionalComments: state,
    });
  }
}
