/** Installed-public Native VDOM against the unchanged original Babel producer in Chromium. */
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import { createServer } from "node:http";
import { createRequire } from "node:module";
import { transformSync } from "@babel/core";
import babelJsx from "@vue/babel-plugin-jsx";
import { chromium } from "@playwright/test";

type Original = {
  source: string;
  initial: { value: { $jsxValue: string } };
  steps: { patch: { value: { $jsxValue: string } } }[];
  expectedText: string[];
  expectedElementIdentities: number[][];
};
type Module = { code: string; functionName: string };
type Packet = {
  original: Original;
  vdom: Module;
  shadowedSource: string;
  shadowedVdom: Module;
};
const [packetPath, outputPath] = process.argv.slice(2);
assert(packetPath && outputPath);
const input = JSON.parse(fs.readFileSync(packetPath, "utf8")) as Packet;
const originalBytes = fs.readFileSync(
  new URL(
    "../../_fixtures/differential/compiler/jsx-child-values-6890/original.json",
    import.meta.url,
  ),
);
assert.equal(
  createHash("sha256").update(originalBytes).digest("hex"),
  "810595733fc8904df998e7d2a1ac1d70319c0be22046d6cb25845b6c46697a37",
);
const original = JSON.parse(originalBytes.toString()) as Original;
assert.deepEqual(input.original, original);
assert.equal(
  input.shadowedSource,
  "const Node=1;const document=1;const Array=1;const String=1;" + original.source,
);
const subjects = new Map<string, string>();
for (const [suffix, source, module] of [
  ["", original.source, input.vdom],
  ["-shadow", input.shadowedSource, input.shadowedVdom],
] as const) {
  const reference = transformSync(source, {
    configFile: false,
    babelrc: false,
    filename: "/semantic-oracle/children/dynamic-value.jsx",
    plugins: [babelJsx],
  });
  assert.equal(typeof reference?.code, "string");
  assert.equal(module.functionName, "App");
  assert.equal(typeof module.code, "string");
  assert(module.code.length > 0);
  subjects.set("reference" + suffix, reference!.code!);
  subjects.set("native" + suffix, module.code);
}
const require = createRequire(import.meta.url);
const vue = fs.readFileSync(require.resolve("vue/dist/vue.runtime-with-vapor.esm-browser.prod.js"));
const server = createServer((request, response) => {
  const pathname = new URL(request.url!, "http://localhost").pathname;
  if (pathname === "/") {
    response.setHeader("Content-Type", "text/html");
    response.end('<script type="importmap">{"imports":{"vue":"/vue.js"}}</script>');
  } else if (pathname === "/vue.js") {
    response.setHeader("Content-Type", "text/javascript");
    response.end(vue);
  } else if (subjects.has(pathname.slice(1, -3))) {
    response.setHeader("Content-Type", "text/javascript");
    response.end(
      `const state=globalThis.__childState;\n${subjects.get(pathname.slice(1, -3))}\nexport {App};`,
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
const capture = {
  schema: 1,
  input,
  browserVersion: browser.version(),
  modules: Object.fromEntries(subjects),
  routes: {} as Record<string, unknown>,
  wholeCorpusQualified: false,
  issue6890Complete: false,
};
try {
  for (const name of subjects.keys()) {
    const page = await browser.newPage();
    const errors: string[] = [];
    page.on("pageerror", (error) => errors.push(String(error)));
    try {
      await page.goto(`http://127.0.0.1:${address.port}/`);
      capture.routes[name] = await page.evaluate(
        async ({ name, original }) => {
          const entry = "/vue.js";
          const Vue = (await import(entry)) as typeof import("vue");
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
                throw Error(name);
            }
          };
          let reads = 0;
          const reactive = Vue.shallowReactive({ value: value(original.initial.value.$jsxValue) });
          const state = new Proxy(reactive, {
            get(target, key, receiver) {
              if (key === "value") reads++;
              return Reflect.get(target, key, receiver);
            },
          });
          (globalThis as unknown as { __childState: typeof state }).__childState = state;
          const module = await import(`/${name}.js`);
          const app = Vue.createApp({ setup: () => () => module.App() });
          const warnings: string[] = [];
          app.config.warnHandler = (message) => warnings.push(message);
          app.config.errorHandler = (error) => warnings.push(String(error));
          const host = document.createElement("div");
          document.body.append(host);
          const ids = new WeakMap<object, number>();
          let next = 0;
          const observe = (parent: Node): unknown[] => {
            const result: unknown[] = [];
            for (const node of parent.childNodes) {
              if (node.nodeType === 3 && node.textContent) {
                const previous = result.at(-1);
                if (typeof previous === "string")
                  result[result.length - 1] = previous + node.textContent;
                else result.push(node.textContent);
              } else if (node.nodeType === 1) {
                const element = node as Element;
                result.push({
                  tag: element.localName,
                  attributes: Object.fromEntries(
                    [...element.attributes]
                      .map((attr) => [attr.name, attr.value])
                      .sort(([a], [b]) => a.localeCompare(b)),
                  ),
                  children: observe(node),
                });
              }
            }
            return result;
          };
          let mounted = false;
          try {
            app.mount(host);
            mounted = true;
            await Vue.nextTick();
            const phases = [];
            for (const [index, phase] of [
              original.initial,
              ...original.steps.map((step) => step.patch),
            ].entries()) {
              if (index) {
                state.value = value(phase.value.$jsxValue);
                await Vue.nextTick();
              }
              phases.push({
                dom: observe(host),
                text: host.textContent,
                elementIdentities: [...host.querySelectorAll("*")].map((element) => {
                  if (!ids.has(element)) ids.set(element, next++);
                  return ids.get(element);
                }),
                warnings: [...warnings],
                reads,
              });
            }
            app.unmount();
            mounted = false;
            await Vue.nextTick();
            const before = reads;
            state.value = "after-unmount";
            await Vue.nextTick();
            if (reads !== before) throw Error("render survived unmount");
            return {
              phases,
              readsAfterUnmount: reads,
              completeUnmount: host.childNodes.length === 0,
            };
          } finally {
            if (mounted) app.unmount();
            host.remove();
          }
        },
        { name, original },
      );
    } catch (error) {
      capture.routes[name] = { error: String(error) };
    } finally {
      if (errors.length) capture.routes[name] = { route: capture.routes[name], pageErrors: errors };
      await page.close();
    }
  }
  type Trace = {
    phases: { text: string; elementIdentities: number[]; warnings: string[]; reads: number }[];
    readsAfterUnmount: number;
    completeUnmount: boolean;
  };
  for (const suffix of ["", "-shadow"]) {
    const reference = capture.routes["reference" + suffix] as Trace;
    const native = capture.routes["native" + suffix] as Trace;
    for (const trace of [reference, native]) {
      assert.ok(trace.phases, JSON.stringify(trace));
      assert.equal(trace.phases.length, 10);
      assert.equal(trace.readsAfterUnmount, 10);
      assert.equal(trace.completeUnmount, true);
      trace.phases.forEach((phase, index) => {
        assert.equal(phase.text, original.expectedText[index]);
        assert.deepEqual(phase.elementIdentities, original.expectedElementIdentities[index]);
        assert.deepEqual(phase.warnings, []);
        assert.equal(phase.reads, index + 1);
      });
    }
    assert.deepEqual(native, reference, "complete genuine Chromium native/Babel traces");
  }
} finally {
  fs.writeFileSync(outputPath, JSON.stringify(capture, null, 2) + "\n");
  await browser.close();
  await new Promise<void>((resolve, reject) =>
    server.close((error) => (error ? reject(error) : resolve())),
  );
}
console.log(
  JSON.stringify({
    routes: subjects.size,
    nativeVdomPhases: 10,
    wholeCorpusQualified: false,
    issue6890Complete: false,
  }),
);
