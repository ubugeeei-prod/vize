/**
 * Vapor lane of the composable conformance check (child process).
 *
 * Resolves `vue` to the Vue 3.6 runtime-with-vapor build for this process,
 * so both the compiled Vapor probes and the composables they call run on the
 * Vapor-capable runtime. Each probe hydrates the server HTML produced by the
 * VDOM SSR lane with `createVaporSSRApp`; warnings, a replaced root, or a
 * differing client state fail the probe.
 */
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { createRequire, registerHooks } from "node:module";
import { pathToFileURL } from "node:url";

import { captureConsole, installDomGlobals, settle } from "./dom-env.ts";
import { readProbeState, type VaporLaneJob, type VaporLaneResult } from "./probe-sfc.ts";

const require = createRequire(import.meta.url);
const vaporVueUrl = pathToFileURL(
  require.resolve("vue-vapor-runtime/dist/vue.runtime-with-vapor.esm-browser.js"),
).href;
registerHooks({
  resolve(specifier, context, nextResolve) {
    if (specifier === "vue") return { url: vaporVueUrl, shortCircuit: true };
    return nextResolve(specifier, context);
  },
});

const jobs = JSON.parse(readFileSync(process.argv[2] ?? "", "utf8")) as VaporLaneJob[];
const domWindow = installDomGlobals();
const { messages: bootMessages } = await captureConsole(async () => {
  await import("vue");
});
void bootMessages;
const vue: Record<string, unknown> = await import("vue");
const createVaporSSRApp = vue.createVaporSSRApp;
assert.equal(typeof createVaporSSRApp, "function", "Vue runtime has no createVaporSSRApp");

interface MountableApp {
  mount(host: Element): unknown;
  unmount(): void;
}

function isApp(value: unknown): value is MountableApp {
  return (
    typeof value === "object" &&
    value !== null &&
    typeof Reflect.get(value, "mount") === "function" &&
    typeof Reflect.get(value, "unmount") === "function"
  );
}

const results: VaporLaneResult[] = [];
for (const job of jobs) {
  const problems: string[] = [];
  domWindow.localStorage.clear();
  domWindow.sessionStorage.clear();
  const host = document.createElement("div");
  host.innerHTML = job.html;
  document.body.append(host);
  const serverRoot = host.firstElementChild;
  let app: MountableApp | undefined;
  try {
    const module: unknown = await import(pathToFileURL(job.module).href);
    const component: unknown =
      typeof module === "object" && module !== null ? Reflect.get(module, "default") : undefined;
    const { messages } = await captureConsole(async () => {
      const created: unknown = Reflect.apply(
        createVaporSSRApp as (...args: unknown[]) => unknown,
        undefined,
        [component],
      );
      assert.ok(isApp(created), "createVaporSSRApp returned no app");
      created.mount(host);
      app = created;
      await settle();
    });
    if (messages.length > 0) problems.push(messages.join("\n"));
    if (host.firstElementChild !== serverRoot)
      problems.push("Vapor hydration replaced the server root");
    const state = readProbeState(host.innerHTML);
    if (job.client !== null) {
      try {
        assert.deepEqual(state, job.client);
      } catch {
        problems.push(`client state ${JSON.stringify(state)} !== ${JSON.stringify(job.client)}`);
      }
    }
  } catch (error) {
    problems.push(error instanceof Error ? (error.stack ?? error.message) : String(error));
  } finally {
    try {
      app?.unmount();
    } catch (error) {
      problems.push(`unmount failed: ${String(error)}`);
    }
    host.remove();
  }
  results.push({ subpath: job.subpath, problems });
}

process.stdout.write(`\n@@RESULTS@@\n${JSON.stringify(results)}`);
process.exit(0);
