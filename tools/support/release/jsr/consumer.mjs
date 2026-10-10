import assert from "node:assert/strict";
import { mkdtempSync, readFileSync, writeFileSync, mkdirSync } from "node:fs";
import { resolve } from "node:path";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";

const [rawVersion, output] = process.argv.slice(2);
const version = rawVersion?.replace(/^v/, "");
assert.match(version ?? "", /^0\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?$/);
assert.ok(output, "An evidence directory is required");
mkdirSync(output, { recursive: true });
const consumer = mkdtempSync(resolve(output, "consumer-"));
const base = `https://jsr.io/@vizejs/vize/${version}`;
const response = await fetch(`${base}_meta.json`);
assert.ok(response.ok, `Published JSR metadata is unavailable: HTTP ${response.status}`);
const metadata = await response.json();
const sources = {};
for (const [file, dependency] of Object.entries({
  "mod.ts": `npm:vize@${version}`,
  "config.ts": `npm:vize@${version}/config`,
  "native.ts": `npm:@vizejs/native@${version}`,
  "vite.ts": `npm:@vizejs/vite-plugin@${version}`,
})) {
  const sourceResponse = await fetch(`${base}/${file}`);
  assert.ok(sourceResponse.ok, `Published source ${file} is missing`);
  const source = await sourceResponse.text();
  assert.ok(source.includes(JSON.stringify(dependency)), `${file} must pin ${dependency}`);
  const checksum = `sha256-${createHash("sha256").update(source).digest("hex")}`;
  assert.equal(metadata.manifest[`/${file}`].checksum, checksum);
  sources[file] = { dependency, checksum };
}
writeFileSync(resolve(output, "registry-metadata.json"), `${JSON.stringify(metadata, null, 2)}\n`);
writeFileSync(
  resolve(consumer, "package.json"),
  `${JSON.stringify(
    {
      name: "vize-public-jsr-consumer",
      private: true,
      type: "module",
      dependencies: { vite: "8.3.4", vue: "3.5.42" },
      devDependencies: { typescript: "6.0.3", "@types/node": "22" },
    },
    null,
    2,
  )}\n`,
);
function npm(args) {
  const result = spawnSync(process.platform === "win32" ? "npm.cmd" : "npm", args, {
    cwd: consumer,
    stdio: "inherit",
    shell: process.platform === "win32",
    env: {
      ...process.env,
      npm_config_ignore_scripts: "true",
      npm_config_save_exact: "true",
      npm_config_registry: "https://registry.npmjs.org",
    },
  });
  if (result.error) throw result.error;
  assert.equal(result.status, 0, `npm ${args.join(" ")} must succeed`);
}
// Exercise the documented JSR installer, including its npm alias/registry
// configuration. No checkout package, tarball or source import is installed.
npm([
  "exec",
  "--yes",
  "--package=jsr@0.14.3",
  "--",
  "jsr",
  "add",
  "--npm",
  `@vizejs/vize@${version}`,
]);
const manifest = JSON.parse(readFileSync(resolve(consumer, "package.json"), "utf8"));
assert.equal(manifest.dependencies["@vizejs/vize"], `npm:@jsr/vizejs__vize@${version}`);
for (const [packageName, originalName] of [
  ["vize", "vize"],
  ["@vizejs/native", "@vizejs/native"],
  ["@vizejs/vite-plugin", "@vizejs/vite-plugin"],
  ["@vizejs/vize", "@jsr/vizejs__vize"],
]) {
  const installed = JSON.parse(
    readFileSync(resolve(consumer, "node_modules", packageName, "package.json"), "utf8"),
  );
  assert.equal(installed.name, originalName);
  assert.equal(installed.version, version, `${packageName} must be the exact public release`);
}
const typedSource = `import { defineConfig, type VizeConfig } from "@vizejs/vize";
import { loadConfig } from "@vizejs/vize/config";
import { compileSfc, type SfcCompileResultNapi } from "@vizejs/vize/native";
import vize from "@vizejs/vize/vite";
const config: VizeConfig = { compiler: { vapor: false } };
defineConfig(config);
const result: SfcCompileResultNapi = compileSfc("<template><h1>Hello</h1></template>");
void result; void loadConfig; void vize();
`;
writeFileSync(resolve(consumer, "types.ts"), typedSource);
writeFileSync(
  resolve(consumer, "tsconfig.json"),
  JSON.stringify({
    compilerOptions: {
      strict: true,
      noEmit: true,
      skipLibCheck: true,
      target: "ES2022",
      module: "NodeNext",
      moduleResolution: "NodeNext",
    },
    files: ["types.ts"],
  }),
);
npm(["exec", "--", "tsc", "--noEmit", "-p", "tsconfig.json"]);
const runtime = `import assert from "node:assert/strict";
import { writeFileSync } from "node:fs";
import { defineConfig } from "@vizejs/vize";
import { loadConfig } from "@vizejs/vize/config";
import { compileSfc } from "@vizejs/vize/native";
import vize from "@vizejs/vize/vite";
import { build } from "vite";
const config = defineConfig({ compiler: { vapor: false } });
assert.equal(config.compiler.vapor, false);
writeFileSync("vize.config.json", JSON.stringify(config));
const loaded = await loadConfig(process.cwd());
assert.equal(loaded.compiler.vapor, false);
const source = '<script setup lang="ts">const title: string = "JSR consumer"</script><template><h1>{{ title }}</h1></template>';
const compiled = compileSfc(source, { filename: "App.vue", sourceMap: true });
assert.deepEqual(compiled.errors, []);
assert.match(compiled.code, /JSR consumer/);
assert.match(compiled.code, /render/);
assert.ok(compiled.map);
writeFileSync("App.vue", source);
writeFileSync("main.js", 'import { createApp } from "vue"; import App from "./App.vue"; createApp(App).mount("#app");');
writeFileSync("index.html", '<div id="app"></div><script type="module" src="/main.js"></script>');
const bundle = await build({ configFile: false, plugins: [vize()], logLevel: "error", build: { write: false } });
const outputs = (Array.isArray(bundle) ? bundle : [bundle]).flatMap((item) => item.output);
assert.ok(outputs.some((item) => item.type === "chunk" && item.code.includes("JSR consumer")));
writeFileSync("runtime-result.json", JSON.stringify({ compiler: compiled, config: loaded, viteChunks: outputs.filter((item) => item.type === "chunk").map((item) => ({ fileName: item.fileName, code: item.code })) }, null, 2));
`;
writeFileSync(resolve(consumer, "runtime.mjs"), runtime);
const result = spawnSync(process.execPath, ["runtime.mjs"], { cwd: consumer, stdio: "inherit" });
assert.equal(
  result.status,
  0,
  "Published JSR imports, native compilation and Vite build must succeed",
);
const evidence = {
  package: "@vizejs/vize",
  version,
  node: process.version,
  platform: process.platform,
  arch: process.arch,
  sources,
  consumer,
  runtime: JSON.parse(readFileSync(resolve(consumer, "runtime-result.json"), "utf8")),
};
writeFileSync(resolve(output, "consumer-result.json"), `${JSON.stringify(evidence, null, 2)}\n`);
console.log(`Verified published @vizejs/vize@${version} on ${process.platform}/${process.arch}`);
