// Infrastructure/reference only: this never loads the plugin, native addon or a browser.
import assert from "node:assert/strict";
import fs from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";
import { pathToFileURL } from "node:url";
import { wholeError, nodes } from "../hosts/contracts.ts";
import {
  runtimeSpecs,
  runtimeIdentity,
  hostSpecs,
  here,
  json,
  hash,
  exists,
} from "./environment.ts";
import { freshMetrics, source } from "./fixtures.ts";
import { prepareReference } from "./reference.ts";
import type { FileResult, PluginTarget } from "@babel/core";
import type { BabelAst, BabelCore, WholeError } from "../host-types.ts";
import type { RuntimeMetrics } from "./fixtures.ts";
import type { RuntimeIdentity, RuntimeModule, GeneratedFile } from "./types.ts";
interface InfrastructureRuntime {
  id: string;
  identity: RuntimeIdentity | null;
  exports: { version: string; functions: Record<string, string> } | null;
  reference: GeneratedFile[] | null;
  html: string | null;
  metrics: RuntimeMetrics | null;
  logs: { type: string; args?: string[]; error?: WholeError }[];
  failure: WholeError | null;
  playwright?: { chromiumLaunch: string };
}
interface InfrastructureHost {
  host: (typeof hostSpecs)[number];
  source: string;
  stripped: FileResult | null;
  failure: WholeError | null;
}

const output = path.resolve(
  process.env.VIZE_BABEL_RUNTIME_INFRA_OUTPUT ??
    path.join(here, ".artifacts/infrastructure-" + Date.now()),
);
assert(!exists(output));
fs.mkdirSync(output, { recursive: true });
const report = {
  scope:
    "actual runtime API/reference/compiler/TSX-parser infrastructure only; zero packed-plugin, source-native or Chromium execution",
  runtime: [] as InfrastructureRuntime[],
  hosts: [] as InfrastructureHost[],
};
for (const spec of runtimeSpecs) {
  const row: InfrastructureRuntime = {
    id: spec.id,
    identity: null,
    exports: null,
    reference: null,
    html: null,
    metrics: null,
    logs: [],
    failure: null,
  };
  report.runtime.push(row);
  const warn = console.warn,
    error = console.error,
    info = console.info;
  try {
    row.identity = runtimeIdentity(spec);
    console.warn = (...args) => row.logs.push({ type: "warning", args: args.map(String) });
    console.error = (...args) => row.logs.push({ type: "error", args: args.map(String) });
    console.info = (...args) => row.logs.push({ type: "info", args: args.map(String) });
    const browserApi: typeof import("vue") & Record<string, unknown> = await import(
      pathToFileURL(row.identity.browserEntry).href
    );
    const names =
      spec.target === "vapor"
        ? [
            "createApp",
            "createSSRApp",
            "createVaporApp",
            "createVaporSSRApp",
            "defineVaporComponent",
          ]
        : ["createApp", "createSSRApp", "defineComponent"];
    row.exports = {
      version: browserApi.version,
      functions: Object.fromEntries(names.map((name) => [name, typeof browserApi[name]])),
    };
    assert.equal(browserApi.version, spec.version);
    assert(names.every((name) => typeof browserApi[name] === "function"));
    const key = hash(Buffer.from(output)).slice(0, 16),
      directory = path.join(row.identity.root, "node_modules/.vize-babel-reference", key);
    assert(!exists(directory));
    fs.mkdirSync(directory, { recursive: true });
    row.reference = prepareReference(row.identity, directory);
    const requireRuntime = createRequire(path.join(row.identity.root, "package.json"));
    const playwright = requireRuntime("playwright");
    row.playwright = { chromiumLaunch: typeof playwright.chromium?.launch };
    assert.equal(row.playwright.chromiumLaunch, "function", "public CommonJS Playwright host API");
    const Vue: typeof import("vue") = await import(
      pathToFileURL(requireRuntime.resolve("vue")).href
    );
    const renderer: { renderToString(app: ReturnType<typeof Vue.createSSRApp>): Promise<string> } =
      await import(pathToFileURL(requireRuntime.resolve("@vue/server-renderer")).href);
    globalThis.__vizeBabelRuntime = freshMetrics();
    const module: RuntimeModule = await import(
      pathToFileURL(path.join(directory, "reference-server.mjs")).href
    );
    const app = Vue.createSSRApp(module.App);
    app.config.warnHandler = (message) => row.logs.push({ type: "warning", args: [message] });
    app.config.errorHandler = (error) => row.logs.push({ type: "error", error: wholeError(error) });
    row.html = await renderer.renderToString(app);
    row.metrics = structuredClone(globalThis.__vizeBabelRuntime);
    assert.deepEqual(row.metrics, {
      setup: { App: 1, Counter: 1 },
      childEvents: 0,
      parentEvents: 0,
      events: [],
    });
    assert(row.html.includes("雪 🧭:0") && row.html.includes('id="counter-initial"'));
    assert(row.html.includes('data-probe="雪 🧭"'));
    assert(row.logs.every((log) => log.type === "info"));
    fs.cpSync(directory, path.join(output, spec.id + "-reference"), { recursive: true });
  } catch (error) {
    row.failure = wholeError(error);
  } finally {
    console.warn = warn;
    console.error = error;
    console.info = info;
  }
}
for (const spec of hostSpecs) {
  const row: InfrastructureHost = { host: spec, source, stripped: null, failure: null };
  report.hosts.push(row);
  try {
    const requireHost = createRequire(
      path.resolve(here, "../hosts/fixtures/babel" + spec.major + "/package.json"),
    );
    const core: BabelCore = await import(pathToFileURL(requireHost.resolve("@babel/core")).href);
    const typescript: PluginTarget = (
      await import(pathToFileURL(requireHost.resolve("@babel/plugin-transform-typescript")).href)
    ).default;
    assert.equal(core.version, spec.version);
    const result = core.transformSync(source, {
      filename: "Counter.tsx",
      babelrc: false,
      configFile: false,
      ast: true,
      parserOpts: { plugins: ["jsx", "typescript"] },
      plugins: [[typescript, { isTSX: true }]],
    });
    assert(result);
    assert(!nodes(result.ast).some((node) => node.type.startsWith("TS")));
    for (const name of ["Counter", "App"]) {
      const declaration: BabelAst | undefined = nodes(result.ast).find(
        (node) =>
          node.type === "VariableDeclarator" &&
          node.id.type === "Identifier" &&
          node.id.name === name,
      );
      assert(
        declaration?.type === "VariableDeclarator" &&
          declaration.init?.type === "ArrowFunctionExpression",
      );
      assert.equal(declaration.init.body.type, "BlockStatement");
    }
    row.stripped = result;
  } catch (error) {
    row.failure = wholeError(error);
  }
}
json(path.join(output, "infrastructure.json"), report);
assert(
  report.runtime.every((row) => row.failure === null) &&
    report.hosts.every((row) => row.failure === null),
  "inspect all infrastructure/reference packets at " + output,
);
process.stdout.write(
  JSON.stringify({
    output,
    runtimeReferenceControls: report.runtime.length,
    actualHostParserControls: report.hosts.length,
    packedNativeBrowserCredit: false,
  }) + "\n",
);
