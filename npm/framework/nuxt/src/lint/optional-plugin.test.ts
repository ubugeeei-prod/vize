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

async function setupWithoutPlugin(t: TestContext, options: Record<string, unknown>) {
  const root = await mkdtemp(path.join(os.tmpdir(), "vize-nuxt-no-plugin-"));
  t.after(() => rm(root, { force: true, recursive: true }));
  await writeFile(path.join(root, "package.json"), '{ "name": "app" }\n');
  const child = spawnSync(
    process.execPath,
    [
      "--import",
      hidePlugin,
      fileURLToPath(new URL("../../test/setup-without-plugin.ts", import.meta.url)),
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
