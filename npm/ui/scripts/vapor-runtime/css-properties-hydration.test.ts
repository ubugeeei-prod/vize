import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync, realpathSync } from "node:fs";
import { createRequire } from "node:module";
import test from "node:test";
import { pathToFileURL } from "node:url";

import { compileSfc } from "@vizejs/native";

import { installDomGlobals, withoutBrowserGlobals } from "./dom-env.ts";

const require = createRequire(import.meta.url);
const stableCompiler = require("vue/compiler-sfc") as typeof import("vue/compiler-sfc");
const originalCompiler = require("vue-vapor-runtime/compiler-sfc") as typeof stableCompiler;
const supportedCompiler =
  require("vue-vapor-runtime-supported/compiler-sfc") as typeof stableCompiler;
const stable = require("vue") as typeof import("vue");
const server = require("vue/server-renderer") as typeof import("vue/server-renderer");
const window = installDomGlobals();
const digest = (value: string): string => createHash("sha256").update(value).digest("hex");
const fixtureRoot = new URL(
  "../../../../tests/_fixtures/differential/compiler/vapor-css-properties-hydration/",
  import.meta.url,
);
const packet = JSON.parse(readFileSync(new URL("fixture.json", fixtureRoot), "utf8")) as {
  originalVue: string;
  supportedVue: string;
  inputs: { path: string; sha256: string }[];
};
let sequence = 0;

function runtimeUrl(name: string): string {
  return pathToFileURL(
    realpathSync(require.resolve(`${name}/dist/vue.runtime-with-vapor.esm-browser.js`)),
  ).href;
}

function reference(source: string, filename: string, vapor: boolean, original = false): string {
  const compiler = vapor ? (original ? originalCompiler : supportedCompiler) : stableCompiler;
  const { descriptor, errors } = compiler.parse(source, { filename });
  assert.deepEqual(errors, []);
  return compiler.compileScript(descriptor, {
    id: "custom-properties-hydration",
    inlineTemplate: true,
    templateOptions: { ssr: !vapor },
    vapor,
  } as Parameters<typeof compiler.compileScript>[1]).content;
}

async function evaluate(code: string, vueUrl: string): Promise<import("vue").Component> {
  const routed = code
    .replaceAll(/from ["']vue["']/g, `from ${JSON.stringify(vueUrl)}`)
    .replaceAll(
      /from ["'](?:@vue\/server-renderer|vue\/server-renderer)["']/g,
      `from ${JSON.stringify(pathToFileURL(require.resolve("vue/server-renderer")).href)}`,
    );
  return (
    await import(
      `data:text/javascript;base64,${Buffer.from(`${routed}\n// ${sequence++}`).toString("base64")}`
    )
  ).default as import("vue").Component;
}

await test("Vapor hydrates custom properties across a VDOM parent with the supported runtime", async () => {
  assert.equal(
    originalCompiler.version,
    packet.originalVue,
    "immutable reference remains installed",
  );
  assert.equal(supportedCompiler.version, packet.supportedVue);
  const originalUrl = runtimeUrl("vue-vapor-runtime");
  const supportedUrl = runtimeUrl("vue-vapor-runtime-supported");
  const original = (await import(originalUrl)) as typeof import("vue");
  const supported = (await import(supportedUrl)) as typeof import("vue");
  assert.equal(original.version, originalCompiler.version);
  assert.equal(supported.version, supportedCompiler.version);
  const observations: unknown[] = [];
  const completeReference = new Map<
    string,
    { hydrated: string; updated: string; unmounted: string }
  >();
  try {
    for (const name of ["static", "dynamic"]) {
      const filename = `${name}.vue`;
      const source = readFileSync(
        new URL(
          `../../../../tests/_fixtures/differential/compiler/vapor-css-properties-hydration/${filename}.txt`,
          import.meta.url,
        ),
        "utf8",
      );
      assert.equal(
        digest(source),
        packet.inputs.find((input) => input.path === `${filename}.txt`)?.sha256,
      );
      const nativeSsr = compileSfc(source, { filename, mode: "module", ssr: true });
      const nativeVapor = compileSfc(source, { filename, mode: "module", vapor: true });
      assert.deepEqual(nativeSsr.errors, []);
      assert.deepEqual(nativeVapor.errors, []);
      const stableUrl = pathToFileURL(require.resolve("vue")).href;
      const ssrOutputs: string[] = [];
      for (const code of [reference(source, filename, false), nativeSsr.code]) {
        const component = await evaluate(code, stableUrl);
        ssrOutputs.push(
          await withoutBrowserGlobals(() => server.renderToString(stable.createSSRApp(component))),
        );
      }
      assert.deepEqual(
        ssrOutputs,
        [ssrOutputs[0], ssrOutputs[0]],
        "whole native/official SSR agrees",
      );
      assert.match(ssrOutputs[0] ?? "", /^<div style="--percent:50%;?">50%<\/div>$/);
      for (const [backend, code] of [
        ["vue-rc9", reference(source, filename, true, true)],
        ["vue-supported", reference(source, filename, true)],
        ["native", nativeVapor.code],
      ]) {
        assert.ok(code !== undefined);
        const runs: unknown[] = [];
        for (const [runtime, url, expectedFailure] of [
          [original, originalUrl, true],
          [supported, supportedUrl, false],
        ] as const) {
          const component = await evaluate(code, url);
          const root = runtime.defineComponent({ setup: () => () => runtime.h(component) });
          const app = runtime.createSSRApp(root);
          const host = document.createElement("div");
          host.innerHTML = ssrOutputs[0] ?? "";
          document.body.append(host);
          const originalRoot = host.firstElementChild;
          const diagnostics: string[] = [];
          const warn = console.warn;
          const error = console.error;
          console.warn = (...values: unknown[]) => diagnostics.push(values.map(String).join(" "));
          console.error = (...values: unknown[]) => diagnostics.push(values.map(String).join(" "));
          app.use(runtime.vaporInteropPlugin);
          let mounted = false;
          try {
            if (expectedFailure) {
              assert.throws(
                () => app.mount(host),
                /Cannot read properties of undefined \(reading 'nodes'\)/,
              );
              runs.push({ runtime: runtime.version, status: "reproduced-original-failure" });
              continue;
            }
            app.mount(host);
            mounted = true;
            await runtime.nextTick();
            const element = host.firstElementChild;
            assert.ok(element instanceof HTMLElement);
            assert.equal(element, originalRoot, "server root must be adopted");
            assert.equal(element.tagName, "DIV");
            assert.equal(element.attributes.length, 1);
            assert.equal(element.style.length, 1);
            assert.equal(element.style.getPropertyValue("--percent"), "50%");
            assert.equal(element.textContent, "50%");
            const hydrated = host.innerHTML;
            element.click();
            await runtime.nextTick();
            assert.equal(
              host.firstElementChild,
              originalRoot,
              "reactive updates retain the hydrated root",
            );
            assert.equal(element.style.length, 1);
            assert.equal(element.style.getPropertyValue("--percent"), "75%");
            assert.equal(element.textContent, "75%");
            const updated = host.innerHTML;
            app.unmount();
            mounted = false;
            await runtime.nextTick();
            assert.equal(host.childNodes.length, 0, "unmount removes all nodes and anchors");
            assert.deepEqual(diagnostics, []);
            const complete = { hydrated, updated, unmounted: host.innerHTML };
            if (backend === "vue-rc9") completeReference.set(name, complete);
            else
              assert.deepEqual(
                complete,
                completeReference.get(name),
                "whole native/official DOM agrees",
              );
            runs.push({
              runtime: runtime.version,
              status: "pass",
              hydrated,
              updated,
              unmounted: host.innerHTML,
            });
          } finally {
            console.warn = warn;
            console.error = error;
            if (mounted) app.unmount();
            host.remove();
            const reset: unknown = Reflect.get(runtime, "setInsertionState");
            if (typeof reset === "function") reset(null);
          }
        }
        observations.push({
          name,
          backend,
          sourceSha256: digest(source),
          moduleSha256: digest(code),
          ssrOutputs,
          runs,
        });
      }
    }
    console.log(JSON.stringify({ check: "Vapor custom-properties hydration", observations }));
  } finally {
    await window.happyDOM.close();
  }
});
