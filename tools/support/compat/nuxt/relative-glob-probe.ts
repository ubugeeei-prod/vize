// Real module importers and untouched glob syntax; no output/key normalization.
import assert from "node:assert/strict";
import fs from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";
import { fileURLToPath } from "node:url";
import {
  asArray,
  asRecord,
  asString,
  asStringArray,
  at,
  parseJson,
  readJsonRecord,
} from "../../json-value.ts";
import {
  bundleOutput,
  importChecked,
  importDefault,
  loadChromium,
  previewPort,
  recordVizeLoads,
  viteRuntimeFunctions,
  vitePlugin,
} from "./host-types.ts";
import type {
  Browser,
  BrowserGlobals,
  LoadRow,
  Page,
  PreviewServer,
  VitePlugin,
  ViteRuntimeHost,
} from "./host-types.ts";
import { readSourceBindingCustody, verifyNuxtSourceBindingEvents } from "./source-binding.ts";

interface NuxtInstance {
  options: Record<string, unknown>;
  close(): Promise<void>;
}
interface RelativeGlobHost extends ViteRuntimeHost {
  vue(): unknown;
  loadNuxt?: (options: Record<string, unknown>) => Promise<NuxtInstance>;
}

const [project, artifacts, inputs, cohortJson] = process.argv.slice(2);
assert.ok(project && artifacts && inputs && cohortJson, "usage: PROJECT ARTIFACTS INPUTS COHORT");
const cohort = asRecord(parseJson(cohortJson), "cohort");
const corpus = readJsonRecord(path.join(inputs, "corpus.json"));
const expected = asRecord(corpus.expected, "corpus.expected");
const require = createRequire(path.join(project, "package.json"));
for (const [name, version] of [
  ["vite", cohort.vite],
  ["vue", cohort.vue],
  ["@vitejs/plugin-vue", cohort.pluginVue],
] as const)
  assert.equal(at(require(`${name}/package.json`), "version"), version);
if (cohort.nuxt) assert.equal(at(require("nuxt/package.json"), "version"), cohort.nuxt);
const root = fileURLToPath(new URL("../../../../", import.meta.url));
const chromium = loadChromium(root);
const host = path.join(project, "runtime-api.mjs");
fs.writeFileSync(
  host,
  'export { build, preview } from "vite";\nexport { default as vue } from "@vitejs/plugin-vue";\nexport { createSSRApp } from "vue";\nexport { renderToString } from "vue/server-renderer";\n' +
    (cohort.nuxt ? 'export { loadNuxt } from "nuxt";\n' : ""),
);
const api = await importChecked<RelativeGlobHost>(host, [
  ...viteRuntimeFunctions,
  "vue",
  ...(cohort.nuxt ? ["loadNuxt"] : []),
]);
const json = (directory: string, name: string, value: unknown) =>
  fs.writeFileSync(
    path.join(directory, name),
    JSON.stringify(value, (_key, item) => (item instanceof Set ? [...item] : item), 2) + "\n",
  );
const custodyFile = process.env.VIZE_NUXT_NATIVE_CUSTODY;
assert.ok(custodyFile, "VIZE_NUXT_NATIVE_CUSTODY names the custody file");
const binding = readSourceBindingCustody(custodyFile);
const events = (): unknown[] =>
  fs.existsSync(binding.calls)
    ? fs
        .readFileSync(binding.calls, "utf8")
        .trim()
        .split("\n")
        .map((line): unknown => parseJson(line))
    : [];
const originalFiles = binding.fixtures.map((fixture) => fixture.filename);
const nuxtPlugins = (nuxt: NuxtInstance): VitePlugin[] =>
  asArray(at(nuxt.options, "vite", "plugins"), "vite.plugins")
    .flat(Infinity)
    .map(vitePlugin);
const scenarioHtml = (scenario: string, key: string): string =>
  asString(at(expected, scenario, key), `expected.${scenario}.${key}`);
const observations: unknown[] = [];
let browser: Browser | undefined;
try {
  const launched = await chromium.launch();
  browser = launched;
  for (const rootKind of asStringArray(corpus.roots, "corpus.roots")) {
    const buildRoot = rootKind === "src" ? path.join(project, "src") : project;
    let nuxt: NuxtInstance | undefined;
    try {
      if (cohort.nuxt) {
        assert.ok(api.loadNuxt, "the Nuxt cohort host exports loadNuxt");
        nuxt = await api.loadNuxt({ cwd: project, dev: false });
        assert.equal(nuxt.options.rootDir, project);
        assert.equal(
          path.resolve(asString(nuxt.options.srcDir, "srcDir")),
          path.join(project, "src"),
        );
        assert.equal(
          nuxtPlugins(nuxt).filter((plugin) => plugin.name === "vite-plugin-vize").length,
          1,
        );
      }
      for (const scenario of asStringArray(corpus.scenarios, "corpus.scenarios")) {
        for (const backend of ["client", "ssr"]) {
          for (const adapter of ["stock", "source"]) {
            const directory = path.join(artifacts, `${rootKind}-${scenario}-${backend}-${adapter}`);
            fs.mkdirSync(directory);
            const before = events().length;
            const loads: LoadRow[] = [];
            let configFile: string | false = false,
              plugins: VitePlugin[] | undefined;
            if (nuxt) {
              plugins = recordVizeLoads(
                adapter === "stock" ? [vitePlugin(api.vue())] : nuxtPlugins(nuxt),
                loads,
                originalFiles,
              );
            } else {
              if (adapter === "source") process.env.USE_VIZE = "1";
              else delete process.env.USE_VIZE;
              configFile = path.join(
                project,
                rootKind === "src" ? "vite.src-root.config.ts" : "vite.config.ts",
              );
            }
            const entry = scenario === "plain" ? "index.html" : `${scenario}.html`;
            const ssr = backend === "ssr";
            const result = await api.build({
              root: buildRoot,
              configFile,
              ...(plugins ? { plugins } : {}),
              logLevel: "warn",
              build: {
                minify: false,
                emptyOutDir: true,
                outDir: path.join(directory, "dist"),
                ssr: ssr ? path.join(project, `src/ssr-${scenario}.ts`) : false,
                rollupOptions: {
                  ...(ssr
                    ? { external: ["vue", "vue/server-renderer", "@vue/server-renderer"] }
                    : { input: path.join(buildRoot, entry) }),
                  output: ssr
                    ? { entryFileNames: "entry.mjs", chunkFileNames: "[name]-[hash].mjs" }
                    : {},
                },
              },
            });
            const output = bundleOutput(result);
            json(directory, "bundle.json", output);
            json(directory, "loads.json", loads);
            json(directory, "native-events.json", events().slice(before));
            json(directory, "configuration.json", {
              rootKind,
              buildRoot,
              configFile,
              scenario,
              backend,
              adapter,
              cohort,
            });
            assert.ok(output.length > 0);
            let runtime: Record<string, unknown> | undefined;
            if (ssr) {
              fs.symlinkSync(
                path.join(project, "node_modules"),
                path.join(directory, "dist/node_modules"),
              );
              try {
                const component = await importDefault(path.join(directory, "dist/entry.mjs"));
                const context: { modules?: Iterable<string> } = {};
                const html = await api.renderToString(api.createSSRApp(component), context);
                runtime = {
                  html,
                  modules: [...(context.modules ?? [])].sort((a, b) =>
                    a < b ? -1 : a > b ? 1 : 0,
                  ),
                };
                json(directory, "runtime.json", runtime);
                assert.equal(html, scenarioHtml(scenario, "ssrHtml"));
              } finally {
                fs.unlinkSync(path.join(directory, "dist/node_modules"));
              }
            } else {
              let server: PreviewServer | undefined, page: Page | undefined;
              const pageErrors: string[] = [],
                consoleMessages: { type: string; text: string }[] = [],
                responses: { url: string; status: number }[] = [];
              try {
                server = await api.preview({
                  root: buildRoot,
                  configFile: false,
                  logLevel: "warn",
                  build: { outDir: path.join(directory, "dist") },
                  preview: { host: "127.0.0.1", port: 0, strictPort: false },
                });
                const opened = await launched.newPage();
                page = opened;
                opened.on("pageerror", (error) => pageErrors.push(String(error)));
                opened.on("console", (message) =>
                  consoleMessages.push({ type: message.type(), text: message.text() }),
                );
                opened.on("response", (response) =>
                  responses.push({ url: response.url(), status: response.status() }),
                );
                const response = await opened.goto(
                  `http://127.0.0.1:${previewPort(server)}/${entry}`,
                );
                assert.equal(response?.status(), 200);
                await opened.waitForFunction(() =>
                  (globalThis as unknown as BrowserGlobals).document
                    .querySelector("#app")
                    ?.hasAttribute("data-v-app"),
                );
                runtime = {
                  appOuterHtml: await opened
                    .locator("#app")
                    .evaluate((element) => element.outerHTML),
                  documentHtml: await opened.content(),
                  pageErrors,
                  consoleMessages,
                  responses,
                };
                json(directory, "runtime.json", runtime);
                assert.equal(
                  runtime.appOuterHtml,
                  `<div id="app" data-v-app="">${scenarioHtml(scenario, "innerHtml")}</div>`,
                );
                assert.deepEqual(pageErrors, []);
                assert.deepEqual(
                  consoleMessages.filter((message) => message.type === "error"),
                  [],
                );
                assert.ok(responses.every((response) => response.status < 400));
              } finally {
                json(directory, "browser-session.json", { pageErrors, consoleMessages, responses });
                try {
                  await page?.close();
                } finally {
                  await server?.close();
                }
              }
            }
            observations.push({
              rootKind,
              scenario,
              backend,
              adapter,
              wholeBundleOutputs: output.length,
              runtime,
            });
          }
        }
      }
    } finally {
      await nuxt?.close();
    }
  }
} finally {
  await browser?.close();
}
for (const fixture of binding.fixtures)
  assert.equal(fs.readFileSync(fixture.filename, "utf8"), fixture.source);
json(artifacts, "proof.json", {
  issue: 7936,
  cohort,
  observations,
  sourceBinding: verifyNuxtSourceBindingEvents(binding, events()),
  scope: corpus.scope,
});
assert.equal(observations.length, 24);
console.log(
  `Complete original #7936 ${String(cohort.id)} root/src-root source+stock client/browser/SSR controls passed`,
);
