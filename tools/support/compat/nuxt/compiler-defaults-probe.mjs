// Actual Nuxt module setup -> actual Vite builds; no synthetic configResolved.
import assert from "node:assert/strict";
import { createRequire } from "node:module";
import fs from "node:fs";
import path from "node:path";
import { pathToFileURL } from "node:url";
import { verifyNuxtSourceBindingEvents } from "./source-binding.mjs";

const [project, artifacts, inputs, version, scenarioId] = process.argv.slice(2);
const corpus = JSON.parse(fs.readFileSync(path.join(inputs, "corpus.json"), "utf8"));
const require = createRequire(path.join(project, "package.json"));
assert.equal(require("nuxt/package.json").version, version);
assert.equal(require("vue/package.json").version, "3.5.43");
const host = path.join(project, "runtime-api.mjs");
fs.writeFileSync(
  host,
  'export { loadNuxt } from "nuxt";\nexport { build } from "vite";\nexport { default as vize } from "@vizejs/vite-plugin";\nexport { createSSRApp } from "vue";\nexport { renderToString } from "vue/server-renderer";\n',
);
const { loadNuxt, build, vize, createSSRApp, renderToString } = await import(
  pathToFileURL(host).href
);
const json = (directory, name, value) =>
  fs.writeFileSync(
    path.join(directory, name),
    JSON.stringify(value, (_key, item) => (item instanceof Set ? [...item] : item), 2) + "\n",
  );
const sources = corpus.paths.map((name) => path.join(project, name));
const sourceRoot = path.join(project, "app");
const configFile = path.join(project, "vize.config.json");
const originalConfig = fs.readFileSync(path.join(inputs, "vize.config.json.txt"));
const originalNuxt = fs.readFileSync(path.join(project, "nuxt.config.ts"));

function recordLoads(plugins, rows) {
  return plugins.map((plugin) => {
    if (plugin.name !== "vite-plugin-vize") return plugin;
    const original = typeof plugin.load === "function" ? plugin.load : plugin.load.handler;
    const observed = async function (...args) {
      const result = await original.apply(this, args);
      if (result && sources.some((source) => args[0].includes(source))) rows.push({ args, result });
      return result;
    };
    return {
      ...plugin,
      load: typeof plugin.load === "function" ? observed : { ...plugin.load, handler: observed },
    };
  });
}
async function compile(directory, plugins, ssr) {
  const loads = [];
  const result = await build({
    root: project,
    configFile: false,
    plugins: recordLoads(plugins, loads),
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
      ssr: ssr ? sources[2] : false,
      ...(ssr ? {} : { lib: { entry: sources[2], formats: ["es"] } }),
      rollupOptions: {
        external: ["vue", "vue/server-renderer", "@vue/server-renderer"],
        output: { entryFileNames: "entry.mjs" },
      },
    },
  });
  const output = Array.isArray(result) ? result.flatMap((item) => item.output) : result.output;
  json(directory, "loads.json", loads);
  json(directory, "bundle.json", output);
  assert.equal(output.length, 1, "all original imports must form one complete runnable module");
  assert.equal(output[0].type, "chunk");
  fs.writeFileSync(path.join(directory, "entry.mjs"), output[0].code);
  // ESM resolution must use the actual pinned project dependencies.
  fs.symlinkSync(path.join(project, "node_modules"), path.join(directory, "node_modules"));
  try {
    const { default: component } = await import(
      pathToFileURL(path.join(directory, "entry.mjs")).href
    );
    assert.ok(component, "the whole original app must remain the default component");
    const context = {};
    const html = await renderToString(createSSRApp(component), context);
    const rendered = {
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
const sortedLoads = (rows) => [...rows].sort((a, b) => a.args[0].localeCompare(b.args[0]));
const observations = [];
const selected = corpus.scenarios.filter((scenario) => scenario.id === scenarioId);
assert.equal(selected.length, 1, "execute exactly one frozen scenario per fresh process");
const binding = JSON.parse(fs.readFileSync(process.env.VIZE_NUXT_NATIVE_CUSTODY, "utf8"));
const readEvents = () =>
  fs.existsSync(binding.calls)
    ? fs
        .readFileSync(binding.calls, "utf8")
        .trim()
        .split("\n")
        .map((line) => JSON.parse(line))
    : [];
const results = (events) =>
  events
    .filter((event) => event.kind === "call")
    .map((event) => {
      assert.equal(event.entrypoint, "compileSfc");
      assert.equal(event.outcome, "return");
      assert.ok(sources.includes(event.args[1].filename));
      assert.equal(event.args[0], fs.readFileSync(event.args[1].filename, "utf8"));
      assert.deepEqual(event.result.errors, []);
      assert.deepEqual(event.result.warnings, []);
      return { filename: event.args[1].filename, result: event.result };
    })
    .sort((a, b) => a.filename.localeCompare(b.filename));
for (const scenario of selected) {
  const directory = path.join(artifacts, scenario.id);
  fs.mkdirSync(directory);
  if (scenario.config === null) fs.rmSync(configFile, { force: true });
  else
    fs.writeFileSync(
      configFile,
      scenario.config === "original"
        ? originalConfig
        : JSON.stringify(scenario.config, null, 2) + "\n",
    );
  if (scenario.forward) process.env.FORWARD = "1";
  else delete process.env.FORWARD;
  let nuxt;
  try {
    nuxt = await loadNuxt({
      cwd: project,
      dev: false,
      overrides: {
        srcDir: sourceRoot,
        ...(scenario.nuxtWhitespace
          ? { vue: { compilerOptions: { whitespace: scenario.nuxtWhitespace } } }
          : {}),
        vize: { compiler: scenario.module, lint: false, checker: false, musea: false },
      },
    });
    assert.equal(nuxt.options.rootDir, project);
    assert.equal(path.resolve(nuxt.options.srcDir), sourceRoot);
    const plugins = nuxt.options.vite.plugins.flat(Infinity);
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
        assert.equal(event.args[1].ssr, ssr);
        assert.equal(
          event.args[1].whitespace ?? "condense",
          scenario.expected[sources.indexOf(event.args[1].filename)],
        );
      }
      const literal = sources.flatMap((filename, index) =>
        vize({
          ...scenario.module,
          root: project,
          configMode: false,
          scanPatterns: [],
          include: [filename],
          whitespace: scenario.expected[index],
          nuxtPageMeta: true,
          ssrModuleIdRoot: sourceRoot,
        }),
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
          ? corpus.paths
              .map((name) => name.slice(4))
              .sort((left, right) => (left < right ? -1 : left > right ? 1 : 0))
          : [],
      };
      assert.deepEqual(observed.rendered, expected);
      assert.deepEqual(control.rendered, expected);
      observations.push({
        scenario: scenario.id,
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
const custody = JSON.parse(fs.readFileSync(process.env.VIZE_NUXT_NATIVE_CUSTODY, "utf8"));
const events = fs
  .readFileSync(custody.calls, "utf8")
  .trim()
  .split("\n")
  .map((line) => JSON.parse(line));
json(path.join(artifacts, scenarioId), "proof.json", {
  issue: 7959,
  version,
  vue: require("vue/package.json").version,
  observations,
  sourceBinding: verifyNuxtSourceBindingEvents(custody, events),
  scope:
    "real Nuxt setup -> actual Vite client/SSR compilation -> complete Vue SSR rendering; no Nitro generate, hydration or performance claim",
});
console.log(`Complete original #7959 ${version} setup/client/SSR/render controls passed`);
