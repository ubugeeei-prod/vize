import { readFile, writeFile } from "node:fs/promises";
import { requiredEnv } from "../../../../tools/ci/lib/scripts/files.ts";
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
declare const Bun: {
  Glob: new (pattern: string) => {
    scan(options: { cwd: string; onlyFiles: boolean }): AsyncIterable<string>;
  };
};
function filesMatching(pattern: string, cwd: string) {
  return new Bun.Glob(pattern).scan({ cwd, onlyFiles: true });
}

const project = requiredEnv("PROJECT");
const deploy = requiredEnv("DEPLOY");
const evidence = process.env.EVIDENCE;
const version = process.env.RELEASE_VERSION;
const packages: Record<string, string> = {};
for await (const path of filesMatching("node_modules/**/package.json", project)) {
  const manifest = JSON.parse(await readFile(`${project}/${path}`, "utf8")) as {
    name?: string;
    version: string;
  };
  if (manifest.name === "vize" || manifest.name?.startsWith("@vizejs/")) {
    assert.equal(manifest.version, version, `${manifest.name} version differs`);
    packages[manifest.name] = manifest.version;
  }
}
for (const name of ["@vizejs/native", "@vizejs/nuxt", "@vizejs/vite-plugin", "vize"])
  assert.equal(packages[name], version, `${name} is absent from the public graph`);
const lock = await readFile(`${project}/bun.lock`, "utf8");
assert.equal(lock.includes("workspace:"), false, "public lock contains a workspace edge");
assert.equal(lock.includes("file:"), false, "public lock contains a file edge");
const runtimeManifestText = await readFile(`${deploy}/server/package.json`, "utf8");
const runtimeManifest = JSON.parse(runtimeManifestText);
const runtimeDependencies = (runtimeManifest.dependencies ?? {}) as Record<string, string>;
assert.ok(Object.keys(runtimeDependencies).length > 0, "deployment graph is empty");
for (const [name, specifier] of Object.entries(runtimeDependencies)) {
  assert.match(specifier, /^\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?$/, `${name} is local`);
  const installed = JSON.parse(
    await readFile(`${deploy}/server/node_modules/${name}/package.json`, "utf8"),
  ) as { name?: string; version: string };
  assert.equal(installed.version, specifier, `${name} deployment version differs`);
}
const runtimePackages = new Map<string, Set<string>>();
for await (const path of filesMatching("server/node_modules/**/package.json", deploy)) {
  const manifest = JSON.parse(await readFile(`${deploy}/${path}`, "utf8")) as {
    name?: string;
    version: string;
  };
  if (typeof manifest.name !== "string" || typeof manifest.version !== "string") continue;
  const versions = runtimePackages.get(manifest.name) ?? new Set();
  versions.add(manifest.version);
  runtimePackages.set(manifest.name, versions);
}
const runtimeInventory = `${JSON.stringify(
  Object.fromEntries(
    [...runtimePackages]
      .sort(([a], [b]) => a.localeCompare(b))
      .map(([name, versions]) => [name, [...versions].sort()]),
  ),
  null,
  2,
)}\n`;
await writeFile(`${evidence}/server-packages.json`, runtimeInventory);
const response = await readFile(`${evidence}/response.html`, "utf8");
let assets = 0;
for await (const _ of filesMatching("**/*", `${project}/.output/public/_nuxt`)) assets++;
assert.ok(assets > 0, "Nuxt emitted no Vite client assets");
await writeFile(
  `${evidence}/receipt.json`,
  `${JSON.stringify(
    {
      schema: "vize.published-fresh-nuxt-bun.v1",
      version,
      bun: process.versions.bun,
      nuxt: "4.6.4",
      vue: "3.5.43",
      packages: Object.fromEntries(
        Object.entries(packages).sort(([left], [right]) =>
          left < right ? -1 : left > right ? 1 : 0,
        ),
      ),
      assets,
      installMs: Number(process.env.INSTALL_MS),
      buildMs: Number(process.env.BUILD_MS),
      runtimeInstallMs: Number(process.env.RUNTIME_INSTALL_MS),
      runtimeDependencies: Object.keys(runtimeDependencies).length,
      lockSha256: createHash("sha256").update(lock).digest("hex"),
      runtimeManifestSha256: createHash("sha256").update(runtimeManifestText).digest("hex"),
      runtimeInventorySha256: createHash("sha256").update(runtimeInventory).digest("hex"),
      responseSha256: createHash("sha256").update(response).digest("hex"),
    },
    null,
    2,
  )}\n`,
);
