// Actual Nuxt module setup -> actual Vite builds; no synthetic configResolved.
import assert from "node:assert/strict";
import { createRequire } from "node:module";
import fs from "node:fs";
import path from "node:path";
import { inlineVuePluginFixtureOptions } from "../../../../tests/tooling/support/vue-config-fixture.ts";
import {
  asArray,
  asRecord,
  asString,
  asStringArray,
  at,
  parseJson,
  readJsonRecord,
} from "../../json-value.ts";
import type { JsonRecord } from "../../json-value.ts";
import {
  bundleOutput,
  importChecked,
  importDefault,
  recordVizeLoads,
  vitePlugin,
} from "./host-types.ts";
import type { LoadRow, VitePlugin } from "./host-types.ts";
import { readSourceBindingCustody, verifyNuxtSourceBindingEvents } from "./source-binding.ts";

interface NuxtInstance {
  options: JsonRecord;
  close(): Promise<void>;
}
interface CompilerDefaultsHost {
  loadNuxt: (options: Record<string, unknown>) => Promise<NuxtInstance>;
  build: (config: Record<string, unknown>) => Promise<unknown>;
  vize: (options: Record<string, unknown>) => unknown;
  createSSRApp: (component: unknown) => unknown;
  renderToString: (
    application: unknown,
    context?: { modules?: Iterable<string> },
  ) => Promise<string>;
}
interface Scenario {
  id: string;
  forward: boolean;
  nuxtWhitespace?: string;
  config: JsonRecord | string | null;
  module: JsonRecord;
  expected: string[];
  html: string;
}
function parseScenario(value: unknown): Scenario {
  const scenario = asRecord(value, "scenario");
  const { config, nuxtWhitespace } = scenario;
  assert.ok(nuxtWhitespace === undefined || typeof nuxtWhitespace === "string");
  return {
    id: asString(scenario.id, "scenario.id"),
    forward: scenario.forward === true,
    nuxtWhitespace,
    config:
      typeof config === "string" || config === null ? config : asRecord(config, "scenario.config"),
    module: asRecord(scenario.module, "scenario.module"),
    expected: asStringArray(scenario.expected, "scenario.expected"),
    html: asString(scenario.html, "scenario.html"),
  };
}
interface Rendered {
  html: string;
  modules: string[];
}

const [project, artifacts, inputs, version, scenarioId] = process.argv.slice(2);
assert.ok(
  project && artifacts && inputs && version && scenarioId,
  "usage: PROJECT ARTIFACTS INPUTS VERSION SCENARIO",
);
const corpus = readJsonRecord(path.join(inputs, "corpus.json"));
const corpusPaths = asStringArray(corpus.paths, "corpus.paths");
const require = createRequire(path.join(project, "package.json"));
assert.equal(at(require("nuxt/package.json"), "version"), version);
assert.equal(at(require("vue/package.json"), "version"), "3.5.43");
const host = path.join(project, "runtime-api.mjs");
fs.writeFileSync(
  host,
  'export { loadNuxt } from "nuxt";\nexport { build } from "vite";\nexport { default as vize } from "@vizejs/vite-plugin";\nexport { createSSRApp } from "vue";\nexport { renderToString } from "vue/server-renderer";\n',
);
const { loadNuxt, build, vize, createSSRApp, renderToString } =
  await importChecked<CompilerDefaultsHost>(host, [
    "loadNuxt",
    "build",
    "vize",
    "createSSRApp",
    "renderToString",
  ]);
const json = (directory: string, name: string, value: unknown) =>
  fs.writeFileSync(
    path.join(directory, name),
    JSON.stringify(value, (_key, item) => (item instanceof Set ? [...item] : item), 2) + "\n",
  );
const sources = corpusPaths.map((name) => path.join(project, name));
const entry = sources[2];
assert.ok(entry, "the third authored path is the app entry");
const sourceRoot = path.join(project, "app");
const originalConfig = fs.readFileSync(path.join(inputs, "vize.config.json.txt"));
const originalNuxt = fs.readFileSync(path.join(project, "nuxt.config.ts"));

async function compile(directory: string, plugins: readonly VitePlugin[], ssr: boolean) {
  const loads: LoadRow[] = [];
  const result = await build({
    root: project,
    configFile: false,
    plugins: recordVizeLoads(plugins, loads, sources),
    logLevel: "warn",
    define: {
      __VUE_OPTIONS_API__: "true",
      __VUE_PROD_DEVTOOLS__: "false",
      __VUE_PROD_HYDRATION_MISMATCH_DETAILS__: "false",
    },
    build: {
      write: false,
      minify: false,
      sourcemap: false,
      ssr: ssr ? entry : false,
      ...(ssr ? {} : { lib: { entry, formats: ["es"] } }),
      rollupOptions: {
        external: ["vue", "vue/server-renderer", "@vue/server-renderer"],
        output: { entryFileNames: "entry.mjs" },
      },
    },
  });
  const output = bundleOutput(result);
  json(directory, "loads.json", loads);
  json(directory, "bundle.json", output);
  assert.equal(output.length, 1, "all original imports must form one complete runnable module");
  assert.equal(at(output, 0, "type"), "chunk");
  fs.writeFileSync(
    path.join(directory, "entry.mjs"),
    asString(at(output, 0, "code"), "chunk code"),
  );
  // ESM resolution must use the actual pinned project dependencies.
  fs.symlinkSync(path.join(project, "node_modules"), path.join(directory, "node_modules"));
  try {
    const component = await importDefault(path.join(directory, "entry.mjs"));
    assert.ok(component, "the whole original app must remain the default component");
    const context: { modules?: Iterable<string> } = {};
    const html = await renderToString(createSSRApp(component), context);
    const rendered: Rendered = {
      html,
      modules: [...(context.modules ?? [])].sort((left, right) =>
        left < right ? -1 : left > right ? 1 : 0,
      ),
    };
    json(directory, "rendered.json", rendered);
    return { loads, output, rendered };
  } finally {
    fs.unlinkSync(path.join(directory, "node_modules"));
  }
}
const sortedLoads = (rows: readonly LoadRow[]) =>
  [...rows].sort((a, b) => String(a.args[0]).localeCompare(String(b.args[0])));
const observations: unknown[] = [];
const selected = asArray(corpus.scenarios, "corpus.scenarios")
  .map(parseScenario)
  .filter((scenario) => scenario.id === scenarioId);
assert.equal(selected.length, 1, "execute exactly one frozen scenario per fresh process");
const custodyFile = process.env.VIZE_NUXT_NATIVE_CUSTODY;
assert.ok(custodyFile, "VIZE_NUXT_NATIVE_CUSTODY names the custody file");
const binding = readSourceBindingCustody(custodyFile);
const readEvents = (): JsonRecord[] =>
  fs.existsSync(binding.calls)
    ? fs
        .readFileSync(binding.calls, "utf8")
        .trim()
        .split("\n")
        .map((line) => asRecord(parseJson(line), "native event"))
    : [];
const callFilename = (event: JsonRecord): string =>
  asString(at(event, "args", 1, "filename"), "compileSfc filename");
const results = (events: readonly JsonRecord[]) =>
  events
    .filter((event) => event.kind === "call")
    .map((event) => {
      assert.equal(event.entrypoint, "compileSfc");
      assert.equal(event.outcome, "return");
      const filename = callFilename(event);
      assert.ok(sources.includes(filename));
      assert.equal(at(event, "args", 0), fs.readFileSync(filename, "utf8"));
      assert.deepEqual(at(event, "result", "errors"), []);
      assert.deepEqual(at(event, "result", "warnings"), []);
      return { filename, result: event.result };
    })
    .sort((a, b) => a.filename.localeCompare(b.filename));
for (const scenario of selected) {
  const { id, expected: expectedWhitespace } = scenario;
  const directory = path.join(artifacts, id);
  fs.mkdirSync(directory);
  // Scoped compiler entries belong to the JavaScript plugin API. Canonical
  // Vue entries expose native lint scopes; do not claim compiler scopes there.
  const shared = scenario.config === "original" ? originalConfig : scenario.config;
  const compiler = inlineVuePluginFixtureOptions(scenario.module, shared);
  assert.equal(fs.existsSync(path.join(project, "vize.config.json")), false);
  json(directory, "inline-plugin-config.json", compiler);
  if (scenario.forward) process.env.FORWARD = "1";
  else delete process.env.FORWARD;
  let nuxt: NuxtInstance | undefined;
  try {
    nuxt = await loadNuxt({
      cwd: project,
      dev: false,
      overrides: {
        srcDir: sourceRoot,
        ...(scenario.nuxtWhitespace
          ? { vue: { compilerOptions: { whitespace: scenario.nuxtWhitespace } } }
          : {}),
        vize: { compiler, lint: false, checker: false, musea: false },
      },
    });
    assert.equal(nuxt.options.rootDir, project);
    assert.equal(path.resolve(asString(nuxt.options.srcDir, "srcDir")), sourceRoot);
    const plugins = asArray(at(nuxt.options, "vite", "plugins"), "vite.plugins")
      .flat(Infinity)
      .map(vitePlugin);
    assert.equal(plugins.filter((plugin) => plugin.name === "vite-plugin-vize").length, 1);
    json(directory, "nuxt-options.json", {
      version,
      rootDir: nuxt.options.rootDir,
      srcDir: nuxt.options.srcDir,
      vue: nuxt.options.vue,
      vize: nuxt.options.vize,
    });
    for (const ssr of [false, true]) {
      const backend = ssr ? "ssr" : "client";
      const current = path.join(directory, backend),
        reference = path.join(directory, backend + "-explicit-control");
      fs.mkdirSync(current);
      fs.mkdirSync(reference);
      const before = readEvents().length;
      const observed = await compile(current, plugins, ssr);
      const currentEvents = readEvents().slice(before);
      json(current, "native-events.json", currentEvents);
      for (const event of currentEvents.filter((event) => event.kind === "call")) {
        assert.equal(at(event, "args", 1, "ssr"), ssr);
        assert.equal(
          at(event, "args", 1, "whitespace") ?? "condense",
          expectedWhitespace[sources.indexOf(callFilename(event))],
        );
      }
      const literal = sources.flatMap((filename, index) =>
        [
          vize({
            ...scenario.module,
            root: project,
            configMode: false,
            scanPatterns: [],
            include: [filename],
            whitespace: expectedWhitespace[index],
            nuxtPageMeta: true,
            ssrModuleIdRoot: sourceRoot,
          }),
        ]
          .flat()
          .map(vitePlugin),
      );
      const referenceStart = readEvents().length;
      const control = await compile(reference, literal, ssr);
      const controlEvents = readEvents().slice(referenceStart);
      json(reference, "native-events.json", controlEvents);
      assert.equal(results(currentEvents).length, 3);
      assert.equal(results(controlEvents).length, 3);
      assert.deepEqual(results(currentEvents), results(controlEvents));
      // Every whole module/load envelope, code/map/metadata and entire bundle is compared.
      assert.equal(observed.loads.length, 3);
      assert.equal(control.loads.length, 3);
      assert.deepEqual(sortedLoads(observed.loads), sortedLoads(control.loads));
      assert.deepEqual(observed.output, control.output);
      const expected = {
        html: scenario.html,
        modules: ssr
          ? corpusPaths
              .map((name) => name.slice(4))
              .sort((left, right) => (left < right ? -1 : left > right ? 1 : 0))
          : [],
      };
      assert.deepEqual(observed.rendered, expected);
      assert.deepEqual(control.rendered, expected);
      observations.push({
        scenario: id,
        backend,
        completeLoadResults: 3,
        completeBundleOutputs: observed.output.length,
        rendered: observed.rendered,
      });
    }
  } finally {
    await nuxt?.close();
  }
  assert.deepEqual(fs.readFileSync(path.join(project, "nuxt.config.ts")), originalNuxt);
}
const custody = readSourceBindingCustody(custodyFile);
const events = fs
  .readFileSync(custody.calls, "utf8")
  .trim()
  .split("\n")
  .map((line): unknown => parseJson(line));
json(path.join(artifacts, scenarioId), "proof.json", {
  issue: 7959,
  version,
  vue: at(require("vue/package.json"), "version"),
  observations,
  sourceBinding: verifyNuxtSourceBindingEvents(custody, events),
  scope:
    "real Nuxt setup -> actual Vite client/SSR compilation -> complete Vue SSR rendering; no Nitro generate, hydration or performance claim",
});
console.log(`Complete original #7959 ${version} setup/client/SSR/render controls passed`);
