/**
 * Renderer conformance for every `@vizejs/composable` catalog entry.
 *
 * For each probe in `src/conformance/probes.ts` (one per catalog entry,
 * enforced here) the probe SFC is compiled through the SSR, DOM, and Vapor
 * lanes of `@vizejs/native`, then:
 *
 * 1. server-rendered twice with `vue/server-renderer` while browser globals
 *    are absent and Node's host globals are trapped (output must be identical);
 * 2. hydrated into a happy-dom document with `createSSRApp().mount()` —
 *    any Vue warning, hydration mismatch, or replaced root fails the probe;
 * 3. hydrated again as a Vapor component with Vue 3.6 `createVaporSSRApp`
 *    in a child process (see `conformance/vapor-lane.ts`).
 *
 * Expected server and post-activation client states declared by the probe
 * are asserted exactly. Requires a built `@vizejs/native` binding.
 */
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { mkdirSync, rmSync, writeFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

import { compileSfc } from "@vizejs/native";

import { COMPOSABLE_CATALOG } from "../src/catalog.ts";
import type { ComponentProbe } from "../src/conformance/probe-types.ts";
import { composableProbes } from "../src/conformance/probes.ts";
import {
  captureConsole,
  installDomGlobals,
  settle,
  withoutBrowserGlobals,
} from "./conformance/dom-env.ts";
import {
  buildProbeSfc,
  probeFileName,
  readProbeState,
  type VaporLaneJob,
  type VaporLaneResult,
} from "./conformance/probe-sfc.ts";

const packageRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const outputRoot = path.join(packageRoot, ".conformance", String(process.pid));
const only = new Set(process.argv.slice(2));

const failures: string[] = [];
const fail = (subpath: string, lane: string, message: string): void => {
  failures.push(`${subpath} [${lane}] ${message}`);
};

// 1. Coverage: every catalog entry has exactly one probe and vice versa
// (skipped when the run is filtered to explicit subpaths).
const entrySubpaths = COMPOSABLE_CATALOG.entries.map((entry) => entry.subpath);
for (const subpath of only.size > 0 ? [] : entrySubpaths) {
  if (!(subpath in composableProbes))
    fail(subpath, "coverage", "catalog entry has no conformance probe");
}
for (const subpath of Object.keys(composableProbes)) {
  if (!entrySubpaths.some((entry) => entry === subpath))
    fail(subpath, "coverage", "probe has no catalog entry");
}

// 2. Compile every component probe through the three renderer lanes.
rmSync(outputRoot, { recursive: true, force: true });
mkdirSync(outputRoot, { recursive: true });
interface CompiledProbe {
  readonly subpath: `./${string}`;
  readonly probe: ComponentProbe;
  readonly ssr: string;
  readonly dom: string;
  readonly vapor: string;
}
const compiled: CompiledProbe[] = [];
for (const [key, probe] of Object.entries(composableProbes)) {
  const subpath = key as `./${string}`;
  if (probe.kind !== "component") continue;
  if (only.size > 0 && !only.has(subpath)) continue;
  const source = buildProbeSfc(probe, path.join(packageRoot, "src"));
  const filename = probeFileName(subpath);
  const files: Record<string, string> = {};
  let ok = true;
  for (const lane of ["ssr", "dom", "vapor"] as const) {
    const result = compileSfc(source, {
      filename,
      isTs: true,
      mode: "module",
      ssr: lane === "ssr",
      vapor: lane === "vapor",
    });
    if (result.errors.length > 0 || result.warnings.length > 0) {
      fail(subpath, `compile:${lane}`, [...result.errors, ...result.warnings].join("\n"));
      ok = false;
      continue;
    }
    const isVapor = /defineVaporComponent|__vaporRender/.test(result.code);
    if (isVapor !== (lane === "vapor")) {
      fail(subpath, `compile:${lane}`, `unexpected ${isVapor ? "Vapor" : "VDOM"} output`);
      ok = false;
    }
    const file = path.join(outputRoot, `${filename.replace(/\.vue$/, "")}.${lane}.ts`);
    writeFileSync(file, result.code.replaceAll('"@vue/server-renderer"', '"vue/server-renderer"'));
    files[lane] = file;
  }
  if (ok && files.ssr && files.dom && files.vapor) {
    compiled.push({ subpath, probe, ssr: files.ssr, dom: files.dom, vapor: files.vapor });
  }
}

// 3. Server render + VDOM hydration in a happy-dom document.
const domWindow = installDomGlobals();
const vue = await import("vue");
const { renderToString } = await import("vue/server-renderer");
const { withTrappedServerGlobals } = await import("../src/testing/ssr-harness.ts");

function isComponent(value: unknown): value is Parameters<typeof vue.createSSRApp>[0] {
  return typeof value === "object" && value !== null;
}

async function loadComponent(file: string): Promise<Parameters<typeof vue.createSSRApp>[0]> {
  const module: unknown = await import(pathToFileURL(file).href);
  const component: unknown =
    typeof module === "object" && module !== null ? Reflect.get(module, "default") : undefined;
  assert.ok(isComponent(component), `${file} has no default component export`);
  return component;
}

const vaporJobs: VaporLaneJob[] = [];
for (const entry of compiled) {
  const { subpath, probe } = entry;
  domWindow.localStorage.clear();
  domWindow.sessionStorage.clear();
  let html: string;
  try {
    const ssrComponent = await loadComponent(entry.ssr);
    const rendered = await captureConsole(() =>
      withoutBrowserGlobals(() =>
        withTrappedServerGlobals(async () => {
          const first = await renderToString(vue.createSSRApp(ssrComponent));
          const second = await renderToString(vue.createSSRApp(ssrComponent));
          assert.equal(second, first, "server rendering is not deterministic");
          return first;
        }),
      ),
    );
    if (rendered.messages.length > 0) fail(subpath, "ssr", rendered.messages.join("\n"));
    html = rendered.result;
  } catch (error) {
    fail(subpath, "ssr", error instanceof Error ? (error.stack ?? error.message) : String(error));
    continue;
  }
  const serverState = readProbeState(html);
  if (probe.server !== undefined) {
    try {
      assert.deepEqual(serverState, probe.server);
    } catch {
      fail(
        subpath,
        "ssr",
        `server state ${JSON.stringify(serverState)} !== ${JSON.stringify(probe.server)}`,
      );
    }
  }

  const host = document.createElement("div");
  host.innerHTML = html;
  document.body.append(host);
  const serverRoot = host.firstElementChild;
  let app: ReturnType<typeof vue.createSSRApp> | undefined;
  let mounted = false;
  try {
    const domComponent = await loadComponent(entry.dom);
    const { messages } = await captureConsole(async () => {
      app = vue.createSSRApp(domComponent);
      app.mount(host);
      mounted = true;
      await settle();
    });
    if (messages.length > 0) fail(subpath, "hydrate", messages.join("\n"));
    if (host.firstElementChild !== serverRoot)
      fail(subpath, "hydrate", "hydration replaced the server root");
    const clientState = readProbeState(host.innerHTML);
    if (probe.client !== undefined) {
      try {
        assert.deepEqual(clientState, probe.client);
      } catch {
        fail(
          subpath,
          "hydrate",
          `client state ${JSON.stringify(clientState)} !== ${JSON.stringify(probe.client)}`,
        );
      }
    }
  } catch (error) {
    fail(
      subpath,
      "hydrate",
      error instanceof Error ? (error.stack ?? error.message) : String(error),
    );
  } finally {
    if (mounted) app?.unmount();
    host.remove();
  }
  if (probe.vaporSkip === undefined) {
    vaporJobs.push({ subpath, module: entry.vapor, html, client: probe.client ?? null });
  }
}

// 4. Vapor hydration in a child process that resolves `vue` to Vue 3.6.
const jobsFile = path.join(outputRoot, "vapor-jobs.json");
writeFileSync(jobsFile, JSON.stringify(vaporJobs));
const child = spawnSync(
  process.execPath,
  [path.join(packageRoot, "scripts/conformance/vapor-lane.ts"), jobsFile],
  { cwd: packageRoot, encoding: "utf8", maxBuffer: 64 * 1024 * 1024 },
);
if (child.status !== 0) {
  fail("*", "vapor", `vapor lane crashed (${String(child.status)}):\n${child.stderr}`);
} else {
  const results = JSON.parse(
    child.stdout.slice(child.stdout.indexOf("\n@@RESULTS@@\n") + 13),
  ) as VaporLaneResult[];
  for (const result of results) {
    for (const problem of result.problems) fail(result.subpath, "vapor", problem);
  }
}

const componentProbes = compiled.length;
const exemptProbes = Object.values(composableProbes).filter(
  (probe) => probe.kind === "exempt",
).length;
console.log(
  JSON.stringify({
    check: "@vizejs/composable renderer conformance",
    entries: entrySubpaths.length,
    componentProbes,
    exemptProbes,
    vaporExecuted: vaporJobs.length,
    lanes: ["ssr", "hydrate", "vapor-compile", "vapor-hydrate"],
    failures: failures.length,
  }),
);
rmSync(outputRoot, { recursive: true, force: true });
if (failures.length > 0) {
  console.error(failures.join("\n\n"));
  process.exit(1);
}
process.exit(0);
