import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { mkdirSync, mkdtempSync, readFileSync, symlinkSync, writeFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import path from "node:path";

/** Reuse the genuine built packages and installed consumer from the original gate. */
export function checkCanonicalVueConsumer(directory: string, tsc: string, vite: string) {
  const artifactRoot = fileURLToPath(
    new URL("../../../target/differential/vue-config-packed/", import.meta.url),
  );
  mkdirSync(artifactRoot, { recursive: true });
  const evidence = mkdtempSync(path.join(artifactRoot, "consumer-"));
  const write = (name: string, body: string) => {
    writeFileSync(path.join(directory, name), body);
    writeFileSync(path.join(evidence, name), body);
  };
  const capture = (name: string, result: ReturnType<typeof spawnSync>) => {
    writeFileSync(path.join(evidence, `${name}.stdout`), result.stdout ?? "");
    writeFileSync(path.join(evidence, `${name}.stderr`), result.stderr ?? "");
    writeFileSync(
      path.join(evidence, `${name}.outcome.json`),
      JSON.stringify({
        status: result.status,
        signal: result.signal,
        error: result.error?.message,
      }),
    );
  };
  symlinkSync(
    vite,
    path.join(directory, "node_modules/vite"),
    process.platform === "win32" ? "junction" : "dir",
  );
  write(
    "vue-consumer.mts",
    `
import { defineConfig as defineVpConfig, type ConfigEnv, type UserConfig } from "vite-plus";
import { defineConfig as defineViteConfig } from "vite";
import { defineConfig, withVue, withVize, type VueOptions, type VueOptionsEntry } from "@vizejs/vite-plugin/vite-plus";
import type { VueOptions as RootVueOptions, VueOptionsEntry as RootVueOptionsEntry } from "@vizejs/vite-plugin";
declare module "vite-plus" { interface UserConfig { futureOption?: { enabled: boolean }; } }
const vue: VueOptions = {
  version: "0.10", compiler: { compatibility: { hostCompiler: true }, sourceMap: false },
  typeChecker: { enabled: false, strict: false, tsconfig: "tsconfig.json" },
  linter: { enabled: false, preset: "essential", ruleOptions: { "vue/no-mutating-props": { shallowOnly: true } } },
  formatter: { singleQuote: false, attributeGroups: [] },
  languageServer: { enabled: true, hover: false }, globalTypes: { flag: "boolean" },
  files: [], ignores: [], basePath: "src", lib: { dir: "components" },
  entries: [{ files: ["widgets/**/*.vue"], linter: { enabled: false } }],
};
const rootVue: RootVueOptions = vue;
const scope: VueOptionsEntry = { name: "application sources", basePath: "src", files: ["**/*.vue"], ignores: ["generated/**"], linter: { enabled: false } };
const rootScope: RootVueOptionsEntry = scope;
const scopeKeys: Record<keyof VueOptionsEntry, true> = { name: true, basePath: true, files: true, ignores: true, linter: true };
const rootScopeKeys: Record<keyof RootVueOptionsEntry, true> = scopeKeys;
withVue({ entries: [rootScope] }, {});
const vpConfig: UserConfig = { futureOption: { enabled: true }, server: { host: true } };
const callback = (env: ConfigEnv): UserConfig | Promise<UserConfig> => env.isPreview ? vpConfig : Promise.resolve(vpConfig);
defineConfig({ vue: rootVue, futureOption: { enabled: true } });
withVue({}, defineVpConfig(vpConfig));
withVue(vue, defineVpConfig(callback));
withVue(vue, defineVpConfig(async (env) => callback(env)));
withVue(vue, defineVpConfig(Promise.resolve(vpConfig)));
defineViteConfig({ vue });
withVize({ vue: { formatter: { singleQuote: true } } });
// @ts-expect-error Native compiler has no enabled flag.
withVue({ compiler: { enabled: false } }, vpConfig);
// @ts-expect-error Native formatter has no enabled flag.
defineConfig({ vue: { formatter: { enabled: false } } });
// @ts-expect-error Retain the registered native option name.
defineConfig({ vue: { linter: { ruleOptions: { "script/define-props-destructuring": { mode: "always" } } } } });
// @ts-expect-error Keep the actual native dialect vocabulary.
withVue({ version: "4" }, vpConfig);
// @ts-expect-error Consumer Vite Plus still owns unknown option types.
withVue({}, defineVpConfig({ futureOption: { enabled: "wrong" } }));
// @ts-expect-error Library configuration is project-wide.
withVue({ entries: [{ lib: { dir: "components" } }] }, vpConfig);
// @ts-expect-error Vue version is project-wide.
withVue({ entries: [{ version: "3" }] }, vpConfig);
// @ts-expect-error Compiler settings are project-wide.
withVue({ entries: [{ compiler: { sourceMap: true } }] }, vpConfig);
// @ts-expect-error Typechecker settings are project-wide.
withVue({ entries: [{ typeChecker: { enabled: false } }] }, vpConfig);
// @ts-expect-error Formatter settings are project-wide.
withVue({ entries: [{ formatter: { singleQuote: true } }] }, vpConfig);
// @ts-expect-error Language-server settings are project-wide.
withVue({ entries: [{ languageServer: { enabled: true } }] }, vpConfig);
// @ts-expect-error Global declarations are project-wide.
withVue({ entries: [{ globalTypes: { flag: "boolean" } }] }, vpConfig);
`,
  );
  write(
    "tsconfig.vue-consumer.json",
    JSON.stringify({
      compilerOptions: {
        noEmit: true,
        strict: true,
        skipLibCheck: true,
        target: "ES2022",
        module: "NodeNext",
      },
      files: ["vue-consumer.mts"],
    }),
  );
  const types = spawnSync(process.execPath, [tsc, "-p", "tsconfig.vue-consumer.json"], {
    cwd: directory,
    encoding: "utf8",
  });
  capture("types", types);
  assert.equal(types.error, undefined);
  assert.equal(types.status, 0, types.stdout + types.stderr);
  write(
    "vue-consumer.mjs",
    `
import assert from "node:assert/strict";
import { defineConfig as defineVpConfig } from "vite-plus";
import { defineConfig, withVue } from "@vizejs/vite-plugin/vite-plus";
import { resolveConfigExport, resolveViteConfigExport } from "@vizejs/vite-plugin";
const sourceKey = Symbol.for("@vizejs/vite-plugin/vite-plus/source");
const taskKey = Symbol.for("@vizejs/vite-plugin/vite-plus");
const env = { command: "serve", mode: "packed", isSsrBuild: true, isPreview: true };
const options = { plugin: false, tasks: false, conflicts: false, lint: false, fmt: false };
const injected = ["vite-plus:vitest-resolver", "vite-plus:auto-inline-matcher-deps", "vite-plus:coverage-version-guard"];
for (const shape of ["object", "promise", "function", "async"]) {
  let sourceCalls = 0;
  const hook = () => "kept";
  const plugin = { name: "packed-consumer", transform: hook };
  const source = {
    plugins: [plugin], futureOption: { enabled: true }, run: { tasks: { original: "echo original" } },
    test: { projects: [{ test: { name: "nested" }, plugins: [plugin] }] },
    vue: { formatter: { printWidth: 70, singleQuote: true } },
  };
  const callback = (context) => { sourceCalls++; assert.deepEqual(context, env); return source; };
  const selected = defineVpConfig(shape === "object" ? source : shape === "promise" ? Promise.resolve(source) : shape === "function" ? callback : async (context) => callback(context));
  const wrapped = defineConfig(selected, options).withVue({ formatter: { singleQuote: false }, typeChecker: { enabled: false }, linter: { enabled: false } });
  assert.equal(sourceCalls, 0);
  const result = await wrapped(env);
  assert.equal(sourceCalls, shape === "function" || shape === "async" ? 1 : 0);
  assert.deepEqual(result.plugins.map(({name}) => name), [...injected, "packed-consumer"]);
  assert.equal(result.plugins[3], plugin); assert.equal(result.plugins[3].transform, hook);
  assert.deepEqual(result.test.projects[0].plugins.map(({name}) => name), [...injected, "packed-consumer"]);
  assert.equal(result.test.projects[0].plugins[3], plugin);
  assert.deepEqual(result.futureOption, source.futureOption); assert.deepEqual(result.run.tasks, source.run.tasks);
  assert.ok(!("vue" in result)); assert.ok(!("vize" in result));
  const native = await resolveConfigExport(result[taskKey].config, env);
  const direct = await resolveViteConfigExport(result, env, process.cwd());
  assert.equal(typeof result[sourceKey], "object");
  assert.equal(sourceCalls, shape === "function" || shape === "async" ? 1 : 0);
  assert.deepEqual(native, direct);
  assert.deepEqual(native.formatter, { printWidth: 70, singleQuote: false });
  assert.equal(native.typeChecker.enabled, false); assert.equal(native.linter.enabled, false);
  assert.deepEqual(source.plugins, [plugin]);
  let legacyCalls = 0;
  await assert.rejects(withVue({}, { vize: () => { legacyCalls++; return {}; } }, options)(env), /vize: configuration is no longer supported/);
  assert.equal(legacyCalls, 0, "a rejected legacy callback must never execute");
}
`,
  );
  const runtime = spawnSync(process.execPath, ["vue-consumer.mjs"], {
    cwd: directory,
    encoding: "utf8",
    timeout: 30_000,
  });
  capture("runtime", runtime);
  assert.equal(runtime.error, undefined);
  assert.equal(runtime.status, 0, runtime.stdout + runtime.stderr);
  write(
    "vize-vue-compat-control.mjs",
    readFileSync(new URL("./vite-vue-compat-control.mjs", import.meta.url), "utf8"),
  );
  write("vue-build-consumer.mjs", packedBuildConsumer);
  const builds = spawnSync(process.execPath, ["vue-build-consumer.mjs"], {
    cwd: directory,
    encoding: "utf8",
    timeout: 600_000,
    env: { ...process.env, VIZE_VUE_PACKED_EVIDENCE: evidence },
  });
  capture("build-controls", builds);
  assert.equal(builds.error, undefined);
  assert.equal(builds.status, 0, builds.stdout + builds.stderr);
}

// Executed only by the existing packed-consumer Actions gate after its original controls.
const packedBuildConsumer = String.raw`
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { cpSync, existsSync, lstatSync, mkdirSync, readFileSync, readdirSync, realpathSync, rmSync, writeFileSync } from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";
const evidence = process.env.VIZE_VUE_PACKED_EVIDENCE;
assert.ok(evidence);
const require = createRequire(path.join(process.cwd(), "package.json"));
const vpPackage = require.resolve("vite-plus/package.json");
const vitePackage = require.resolve("vite/package.json");
const vpRequire = createRequire(vpPackage);
const backendPackage = vpRequire.resolve("@voidzero-dev/vite-plus-core/package.json");
const packageReceipt = (file) => ({
  path: file, realpath: realpathSync(file),
  sha256: createHash("sha256").update(readFileSync(file)).digest("hex"),
  manifest: JSON.parse(readFileSync(file, "utf8")),
});
const packages = { vp: packageReceipt(vpPackage), vite: packageReceipt(vitePackage), backend: packageReceipt(backendPackage) };
writeFileSync(path.join(evidence, "selected-packages.json"), JSON.stringify(packages, null, 2));
assert.equal(packages.vp.manifest.version, "0.2.9");
assert.equal(packages.vite.manifest.name, "@voidzero-dev/vite-plus-core");
assert.equal(packages.vite.manifest.version, "0.1.24");
// The locked VP CLI backend is a distinct direct dependency; the consumer Vite peer stays unchanged.
assert.equal(packages.backend.manifest.version, "0.2.9");
const vp = path.resolve(path.dirname(vpPackage), packages.vp.manifest.bin.vp);
for (const [name, file] of [["vp-package.json", vpPackage], ["vite-package.json", vitePackage], ["backend-package.json", backendPackage], ["vp-entry", vp]])
  writeFileSync(path.join(evidence, name), readFileSync(file));
const app = "<script setup lang=\"ts\">\nconst greeting = \"PACKED_SCRIPT_AUTHORITY\";\n</script>\n<template><div>PACKED   WHITESPACE<span>{{ greeting }}</span> line\n  break</div></template>\n";
const main = "import { createApp } from \"vue\"; import App from \"./App.vue\"; createApp(App).mount(\"#app\");\n";
for (const [name, body] of [["App.vue", app], ["main.ts", main]]) {
  writeFileSync(name, body);
  writeFileSync(path.join(evidence, name), body);
}
const injected = ["vite-plus:vitest-resolver", "vite-plus:auto-inline-matcher-deps", "vite-plus:coverage-version-guard"];
const results = new Map();
for (const kind of ["legacy", "canonical", "wrapped", "condense"]) {
  const compiler = { sourceMap: true, whitespace: kind === "condense" ? "condense" : "preserve" };
  const policy = JSON.stringify({ compiler });
  const config = [
    'import assert from "node:assert/strict";',
    'import { writeFileSync } from "node:fs";',
    'import { defineConfig as defineVpConfig } from "vite-plus";',
    'import { defineConfig, withVue } from "@vizejs/vite-plugin/vite-plus";',
    'import { assertVizeVueCompatShim } from "./vize-vue-compat-control.mjs";',
    'const forbiddenHost = { name: "vite:vue", transform() { throw new Error("The genuine Vize compiler must replace this host compiler"); } };',
    'const existing = { build: { outDir: "dist-vue-packed", sourcemap: true, minify: false, emptyOutDir: true }, plugins: [',
    'forbiddenHost,',
    '{ name: "packed-build-observer", configResolved(config) {',
    'const names = config.plugins.map(({name}) => name);',
    'writeFileSync("plugins-observed.json", JSON.stringify(names));',
    'assert.equal(names.filter(name => name === "vite-plugin-vize").length, 1);',
    'assert.equal(names.filter(name => name === "packed-build-observer").length, 1);',
    'assertVizeVueCompatShim(config.plugins, forbiddenHost);',
    'for (const name of ' + JSON.stringify(injected) + ') { const count = names.filter(value => value === name).length; assert.ok(count <= 1);' + (kind === "wrapped" ? ' assert.equal(count, 1);' : '') + ' }',
    '} }] };',
    'const integration = { check: false, lint: false, fmt: false, conflicts: false };',
    kind === "legacy" ? 'export default defineConfig({ ...existing, compiler: ' + JSON.stringify(compiler) + ' }, integration);'
      : kind === "wrapped" ? 'export default withVue(' + policy + ', defineVpConfig(existing), integration);'
      : 'export default defineConfig({ ...existing, vue: ' + policy + ' }, integration);',
  ].join("\n");
  const target = path.join(evidence, kind);
  mkdirSync(target);
  writeFileSync("vite.config.ts", config);
  writeFileSync(path.join(target, "vite.config.ts"), config);
  rmSync("dist-vue-packed", { recursive: true, force: true });
  rmSync("plugins-observed.json", { force: true });
  const build = spawnSync(process.execPath, [vp, "run", "build"], {
    timeout: 120_000, env: { ...process.env, NO_COLOR: "1", FORCE_COLOR: "0" },
  });
  writeFileSync(path.join(target, "stdout"), build.stdout ?? "");
  writeFileSync(path.join(target, "stderr"), build.stderr ?? "");
  writeFileSync(path.join(target, "outcome.json"), JSON.stringify({ status: build.status, signal: build.signal, error: build.error?.message, argv: [process.execPath, vp, "run", "build"], cwd: process.cwd() }));
  if (existsSync("dist-vue-packed")) cpSync("dist-vue-packed", path.join(target, "dist"), { recursive: true });
  if (existsSync("plugins-observed.json")) cpSync("plugins-observed.json", path.join(target, "plugins-observed.json"));
  assert.equal(build.error, undefined);
  assert.equal(build.status, 0, String(build.stdout) + String(build.stderr));
  assert.equal(readFileSync("App.vue", "utf8"), app);
  assert.equal(readFileSync("main.ts", "utf8"), main);
  const files = readdirSync("dist-vue-packed", { recursive: true }).map(String).sort().filter(name => {
    const status = lstatSync(path.join("dist-vue-packed", name));
    assert.ok(!status.isSymbolicLink() && (status.isFile() || status.isDirectory()));
    return status.isFile();
  });
  const emitted = files.filter(name => /\.(?:html|css|m?js|map)$/.test(name));
  assert.equal(emitted.length, files.length);
  assert.ok(emitted.includes("index.html"));
  assert.ok(emitted.some(name => /\.m?js$/.test(name)));
  const outputs = emitted.map(name => [name, readFileSync(path.join("dist-vue-packed", name))]);
  const maps = outputs.filter(([name]) => name.endsWith(".map"));
  assert.ok(maps.length > 0);
  const authored = maps.map(([, body]) => JSON.parse(body.toString())).flatMap(map => {
    assert.equal(map.version, 3);
    assert.ok(map.mappings.length > 0);
    return map.sources.flatMap((source, index) => /(?:^|[/\\])App\.vue$/.test(source) ? [{ source, content: map.sourcesContent[index] }] : []);
  });
  assert.ok(authored.length > 0, "The emitted map must identify the authored Vue source");
  for (const { content } of authored) assert.equal(content, app);
  writeFileSync(path.join(target, "whole-output-identities.json"), JSON.stringify(outputs.map(([name, body]) => ({
    name, bytes: body.length, sha256: createHash("sha256").update(body).digest("hex"),
  })), null, 2));
  results.set(kind, outputs);
}
assert.deepEqual(results.get("canonical"), results.get("legacy"));
assert.deepEqual(results.get("wrapped"), results.get("legacy"));
const code = kind => results.get(kind).filter(([name]) => /\.m?js$/.test(name)).map(([, body]) => body.toString()).join("\n");
assert.match(code("canonical"), /PACKED   WHITESPACE/);
assert.match(code("condense"), /PACKED WHITESPACE/);
assert.doesNotMatch(code("condense"), /PACKED   WHITESPACE/);
assert.notDeepEqual(results.get("condense"), results.get("canonical"));
writeFileSync(path.join(evidence, "build-comparison.json"), JSON.stringify({
  legacyCanonicalWholeOutputsEqual: true, actualVpWrappedWholeOutputsEqual: true,
  compilerWhitespaceEffect: true, authoredSourceMaps: true,
}));
`;
