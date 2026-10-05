// Real module importers and untouched glob syntax; no output/key normalization.
import assert from "node:assert/strict";
import fs from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import { verifyNuxtSourceBindingEvents } from "./source-binding.mjs";

const [project, artifacts, inputs, cohortJson] = process.argv.slice(2);
const cohort = JSON.parse(cohortJson);
const corpus = JSON.parse(fs.readFileSync(path.join(inputs, "corpus.json"), "utf8"));
const require = createRequire(path.join(project, "package.json"));
for (const [name, version] of [
  ["vite", cohort.vite],
  ["vue", cohort.vue],
  ["@vitejs/plugin-vue", cohort.pluginVue],
])
  assert.equal(require(`${name}/package.json`).version, version);
if (cohort.nuxt) assert.equal(require("nuxt/package.json").version, cohort.nuxt);
const root = fileURLToPath(new URL("../../../../", import.meta.url));
const { chromium } = createRequire(path.join(root, "tests/package.json"))("@playwright/test");
const host = path.join(project, "runtime-api.mjs");
fs.writeFileSync(
  host,
  'export { build, preview } from "vite";\nexport { default as vue } from "@vitejs/plugin-vue";\nexport { createSSRApp } from "vue";\nexport { renderToString } from "vue/server-renderer";\n' +
    (cohort.nuxt ? 'export { loadNuxt } from "nuxt";\n' : ""),
);
const api = await import(pathToFileURL(host).href);
const json = (directory, name, value) =>
  fs.writeFileSync(
    path.join(directory, name),
    JSON.stringify(value, (_key, item) => (item instanceof Set ? [...item] : item), 2) + "\n",
  );
const binding = JSON.parse(fs.readFileSync(process.env.VIZE_NUXT_NATIVE_CUSTODY, "utf8"));
const events = () =>
  fs.existsSync(binding.calls)
    ? fs
        .readFileSync(binding.calls, "utf8")
        .trim()
        .split("\n")
        .map((line) => JSON.parse(line))
    : [];
const originalFiles = binding.fixtures.map((fixture) => fixture.filename);
const observations = [];
let browser;
try {
  browser = await chromium.launch();
  for (const rootKind of corpus.roots) {
    const buildRoot = rootKind === "src" ? path.join(project, "src") : project;
    let nuxt;
    try {
      if (cohort.nuxt) {
        nuxt = await api.loadNuxt({ cwd: project, dev: false });
        assert.equal(nuxt.options.rootDir, project);
        assert.equal(path.resolve(nuxt.options.srcDir), path.join(project, "src"));
        assert.equal(
          nuxt.options.vite.plugins
            .flat(Infinity)
            .filter((plugin) => plugin.name === "vite-plugin-vize").length,
          1,
        );
      }
      for (const scenario of corpus.scenarios) {
        for (const backend of ["client", "ssr"]) {
          for (const adapter of ["stock", "source"]) {
            const directory = path.join(artifacts, `${rootKind}-${scenario}-${backend}-${adapter}`);
            fs.mkdirSync(directory);
            const before = events().length;
            const loads = [];
            let configFile = false,
              plugins;
            if (cohort.nuxt) {
              plugins =
                adapter === "stock" ? [api.vue()] : nuxt.options.vite.plugins.flat(Infinity);
              plugins = plugins.map((plugin) => {
                if (plugin.name !== "vite-plugin-vize") return plugin;
                const original =
                  typeof plugin.load === "function" ? plugin.load : plugin.load.handler;
                const observed = async function (...args) {
                  const result = await original.apply(this, args);
                  if (result && originalFiles.some((source) => args[0].includes(source)))
                    loads.push({ args, result });
                  return result;
                };
                return {
                  ...plugin,
                  load:
                    typeof plugin.load === "function"
                      ? observed
                      : { ...plugin.load, handler: observed },
                };
              });
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
            const output = Array.isArray(result)
              ? result.flatMap((item) => item.output)
              : result.output;
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
            let runtime;
            if (ssr) {
              fs.symlinkSync(
                path.join(project, "node_modules"),
                path.join(directory, "dist/node_modules"),
              );
              try {
                const { default: component } = await import(
                  pathToFileURL(path.join(directory, "dist/entry.mjs")).href
                );
                const context = {};
                const html = await api.renderToString(api.createSSRApp(component), context);
                runtime = {
                  html,
                  modules: [...(context.modules ?? [])].sort((a, b) =>
                    a < b ? -1 : a > b ? 1 : 0,
                  ),
                };
                json(directory, "runtime.json", runtime);
                assert.equal(html, corpus.expected[scenario].ssrHtml);
              } finally {
                fs.unlinkSync(path.join(directory, "dist/node_modules"));
              }
            } else {
              let server, page;
              const pageErrors = [],
                consoleMessages = [],
                responses = [];
              try {
                server = await api.preview({
                  root: buildRoot,
                  configFile: false,
                  logLevel: "warn",
                  build: { outDir: path.join(directory, "dist") },
                  preview: { host: "127.0.0.1", port: 0, strictPort: false },
                });
                page = await browser.newPage();
                page.on("pageerror", (error) => pageErrors.push(String(error)));
                page.on("console", (message) =>
                  consoleMessages.push({ type: message.type(), text: message.text() }),
                );
                page.on("response", (response) =>
                  responses.push({ url: response.url(), status: response.status() }),
                );
                const address = server.httpServer.address();
                assert.ok(address && typeof address === "object");
                const response = await page.goto(`http://127.0.0.1:${address.port}/${entry}`);
                assert.equal(response.status(), 200);
                await page.waitForFunction(() =>
                  document.querySelector("#app")?.hasAttribute("data-v-app"),
                );
                runtime = {
                  appOuterHtml: await page.locator("#app").evaluate((element) => element.outerHTML),
                  documentHtml: await page.content(),
                  pageErrors,
                  consoleMessages,
                  responses,
                };
                json(directory, "runtime.json", runtime);
                assert.equal(
                  runtime.appOuterHtml,
                  `<div id="app" data-v-app="">${corpus.expected[scenario].innerHtml}</div>`,
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
  `Complete original #7936 ${cohort.id} root/src-root source+stock client/browser/SSR controls passed`,
);
