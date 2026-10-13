import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { createHash } from "node:crypto";
import { createRequire } from "node:module";
import { fileURLToPath, pathToFileURL } from "node:url";

const oracle = fileURLToPath(new URL("./fixtures/app-component-order/", import.meta.url));
const fromOracle = createRequire(path.join(oracle, "package.json"));
const source = fs.readFileSync(path.join(oracle, "nuxt-transform.ts.txt"));
assert.equal(source.length, 4846);
assert.equal(
  createHash("sha256").update(source).digest("hex"),
  "229b33a703af61a63604e1623a207b3a802728a754b368dac8e20dfb983087b6",
);
const materialized = path.join(oracle, ".generated/transform.ts");
fs.mkdirSync(path.dirname(materialized), { recursive: true });
fs.writeFileSync(materialized, source);
assert(fs.readFileSync(materialized).equals(source));
const { TransformPlugin } = await import(pathToFileURL(materialized).href);
for (const [name, version] of [
  ["nuxt", "4.0.3"],
  ["@nuxt/kit", "4.0.3"],
  ["unimport", "5.2.0"],
])
  assert.equal(
    JSON.parse(fs.readFileSync(fromOracle.resolve(`${name}/package.json`))).version,
    version,
  );

export function createNuxtImportOracle(components) {
  return TransformPlugin(
    {
      options: {
        buildDir: path.join(oracle, ".nuxt"),
        sourcemap: { client: false, server: false },
      },
    },
    {
      mode: "all",
      getComponents: () => components,
      serverComponentRuntime: "#app/components/server-component",
    },
  ).raw({});
}

export function originalNuxtComponents() {
  const packageDir = path.dirname(fromOracle.resolve("nuxt/package.json"));
  return [
    ["NuxtPage", "dist/pages/runtime/page.js"],
    ["NuxtRouteAnnouncer", "dist/app/components/nuxt-route-announcer.js"],
    ["NuxtLayout", "dist/app/components/nuxt-layout.js"],
  ].map(([pascalName, relative]) => {
    const filePath = path.join(packageDir, relative);
    assert(fs.existsSync(filePath), "actual pinned Nuxt runtime entry must exist");
    return { pascalName, filePath };
  });
}
