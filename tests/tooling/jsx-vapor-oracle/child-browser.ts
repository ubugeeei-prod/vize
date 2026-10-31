/** Genuine Chromium template-document and nested-effect regression for #6890. */
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import { createServer } from "node:http";
import { createRequire } from "node:module";
import { dirname, join } from "node:path";
import { chromium } from "@playwright/test";
import { transformVueJsxVapor } from "vue-jsx-vapor/api";

type Module = { code: string; functionName: string };
type Packet = {
  original: {
    source: string;
    initial: { value: { $jsxValue: string } };
    steps: { patch: { value: { $jsxValue: string } } }[];
    expectedText: string[];
    expectedElementIdentities: number[][];
  };
  vapor: Module;
  shadowedSource: string;
  shadowedVapor: Module;
};
const [modulePath, receiptPath] = process.argv.slice(2);
assert(modulePath && receiptPath, "actual source-built producer packet and capture are required");
const input = JSON.parse(fs.readFileSync(modulePath, "utf8")) as Packet;
const beforeBytes = fs.readFileSync(
  new URL(
    "../../_fixtures/differential/compiler/jsx-child-values-6890/browser-before-f7c306d.json",
    import.meta.url,
  ),
);
// Full unmodified producer packet from run41185002277/artifact13036597757, not an emulated failure.
assert.equal(
  createHash("sha256").update(beforeBytes).digest("hex"),
  "47106a01b905c1036ad8c0b79c80928b967b799b0f03a1f91dbd5439a41b0a71",
);
const before = JSON.parse(beforeBytes.toString()) as Packet;
assert.deepEqual(input.original, before.original);
assert.equal(input.shadowedSource, before.shadowedSource);
const require = createRequire(import.meta.url);
const vue = fs.readFileSync(require.resolve("vue/dist/vue.runtime-with-vapor.esm-browser.prod.js"));
const runtimeRequire = createRequire(require.resolve("vue-jsx-vapor"));
const runtime = fs.readFileSync(
  join(dirname(runtimeRequire.resolve("@vue-jsx-vapor/runtime")), "vapor.js"),
);
const subjects = new Map<string, string>();
for (const [name, source] of [
  ["public", input.original.source],
  ["public-shadow", input.shadowedSource],
] as const) {
  const result = transformVueJsxVapor(source, `/browser/${name}.jsx?vapor=true`);
  assert.equal(typeof result.code, "string");
  subjects.set(name, result.code!);
}
for (const [name, module] of [
  ["source", input.vapor],
  ["source-shadow", input.shadowedVapor],
  ["before", before.vapor],
  ["before-shadow", before.shadowedVapor],
] as const)
  subjects.set(name, module.code);
const server = createServer((request, response) => {
  const url = new URL(request.url!, "http://localhost");
  if (url.pathname === "/") {
    response.setHeader("Content-Type", "text/html");
    response.end(
      '<script type="importmap">{"imports":{"vue":"/vue.js","vue-jsx-vapor":"/runtime.js","@vue-jsx-vapor/runtime":"/runtime.js"}}</script>',
    );
  } else if (url.pathname === "/vue.js" || url.pathname === "/runtime.js") {
    response.setHeader("Content-Type", "text/javascript");
    response.end(url.pathname === "/vue.js" ? vue : runtime);
  } else if (subjects.has(url.pathname.slice(1, -3))) {
    response.setHeader("Content-Type", "text/javascript");
    response.end(
      `const state=globalThis.__childState;\n${subjects.get(url.pathname.slice(1, -3))}\nexport {App};`,
    );
  } else {
    response.statusCode = 404;
    response.end();
  }
});
await new Promise<void>((resolve) => server.listen(0, "127.0.0.1", resolve));
const address = server.address();
assert(address && typeof address !== "string");
const browser = await chromium.launch({ headless: true });
const raw: Record<string, unknown> = {
  schema: 1,
  original: input.original,
  beforeSource: "f7c306ddc05377bcaec46d502d995f30f525aa6b",
  browserVersion: browser.version(),
  routes: {},
  wholeCorpusQualified: false,
  issue6890Complete: false,
};
const routes = raw.routes as Record<string, unknown>;
try {
  for (const name of subjects.keys()) {
    const page = await browser.newPage();
    const errors: string[] = [];
    page.on("pageerror", (error) => errors.push(String(error)));
    try {
      await page.goto(`http://127.0.0.1:${address.port}/`);
      routes[name] = await page.evaluate(
        async ({ name, original }) => {
          const entry = "/vue.js";
          const Vue = (await import(entry)) as typeof import("vue");
          const inert = window.document.createElement("template");
          inert.innerHTML = "<main> </main>";
          if (inert.content.ownerDocument.defaultView !== null)
            throw Error("Chromium must expose the real inert template document");
          const nested = Vue.shallowReactive({ value: 0 });
          let reads = 0,
            effectRuns = 0;
          const value = (name: string): unknown => {
            switch (name) {
              case "zero":
                return 0;
              case "empty-string":
                return "";
              case "nan":
                return NaN;
              case "null":
                return null;
              case "undefined":
                return undefined;
              case "false":
                return false;
              case "true":
                return true;
              case "array":
                return [0, "", NaN, null, undefined, false];
              case "nested-vnode":
                return Vue.h("i", { "data-kind": "nested" }, [
                  Vue.h("b", null, "inner"),
                  Vue.h("u", null, "leaf"),
                ]);
              default:
                throw Error("unknown original value");
            }
          };
          const reactive = Vue.shallowReactive({ value: value(original.initial.value.$jsxValue) });
          const state = new Proxy(reactive, {
            get(target, key, receiver) {
              if (key === "value") {
                reads++;
                Vue.effect(() => {
                  nested.value;
                  effectRuns++;
                });
              }
              return Reflect.get(target, key, receiver);
            },
          });
          (globalThis as unknown as { __childState: typeof state }).__childState = state;
          const module = await import(`/${name}.js`);
          const authored = !name.startsWith("public");
          const app = Vue.createVaporApp(
            authored ? module.App : Vue.defineVaporComponent({ setup: () => module.App() }),
          );
          const warnings: string[] = [];
          app.config.warnHandler = (message) => warnings.push(message);
          app.config.errorHandler = (error) => warnings.push(String(error));
          const host = window.document.createElement("div");
          window.document.body.append(host);
          const ids = new WeakMap<object, number>();
          let nextId = 0;
          const observe = (parent: Node): unknown[] => {
            const nodes: unknown[] = [];
            for (const node of parent.childNodes) {
              if (node.nodeType === 3) {
                if (node.textContent) nodes.push(node.textContent);
              } else if (node.nodeType === 1) {
                const element = node as Element;
                nodes.push({
                  tag: element.localName,
                  attributes: Object.fromEntries(
                    [...element.attributes]
                      .map((a) => [a.name, a.value])
                      .sort(([a], [b]) => a.localeCompare(b)),
                  ),
                  children: observe(node),
                });
              }
            }
            return nodes;
          };
          let mounted = false;
          try {
            app.mount(host);
            mounted = true;
            await Vue.nextTick();
            if (warnings.length) throw Error(JSON.stringify(warnings));
            const phases = [];
            for (const [index, phase] of [
              original.initial,
              ...original.steps.map((step) => step.patch),
            ].entries()) {
              if (index) {
                state.value = value(phase.value.$jsxValue);
                await Vue.nextTick();
              }
              const previous = effectRuns;
              nested.value++;
              await Vue.nextTick();
              if (effectRuns !== previous + 1)
                throw Error(`nested child scopes retained at${index}: ${previous}->${effectRuns}`);
              const elements = [...host.querySelectorAll("*")];
              for (const element of elements) if (!ids.has(element)) ids.set(element, nextId++);
              phases.push({
                dom: observe(host),
                text: host.textContent,
                elementIdentities: elements.map((element) => ids.get(element)),
                warnings: [...warnings],
                reads,
                effectRuns,
              });
            }
            app.unmount();
            mounted = false;
            await Vue.nextTick();
            const beforeReads = reads,
              beforeEffects = effectRuns;
            state.value = "after-unmount";
            nested.value++;
            await Vue.nextTick();
            if (reads !== beforeReads || effectRuns !== beforeEffects)
              throw Error("authored getter or nested child effect survived unmount");
            return {
              phases,
              readsAfterUnmount: reads,
              effectsAfterUnmount: effectRuns,
              completeUnmount: host.childNodes.length === 0,
              inertTemplateDefaultView: null,
            };
          } finally {
            if (mounted) app.unmount();
            host.remove();
          }
        },
        { name, original: input.original },
      );
    } catch (error) {
      routes[name] = { error: String(error) };
    } finally {
      if (errors.length) routes[name] = { route: routes[name], pageErrors: errors };
      await page.close();
    }
  }
  for (const name of ["before", "before-shadow"]) {
    assert.match(
      JSON.stringify(routes[name]),
      /Vize JSX child anchor has no owned DOM text realm/,
      `the real prior producer must reproduce inert-template failure: ${name}`,
    );
  }
  for (const [name, reference] of [
    ["source", "public"],
    ["source-shadow", "public-shadow"],
  ]) {
    const actual = routes[name] as {
      phases?: { text: string; elementIdentities: number[]; warnings: string[]; reads: number }[];
      completeUnmount: boolean;
    };
    assert(actual.phases, `complete actual Chromium route ${name}: ${JSON.stringify(actual)}`);
    assert.equal(actual.phases.length, 10);
    assert.equal(actual.completeUnmount, true);
    assert.deepEqual(actual, routes[reference], `whole target-specific browser trace ${name}`);
    for (const [index, phase] of actual.phases.entries()) {
      assert.deepEqual(phase.warnings, []);
      assert.equal(phase.reads, index + 1);
      if (index === 8) {
        assert.equal(phase.text, "[object Object]");
        assert.deepEqual(phase.elementIdentities, [0]);
      } else {
        assert.equal(phase.text, input.original.expectedText[index]);
        assert.deepEqual(phase.elementIdentities, input.original.expectedElementIdentities[index]);
      }
    }
  }
  raw.qualifiedVaporPhases = 9;
  raw.referenceConflictPhases = 1;
} finally {
  fs.writeFileSync(receiptPath, JSON.stringify(raw, null, 2) + "\n");
  await browser.close();
  await new Promise<void>((resolve, reject) =>
    server.close((error) => (error ? reject(error) : resolve())),
  );
}
