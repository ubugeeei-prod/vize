import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const root = fileURLToPath(new URL("../../../", import.meta.url));
const fixture = process.argv[2];
assert.ok(fixture);
const nuxtPackage = path.join(root, "npm/framework/nuxt/package.json");
const requireNuxt = createRequire(nuxtPackage);
const runtimeManifest = requireNuxt.resolve("nuxt/package.json");
assert.equal(JSON.parse(fs.readFileSync(runtimeManifest)).version, "4.5.1");
const requireRuntime = createRequire(runtimeManifest);
const { resolveModulePath } = await import(pathToFileURL(requireRuntime.resolve("exsolve")).href);
const esm = (name, from) => resolveModulePath(name, { from, conditions: ["node", "import"] });
const nuxtEntry = esm("nuxt", nuxtPackage);
const kitEntry = esm("@nuxt/kit", nuxtEntry);
const devtoolsEntry = esm("@nuxt/devtools", nuxtEntry);
const requireDevtools = createRequire(devtoolsEntry);
const gitEntry = esm("simple-git", devtoolsEntry);
const requireGit = createRequire(gitEntry);
const manifest = (require, name) => {
  let directory = path.dirname(fs.realpathSync(require.resolve(name)));
  while (directory !== path.dirname(directory)) {
    const file = path.join(directory, "package.json");
    if (fs.existsSync(file)) {
      const value = JSON.parse(fs.readFileSync(file));
      if (value.name === name) return value;
    }
    directory = path.dirname(directory);
  }
  throw new Error(`missing actual package manifest: ${name}`);
};
assert.equal(manifest(createRequire(nuxtEntry), "@nuxt/kit").version, "4.5.1");
assert.equal(manifest(requireDevtools, "simple-git").version, "4.0.1");
assert.equal(manifest(requireGit, "@simple-git/argv-parser").version, "2.0.1");
assert.equal(manifest(requireGit, "@simple-git/args-pathspec").version, "1.0.4");
fs.symlinkSync(
  path.join(root, "npm/framework/nuxt/node_modules"),
  path.join(fixture, "node_modules"),
  "dir",
);
fs.writeFileSync(
  path.join(fixture, ".gitignore"),
  "node_modules/\n.nuxt/\n.output/\n.data/\n.devtools/\n.cache/\n",
);
fs.writeFileSync(
  path.join(fixture, "nuxt.config.mjs"),
  `export default {
  devtools: { enabled: true, vueDevTools: false, viteInspect: false, componentInspector: false },
  telemetry: false, compatibilityDate: "2026-10-05", test: false
};\n`,
);
fs.writeFileSync(path.join(fixture, "tracked.txt"), "original\n");
const git = (...args) => execFileSync("git", args, { cwd: fixture, encoding: "utf8" }).trim();
git("init", "--initial-branch=security-compat");
git(
  "-c",
  "user.name=Security compatibility",
  "-c",
  "user.email=security@example.invalid",
  "add",
  ".",
);
git(
  "-c",
  "user.name=Security compatibility",
  "-c",
  "user.email=security@example.invalid",
  "commit",
  "-m",
  "fixture",
);
const expected = `security-compat#${git("rev-parse", "--short", "HEAD")}`;
const { loadNuxt } = await import(pathToFileURL(kitEntry).href);
let initialized = false;
const nuxt = await loadNuxt({
  cwd: fixture,
  dev: true,
  ready: true,
  overrides: {
    hooks: {
      "devtools:initialized": () => {
        initialized = true;
      },
    },
  },
});
const results = [];
try {
  assert.equal(initialized, true, "genuine Devtools initialization must run");
  const rpc = nuxt.devtools?.rpc?.functions;
  assert.equal(typeof rpc?.generateAnalyzeBuildName, "function");
  const check = async (suffix) => {
    const actual = await rpc.generateAnalyzeBuildName();
    assert.equal(actual, expected + suffix, "timestamp fallback is not compatibility");
    results.push(actual);
  };
  await check("");
  fs.writeFileSync(path.join(fixture, "tracked.txt"), "modified\n");
  await check("-dirty");
  git("checkout", "--", "tracked.txt");
  await check("");
  fs.writeFileSync(path.join(fixture, "untracked.txt"), "untracked\n");
  await check("-dirty");
  fs.unlinkSync(path.join(fixture, "untracked.txt"));
  await check("");
} finally {
  await nuxt.close();
}
const requireExample = createRequire(path.join(root, "npm/builder/vite/example/package.json"));
const vueEntry = requireExample.resolve("vue");
const requireVue = createRequire(vueEntry);
const ssrEntry = requireVue.resolve("@vue/server-renderer");
assert.equal(manifest(requireExample, "vue").version, "3.5.42");
assert.equal(manifest(requireVue, "@vue/server-renderer").version, "3.5.42");
const { createSSRApp, h } = await import(pathToFileURL(vueEntry).href);
const { renderToString } = await import(pathToFileURL(ssrEntry).href);
const render = (attrs) => renderToString(createSSRApp({ render: () => h("div", attrs, "pass") }));
const safe = await render({ safe: "ok" });
const unsafe = await render({ "x\rautofocus\ronfocus": "alert(1)" });
assert.equal(safe, '<div safe="ok">pass</div>');
assert.equal(unsafe, "<div>pass</div>");
console.log(
  JSON.stringify({
    node: process.versions.node,
    nuxtEntry,
    kitEntry,
    devtoolsEntry,
    gitEntry,
    results,
    vueEntry,
    ssrEntry,
    safe,
    unsafe,
  }),
);
