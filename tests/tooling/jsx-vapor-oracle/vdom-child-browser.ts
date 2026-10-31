/** Actual Chromium before/head VDOM routes, with the original ten values and Babel producer. */
import assert from "node:assert/strict";
import fs from "node:fs";
import { createServer } from "node:http";
import { createRequire } from "node:module";
import { transformSync } from "@babel/core";
import babelJsx from "@vue/babel-plugin-jsx";
import { chromium } from "@playwright/test";

type Packet = {
  original: {
    initial: { value: { $jsxValue: string } };
    steps: { patch: { value: { $jsxValue: string } } }[];
    expectedText: string[];
  };
  cases: { source: string; code: string; functionName: string; tail: string }[];
};
const [basePath, headPath, outputPath] = process.argv.slice(2);
assert.ok(basePath && headPath && outputPath);
const before = JSON.parse(fs.readFileSync(basePath, "utf8")) as Packet;
const after = JSON.parse(fs.readFileSync(headPath, "utf8")) as Packet;
const original = JSON.parse(
  fs.readFileSync(
    new URL(
      "../../_fixtures/differential/compiler/jsx-child-values-6890/original.json",
      import.meta.url,
    ),
    "utf8",
  ),
);
assert.deepEqual(before.original, original);
assert.deepEqual(after.original, original);
assert.equal(before.cases.length, 6);
assert.equal(after.cases.length, 6);
const subjects = new Map<string, string>();
for (const [index, item] of after.cases.entries()) {
  const old = before.cases[index]!;
  for (const key of ["source", "functionName", "tail"] as const) assert.equal(item[key], old[key]);
  const result = transformSync(item.source, {
    configFile: false,
    babelrc: false,
    filename: "original-6890.jsx",
    plugins: [babelJsx],
  });
  assert.equal(typeof result?.code, "string");
  subjects.set(`reference-${index}`, result!.code!);
  subjects.set(`base-${index}`, old.code);
  subjects.set(`head-${index}`, item.code);
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
assert.ok(address && typeof address !== "string");
const browser = await chromium.launch({ headless: true });
const capture = {
  before,
  after,
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
    page.on("pageerror", (e) => errors.push(String(e)));
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
                const e = node as Element;
                result.push({
                  tag: e.localName,
                  attributes: Object.fromEntries(
                    [...e.attributes]
                      .map((a) => [a.name, a.value])
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
                elementIdentities: [...host.querySelectorAll("*")].map((e) => {
                  if (!ids.has(e)) ids.set(e, next++);
                  return ids.get(e);
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
        { name, original: after.original },
      );
    } catch (error) {
      capture.routes[name] = { error: String(error) };
    } finally {
      if (errors.length) capture.routes[name] = { route: capture.routes[name], pageErrors: errors };
      await page.close();
    }
  }
  type Trace = {
    phases: { text: string; warnings: string[]; reads: number }[];
    readsAfterUnmount: number;
    completeUnmount: boolean;
  };
  for (let index = 0; index < 6; index++) {
    const get = (role: string) => {
      const trace = capture.routes[`${role}-${index}`] as Trace;
      assert.ok(trace.phases, JSON.stringify(trace));
      assert.equal(trace.phases.length, 10);
      assert.equal(trace.readsAfterUnmount, 10);
      assert.equal(trace.completeUnmount, true);
      trace.phases.forEach((phase, i) => {
        assert.deepEqual(phase.warnings, []);
        assert.equal(phase.reads, i + 1);
      });
      return trace;
    };
    const reference = get("reference"),
      old = get("base"),
      current = get("head");
    reference.phases.forEach((phase, i) =>
      assert.equal(phase.text, after.original.expectedText[i]! + after.cases[index]!.tail),
    );
    assert.deepEqual(
      old.phases.map((phase, i) => JSON.stringify(phase) === JSON.stringify(reference.phases[i])),
      [true, true, true, true, true, false, false, false, false, true],
      "preserve actual old Chromium failure",
    );
    assert.deepEqual(
      current,
      reference,
      "entire actual Chromium DOM/order/identity/getter/disposal traces",
    );
  }
} finally {
  fs.writeFileSync(outputPath, JSON.stringify(capture, null, 2) + "\n");
  await browser.close();
  await new Promise<void>((resolve, reject) => server.close((e) => (e ? reject(e) : resolve())));
}
console.log(
  JSON.stringify({
    routes: subjects.size,
    phasesPerRoute: 10,
    wholeCorpusQualified: false,
    issue6890Complete: false,
  }),
);
