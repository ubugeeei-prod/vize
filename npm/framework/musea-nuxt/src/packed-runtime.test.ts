import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import test from "node:test";
import { pathToFileURL } from "node:url";
import { runCommand } from "../../../../tools/support/release/smoke_release_install/process.ts";

// Normal public providers from the genuine 0.463.0 missing-runtime reproduction.
// These test the advertised peer contract, independently of the workspace Vite alias.
const providers = { vite: "8.3.4", vue: "3.5.35", "vue-router": "4.6.4" };
const providerIntegrity: Record<string, string> = {
  vite: "sha512-IftLE8phJhdNRIGNEV4JCCvfNScoZQVfcXaN8TQropWGJtOHcu+iAB9pHcuGvmEb/kddw0LBe/jS8ztdC6xdiQ==",
  vue: "sha512-cx89fnr+0kVGHiNFG6y6s0bdjypJRFNZn6x3WPstNdQR1bi1mbB7h4v5IBGTsPJU3nK1+0Iqj3Zf+hZWMieR4Q==",
  "vue-router":
    "sha512-Hz9q5sa33Yhduglwz6g9skT8OBPii+4bFn88w6J+J4MfEo4KRRpmiNG/hHHkdbRFlLBOqxN8y8gf2Fb0MTUgVg==",
};

void test("packed Nuxt mocks share their runtime through generated modules and real Vite", () => {
  const packageRoot = path.resolve(import.meta.dirname, "..");
  const temporary = fs.realpathSync.native(
    fs.mkdtempSync(path.join(os.tmpdir(), "vize-musea-nuxt-packed-")),
  );
  try {
    const consumer = path.join(temporary, "consumer ' 日本 % #");
    fs.mkdirSync(consumer);
    const [packed] = JSON.parse(
      runCommand(
        "npm",
        ["pack", "--json", "--ignore-scripts", "--pack-destination", temporary],
        packageRoot,
      ),
    ) as { filename: string; integrity: string; files: { path: string }[] }[];
    const tarball = path.join(temporary, packed.filename);
    assert.equal(
      packed.integrity,
      `sha512-${createHash("sha512").update(fs.readFileSync(tarball)).digest("base64")}`,
    );
    const files = packed.files.map(({ path: filename }) => filename);
    for (const filename of [
      "dist/index.mjs",
      "dist/index.d.mts",
      "dist/auto-imports.mjs",
      "dist/auto-imports.d.mts",
    ])
      assert.ok(files.includes(filename), `${filename} is in the actual tarball`);

    fs.writeFileSync(
      path.join(consumer, "package.json"),
      JSON.stringify({
        name: "musea-nuxt-packed-consumer",
        private: true,
        type: "module",
        dependencies: { "@vizejs/musea-nuxt": pathToFileURL(tarball).href, ...providers },
      }),
    );
    runCommand("npm", ["install", "--no-audit", "--no-fund"], consumer);
    const lock = JSON.parse(fs.readFileSync(path.join(consumer, "package-lock.json"), "utf8")) as {
      packages: Record<string, { version?: string; integrity?: string }>;
    };
    assert.equal(lock.packages["node_modules/@vizejs/musea-nuxt"].integrity, packed.integrity);
    for (const [name, version] of Object.entries(providers)) {
      assert.equal(lock.packages[`node_modules/${name}`].version, version);
      assert.equal(lock.packages[`node_modules/${name}`].integrity, providerIntegrity[name]);
    }
    fs.writeFileSync(path.join(consumer, "check.mjs"), consumerScript);
    const receipt = JSON.parse(runCommand(process.execPath, ["check.mjs"], consumer)) as {
      direct: boolean;
      client: boolean;
      ssr: boolean;
      providers: typeof providers;
    };
    assert.deepEqual(receipt, { direct: true, client: true, ssr: true, providers });
  } finally {
    fs.rmSync(temporary, { recursive: true, force: true });
  }
});

// Run in the fresh consumer without source imports, workspace aliases, loader
// hooks or replacement modules. Default SSR runs before explicit policy controls.
const consumerScript = String.raw`
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import { build } from "vite";
import * as main from "@vizejs/musea-nuxt";
import * as vue from "vue";

const root = process.cwd();
const packageRoot = path.dirname(path.dirname(fileURLToPath(import.meta.resolve("@vizejs/musea-nuxt"))));
assert.equal(fs.realpathSync.native(packageRoot), path.join(root, "node_modules/@vizejs/musea-nuxt"));
const plugin = main.nuxtMusea({
  route: { path: "/packed", query: { tab: "runtime" } },
  runtimeConfig: { public: { apiBase: "/packed-api" } },
  appConfig: { theme: "packed" },
  stateMocks: { count: 7 },
  cookieMocks: { session: "packed-session" },
});
const before = new Map();
function capture(directory) {
  for (const entry of fs.readdirSync(directory, { withFileTypes: true })) {
    const target = path.join(directory, entry.name);
    if (entry.isDirectory()) capture(target);
    else before.set(target, fs.readFileSync(target));
  }
}
capture(packageRoot);
async function generated(request, filename) {
  const id = await plugin.resolveId.call(null, request, undefined, { ssr: false });
  assert.equal(typeof id, "string");
  const source = await plugin.load.call(null, id, { ssr: false });
  assert.equal(typeof source, "string");
  const targets = [...source.matchAll(/from ("(?:[^"\\]|\\.)*")/g)].map((match) => JSON.parse(match[1]));
  assert.ok(targets.length > 0);
  for (const target of targets) {
    const filename = fileURLToPath(import.meta.resolve(target));
    assert.ok(filename.startsWith(packageRoot + path.sep));
    assert.ok(before.has(filename), "generated import exists in the installed tarball: " + target);
  }
  const file = path.join(root, filename);
  fs.writeFileSync(file, source);
  return import(pathToFileURL(file).href);
}
const imports = await generated("#imports", "generated-imports.mjs");
const components = await generated("#components", "generated-components.mjs");
assert.equal(imports.ref, vue.ref);
assert.equal(imports.useRoute, main.useRoute);
assert.equal(imports.useRoute(), main.useRoute());
assert.equal(main.useRoute().fullPath, "/packed?tab=runtime");
assert.equal(imports.useRuntimeConfig(), main.useRuntimeConfig());
assert.equal(main.useRuntimeConfig().public.apiBase, "/packed-api");
assert.equal(imports.useAppConfig(), main.useAppConfig());
assert.equal(main.useAppConfig().theme, "packed");
assert.equal(imports.useState("count"), main.useState("count"));
assert.equal(main.useState("count").value, 7);
assert.equal(imports.useCookie("session"), main.useCookie("session"));
assert.equal(main.useCookie("session").value, "packed-session");
imports.useState("count").value = 19;
assert.equal(main.useState("count").value, 19);
for (const [name, component] of Object.entries(components)) assert.equal(component, main[name], name);
const render = components.NuxtLink.setup({ to: "/from-component" }, { slots: {} });
let prevented = false;
render().props.onClick({ preventDefault() { prevented = true; } });
assert.equal(prevented, true);
assert.equal(main.useRoute().path, "/from-component");
assert.equal(imports.useRoute().path, "/from-component");
await main.navigateTo("/from-direct");
assert.equal(imports.useRoute().path, "/from-direct");

// Both Vite builds use the actual installed plugin, public entry and Nuxt aliases.
const entry = path.join(root, "entry.mjs");
fs.writeFileSync(entry, [
  'import { useRoute as directRoute, useState as directState, NuxtLink as directLink } from "@vizejs/musea-nuxt";',
  'import { useRoute, useState } from "#imports";',
  'import { NuxtLink } from "#components";',
  'export function verify() {',
  '  if (useRoute() !== directRoute() || useState("count") !== directState("count") || NuxtLink !== directLink) throw new Error("split packed mock context");',
  '  const render = NuxtLink.setup({ to: "/from-vite" }, { slots: {} });',
  '  render().props.onClick({ preventDefault() {} });',
  '  if (directRoute().path !== "/from-vite") throw new Error("split component navigation context");',
  '  return directRoute().path;',
  '}',
].join("\n"));
const base = { configFile: false, root, plugins: [plugin], logLevel: "silent" };
await build({ ...base, build: { outDir: "client", minify: false, lib: { entry, formats: ["es"], fileName: () => "entry.mjs" } } });
const client = await import(pathToFileURL(path.join(root, "client/entry.mjs")).href);
assert.equal(client.verify(), "/from-vite");
await build({ ...base, build: { ssr: entry, outDir: "server", minify: false, rollupOptions: { output: { entryFileNames: "entry.mjs" } } } });
const server = await import(pathToFileURL(path.join(root, "server/entry.mjs")).href);
assert.equal(server.verify(), "/from-vite");
// Explicit package policy must agree for the public entry and its runtime subpath.
for (const [directory, ssr] of [
  ["server-external", { external: ["@vizejs/musea-nuxt"] }],
  ["server-no-external", { noExternal: ["@vizejs/musea-nuxt"] }],
]) {
  await build({ ...base, ssr, build: { ssr: entry, outDir: directory, minify: false, rollupOptions: { output: { entryFileNames: "entry.mjs" } } } });
  const controlled = await import(pathToFileURL(path.join(root, directory, "entry.mjs")).href);
  assert.equal(controlled.verify(), "/from-vite");
}
for (const [filename, bytes] of before) assert.deepEqual(fs.readFileSync(filename), bytes, filename);
const providers = Object.fromEntries(["vite", "vue", "vue-router"].map((name) => [name, JSON.parse(fs.readFileSync(path.join(root, "node_modules", name, "package.json"), "utf8")).version]));
console.log(JSON.stringify({ direct: true, client: true, ssr: true, providers }));
`;
