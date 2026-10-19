// #9455: additive cases in the existing whole-SFC slot-forwarding carrier.
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { createRequire, stripTypeScriptTypes } from "node:module";
import type { Component, Plugin } from "vue";
import type { Window } from "happy-dom";

type Document = Window["document"];
type Host = ReturnType<Document["createElement"]>;
type DomNode = NonNullable<ReturnType<Document["createTreeWalker"]>["currentNode"]>;

type Target = "dom" | "ssr" | "vapor" | "vapor-ssr";
type Runtime = typeof import("vue") & {
  createVaporSSRApp?: typeof import("vue").createSSRApp;
  vaporInteropPlugin?: Plugin;
};
type Compiler = typeof import("vue/compiler-sfc");
type Renderer = typeof import("vue/server-renderer");
type Module = Record<string, unknown>;
type Compiled = { code: string; helper?: string };
type NativeFile = {
  name: string;
  source: string;
  current: Compiled & { errors: unknown[]; warnings: unknown[]; map: unknown };
};
type NativeCase = { name: string; target: Target; files: NativeFile[] };
type Fixture = {
  schema: number;
  issue: number;
  values: string[];
  targets: Target[];
  cases: { name: string; source: string }[];
  files: Record<string, string>;
  reka: {
    version: string;
    gitHead: string;
    npmIntegrity: string;
    sources: { name: string; path: string; source: string; sha256: string; gitBlob: string }[];
    license: { source: string; sha256: string; gitBlob: string };
  };
};
type Packet = { fixture: string; cases: NativeCase[] };
type Seams = {
  vue: Runtime;
  vapor: Runtime;
  compiler: Compiler;
  server: Renderer;
  reference(
    source: string,
    name: string,
    target: Target,
    provider?: { compiler: Compiler; vapor: boolean },
  ): Promise<Compiled>;
  evaluate(
    code: string,
    runtime: Runtime,
    dependencies: Module,
    helper?: string,
    renderer?: Renderer,
  ): Promise<Component & { __vapor?: boolean }>;
  evaluateModule(
    code: string,
    runtime: Runtime,
    dependencies: Module,
    helper?: string,
    renderer?: Renderer,
  ): Promise<Module>;
};
type Profile = {
  name: string;
  vue: Runtime;
  client: Runtime;
  compiler: Compiler;
  renderer: Renderer;
  clientTarget: Target;
  serverTarget: Target;
};
declare global {
  var __vizeKeyedLifecycle: [string, string][];
}

const hash = (value: string | Buffer) => createHash("sha256").update(value).digest("hex");
const fixtureUrl = new URL(
  "../../_fixtures/differential/compiler/ssr-keyed-template-components.json",
  import.meta.url,
);
const names = [
  "keyed-component",
  "keyed-element",
  "unkeyed-component",
  "direct-keyed-component",
  "keyed-multiple-roots",
];

function nodeRecord(node: DomNode) {
  return { type: node.nodeType, name: node.nodeName, text: node.nodeValue };
}

function nodes(host: Host): DomNode[] {
  const walker = host.ownerDocument.createTreeWalker(host, 0xffffffff);
  const result: DomNode[] = [];
  for (let node = walker.nextNode(); node; node = walker.nextNode()) result.push(node);
  return result;
}

export async function observeKeyedTemplates(input: Packet, seams: Seams) {
  const { document, window } = globalThis as unknown as { document: Document; window: Window };
  const raw = readFileSync(fixtureUrl, "utf8");
  assert.equal(input.fixture, raw, "all native-authored fixture bytes");
  const fixture = JSON.parse(raw) as Fixture;
  assert.equal(fixture.schema, 1);
  assert.equal(fixture.issue, 9455);
  assert.deepEqual(
    fixture.cases.map((c) => c.name),
    names,
  );
  assert.deepEqual(fixture.targets, ["dom", "ssr", "vapor", "vapor-ssr"]);
  assert.deepEqual(fixture.values, ["primary", "solid", "md"]);
  assert.equal(fixture.reka.version, "2.10.1");
  assert.equal(fixture.reka.gitHead, "47c433a84ad819eb6becbffc0cc7419e282cf07e");
  assert.equal(
    fixture.reka.npmIntegrity,
    "sha512-drcOQ4rQtDYAcGCsyQBqQg8QQ+H3B+zDaMJU0h8KPEPMa7g9BHu3zcOi4OB39XJSWizceFoNO0Z9tctSGLOXqg==",
  );
  assert.deepEqual(
    fixture.reka.sources.map((s) => s.name),
    ["renderSlotFragments", "Slot", "Primitive"],
  );
  for (const source of [...fixture.reka.sources, fixture.reka.license]) {
    assert.equal(hash(source.source), source.sha256);
    assert.equal(
      createHash("sha1")
        .update(`blob ${Buffer.byteLength(source.source)}\0`)
        .update(source.source)
        .digest("hex"),
      source.gitBlob,
    );
  }
  assert.equal(
    input.cases.length,
    names.length * fixture.targets.length,
    "complete target inventory",
  );
  for (const name of names)
    for (const target of fixture.targets) {
      const selected = input.cases.filter((c) => c.name === name && c.target === target);
      assert.equal(selected.length, 1, `${name}/${target}: unique native packet`);
      const entry = selected[0];
      assert(entry);
      assert.deepEqual(
        entry.files.map((f) => f.name),
        ["Value", "TriggerPush", "TriggerVNode", "App"],
      );
      for (const file of entry.files) {
        const authored =
          file.name === "App"
            ? fixture.cases.find((c) => c.name === name)?.source
            : fixture.files[file.name];
        assert.equal(file.source, authored, `${name}/${target}/${file.name}: authored source`);
        assert.deepEqual(file.current.errors, [], "compiler errors");
        assert.deepEqual(file.current.warnings, [], "compiler warnings");
        assert.equal(typeof file.current.code, "string");
        assert(file.current.map && typeof file.current.map === "object", "complete source map");
      }
    }
  const fromRoot = createRequire(new URL("../../../package.json", import.meta.url));
  const fromNuxt = createRequire(fromRoot.resolve("vue-computed-inlay-oracle/package.json"));
  const nuxtVue = fromNuxt("vue") as Runtime;
  const nuxtCompiler = fromNuxt("vue/compiler-sfc") as Compiler;
  const nuxtRenderer = fromNuxt("vue/server-renderer") as Renderer;
  assert.equal(nuxtVue.version, "3.5.41");
  assert.equal(nuxtCompiler.version, "3.5.41");
  assert.equal(fromNuxt("@vue/server-renderer/package.json").version, "3.5.41");
  for (const version of [seams.vue.version, seams.vapor.version, seams.compiler.version])
    assert.equal(version, "3.6.0-rc.9");
  assert.equal(
    typeof seams.vapor.createVaporSSRApp,
    "function",
    "required supported Vapor hydration API",
  );
  assert.equal(
    typeof seams.vapor.vaporInteropPlugin,
    "function",
    "required supported Vapor VDOM interop plugin",
  );
  const profiles: Profile[] = [
    {
      name: "nuxt-vdom-3.5.41",
      vue: nuxtVue,
      client: nuxtVue,
      compiler: nuxtCompiler,
      renderer: nuxtRenderer,
      clientTarget: "dom",
      serverTarget: "ssr",
    },
    {
      name: "supported-vdom-rc.9",
      vue: seams.vue,
      client: seams.vue,
      compiler: seams.compiler,
      renderer: seams.server,
      clientTarget: "dom",
      serverTarget: "ssr",
    },
    {
      name: "supported-vapor-rc.9",
      vue: seams.vue,
      client: seams.vapor,
      compiler: seams.compiler,
      renderer: seams.server,
      clientTarget: "vapor",
      serverTarget: "vapor-ssr",
    },
  ];
  const report = {
    fixtureSha256: hash(raw),
    fixture,
    primaryProvider: {
      version: "3.5.41",
      manifest: fromNuxt.resolve("vue/package.json"),
      manifestSha256: hash(readFileSync(fromNuxt.resolve("vue/package.json"))),
    },
    observations: [] as {
      profile: string;
      name: string;
      mode: string;
      initial: string;
      reference?: unknown;
      actual?: unknown;
    }[],
    current: null as unknown,
    stage: "start",
  };
  async function bridge(runtime: Runtime, renderer: Renderer) {
    const modules: Record<string, Module> = {};
    for (const file of fixture.reka.sources) {
      const code = stripTypeScriptTypes(file.source, { mode: "strip" });
      modules[file.name] = await seams.evaluateModule(
        code,
        runtime,
        {
          "@/shared": modules.renderSlotFragments,
          "./Slot": modules.Slot,
        },
        undefined,
        renderer,
      );
    }
    assert(modules.Primitive && Object.hasOwn(modules.Primitive, "Primitive"));
    return {
      module: modules.Primitive,
      sources: fixture.reka.sources.map((s) => ({
        ...s,
        transformedCode: stripTypeScriptTypes(s.source, { mode: "strip" }),
      })),
    };
  }
  async function component(
    name: string,
    target: Target,
    mode: string,
    useActual: boolean,
    profile: Profile,
    runtime: Runtime,
  ) {
    const native = input.cases.find((c) => c.name === name && c.target === target);
    assert(native);
    const reka = await bridge(runtime, profile.renderer);
    const components: Record<string, Component & { __vapor?: boolean }> = {};
    const codes: Record<string, Compiled & { source: string }> = {};
    for (const filename of ["Value", mode === "push" ? "TriggerPush" : "TriggerVNode", "App"]) {
      const file = native.files.find((f) => f.name === filename);
      assert(file);
      const compiled = useActual
        ? file.current
        : await seams.reference(file.source, filename, target, {
            compiler: profile.compiler,
            vapor: target.startsWith("vapor"),
          });
      components[filename] = await seams.evaluate(
        compiled.code,
        runtime,
        {
          "./Primitive": reka.module,
          "./Value.vue": { default: components.Value },
          "./Trigger.vue": { default: components.TriggerPush ?? components.TriggerVNode },
        },
        compiled.helper,
        profile.renderer,
      );
      if (target.startsWith("vapor"))
        assert.equal(
          components[filename]?.__vapor,
          true,
          `${filename}/${target}: no VDOM fallback`,
        );
      codes[filename] = { ...compiled, source: file.source };
    }
    assert(components.App);
    return { root: components.App, codes, reka: reka.sources };
  }
  async function observe(
    name: string,
    mode: string,
    initial: string,
    useActual: boolean,
    profile: Profile,
  ) {
    const diagnostics: { phase: string; channel: string; message: string }[] = [];
    let phase = "server";
    const recordDiagnostic = (channel: string, message: unknown) =>
      diagnostics.push({ phase, channel, message: String(message) });
    const originalWarn = console.warn,
      originalError = console.error;
    console.warn = (...args: unknown[]) => recordDiagnostic("warn", args.map(String).join(" "));
    console.error = (...args: unknown[]) => recordDiagnostic("error", args.map(String).join(" "));
    globalThis.__vizeKeyedLifecycle = [];
    const bodyChildren = document.body.childNodes.length;
    let app: ReturnType<Runtime["createSSRApp"]> | undefined;
    let host: Host | undefined;
    let failure: Error | undefined;
    const observation: Record<string, unknown> = {
      name,
      mode,
      initial,
      useActual,
      diagnostics,
      phase,
    };
    const captureFailure = (key: string, cause: unknown) => {
      const err = cause instanceof Error ? cause : new Error(JSON.stringify(cause));
      observation[key] = { name: err.name, message: err.message, stack: err.stack };
      return err;
    };
    report.current = observation;
    try {
      const server = await component(
        name,
        profile.serverTarget,
        mode,
        useActual,
        profile,
        profile.vue,
      );
      observation.server = server;
      const serverApp = profile.vue.createSSRApp(server.root, { initial });
      serverApp.config.warnHandler = (message) => recordDiagnostic("handler-warn", message);
      serverApp.config.errorHandler = (error) => recordDiagnostic("handler-error", error);
      const html = await profile.renderer.renderToString(serverApp);
      observation.html = html;
      assert.deepEqual(globalThis.__vizeKeyedLifecycle, []);
      assert.deepEqual(diagnostics, []);
      phase = "hydrate";
      host = document.createElement("div");
      document.body.appendChild(host);
      host.innerHTML = html;
      const before = nodes(host);
      const parents = before.map((n) => n.parentNode);
      observation.serverNodes = before.map(nodeRecord);
      const client = await component(
        name,
        profile.clientTarget,
        mode,
        useActual,
        profile,
        profile.client,
      );
      observation.client = client;
      const create =
        profile.clientTarget === "vapor"
          ? profile.client.createVaporSSRApp
          : profile.client.createSSRApp;
      assert(create, "the required hydration target must be available");
      app = create(client.root, { initial });
      app.config.warnHandler = (message) => recordDiagnostic("handler-warn", message);
      app.config.errorHandler = (error) => recordDiagnostic("handler-error", error);
      if (profile.clientTarget === "vapor") {
        assert(profile.client.vaporInteropPlugin);
        app.use(profile.client.vaporInteropPlugin);
      }
      app.mount(host);
      await profile.client.nextTick();
      const after = nodes(host);
      const graph = {
        nodes: after.map(nodeRecord),
        parents: after.map((n) => after.findIndex((p) => p === n.parentNode)),
        adoption: before.map((n, i) => [after.indexOf(n), n.parentNode === parents[i]]),
      };
      observation.hydrationGraph = graph;
      const retained = after.filter((n) => before.includes(n));
      before.forEach((n, i) => assert.strictEqual(n, retained[i], `server node ${i} reused`));
      assert.deepEqual(
        graph.adoption.map(([, parent]) => parent),
        before.map(() => true),
      );
      assert.deepEqual(diagnostics, []);
      assert.equal(host.querySelector("[data-value]")?.textContent, initial);
      assert.equal(host.querySelector("[data-tail]")?.textContent, "tail");
      observation.hydratedHtml = host.innerHTML;
      observation.hydratedLifecycle = [...globalThis.__vizeKeyedLifecycle];
      const originalValue = host.querySelector("[data-value]");
      const originalTail = host.querySelector("[data-tail]");
      assert(originalValue && originalTail);
      phase = "update";
      const update = host.querySelector("[data-update]");
      assert(update);
      update.dispatchEvent(new window.Event("click", { bubbles: true }));
      await profile.client.nextTick();
      assert.equal(host.querySelector("[data-value]")?.textContent, "updated");
      assert.equal(host.querySelector("[data-tail]")?.textContent, "tail");
      const updatedNodes = nodes(host);
      const valueReused = host.querySelector("[data-value]") === originalValue;
      const tailReused = host.querySelector("[data-tail]") === originalTail;
      const oldValueConnected = host.contains(originalValue);
      observation.updatedIdentity = {
        valueReused,
        oldValueConnected,
        tailReused,
        retainedServerNodes: before.map((node) => updatedNodes.indexOf(node)),
      };
      assert.equal(valueReused, name === "unkeyed-component", "only unkeyed value retained");
      assert.equal(oldValueConnected, valueReused, "replaced keyed value detached");
      assert.equal(tailReused, true, "following sibling is the same physical node");
      assert.deepEqual(diagnostics, []);
      observation.updatedHtml = host.innerHTML;
      observation.updatedLifecycle = [...globalThis.__vizeKeyedLifecycle];
    } catch (error) {
      failure = captureFailure("failure", error);
    } finally {
      phase = "unmount";
      let cleanupFailure: unknown;
      try {
        app?.unmount();
        await profile.client.nextTick();
        observation.finalLifecycle = [...globalThis.__vizeKeyedLifecycle];
      } catch (error) {
        cleanupFailure = error;
      } finally {
        host?.remove();
        console.warn = originalWarn;
        console.error = originalError;
      }
      try {
        assert.equal(document.body.childNodes.length, bodyChildren, "owned DOM completely removed");
        assert.deepEqual(diagnostics, []);
      } catch (error) {
        cleanupFailure ??= error;
      }
      if (cleanupFailure) {
        const cleanup = captureFailure("cleanupFailure", cleanupFailure);
        failure ??= cleanup;
      }
    }
    if (failure) throw failure;
    return observation;
  }
  try {
    for (const profile of profiles)
      for (const name of names)
        for (const mode of ["push", "vnode"])
          for (const initial of fixture.values) {
            report.stage = "observe";
            const row: (typeof report.observations)[number] = {
              profile: profile.name,
              name,
              mode,
              initial,
            };
            report.observations.push(row);
            row.reference = await observe(name, mode, initial, false, profile);
            row.actual = await observe(name, mode, initial, true, profile);
            const reference = row.reference as Record<string, unknown>,
              actual = row.actual as Record<string, unknown>;
            for (const key of [
              "html",
              "hydratedHtml",
              "hydrationGraph",
              "updatedHtml",
              "updatedIdentity",
              "hydratedLifecycle",
              "updatedLifecycle",
              "finalLifecycle",
              "diagnostics",
            ])
              assert.deepEqual(
                actual[key],
                reference[key],
                `${profile.name}/${name}/${mode}/${initial}: whole ${key}`,
              );
          }
    assert.equal(report.observations.length, 90);
    report.stage = "complete";
    return report;
  } catch (error) {
    throw Object.assign(error instanceof Error ? error : new Error(String(error)), {
      keyedTemplates: report,
    });
  }
}
