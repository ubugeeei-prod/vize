import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { mkdtemp, readdir, rm, writeFile } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import test from "node:test";
import type { TestContext } from "node:test";
import { fileURLToPath } from "node:url";

const moduleEntry = new URL("../../dist/index.mjs", import.meta.url).href;
const hidePlugin = fileURLToPath(new URL("./fixtures/hide-oxlint-plugin.mjs", import.meta.url));

// Set the built module up in a child process where `oxlint-plugin-vize` cannot resolve.
const setupScript = `
const [entry, rootDir, rawOptions] = process.argv.slice(1);
let resolved = true;
try {
  import.meta.resolve("oxlint-plugin-vize");
} catch {
  resolved = false;
}
if (resolved) throw new Error("oxlint-plugin-vize must not resolve");
const { default: nuxtModule } = await import(entry);
await nuxtModule(JSON.parse(rawOptions), {
  _version: "3.19.3",
  options: { rootDir, buildDir: rootDir + "/.nuxt", srcDir: rootDir, dev: true, modules: [] },
  hook() {},
  callHook: async () => {},
});
`;

async function setupWithoutPlugin(t: TestContext, options: Record<string, unknown>) {
  const root = await mkdtemp(path.join(os.tmpdir(), "vize-nuxt-no-plugin-"));
  t.after(() => rm(root, { force: true, recursive: true }));
  await writeFile(path.join(root, "package.json"), '{ "name": "app" }\n');
  const child = spawnSync(
    process.execPath,
    [
      "--import",
      hidePlugin,
      "--input-type=module",
      "--eval",
      setupScript,
      moduleEntry,
      root,
      JSON.stringify({ compiler: false, musea: false, ...options }),
    ],
    { encoding: "utf8", timeout: 60_000 },
  );
  return { child, files: (await readdir(root)).sort() };
}

void test("the module sets up with default lint and no oxlint-plugin-vize, silently", async (t) => {
  const { child, files } = await setupWithoutPlugin(t, {});

  assert.equal(child.status, 0, child.stderr);
  assert.equal(child.stderr, "");
  assert.deepEqual(files, ["package.json"]);
});

void test("requested lint and checker skip with one install hint when the plugin is absent", async (t) => {
  const { child, files } = await setupWithoutPlugin(t, { lint: {}, checker: true });

  assert.equal(child.status, 0, child.stderr);
  assert.deepEqual(child.stderr.trim().split("\n"), [
    "@vizejs/nuxt: lint skipped; oxlint-plugin-vize is not installed. Run `vp install -D oxlint-plugin-vize` to enable it.",
  ]);
  assert.deepEqual(files, ["package.json"]);
});
