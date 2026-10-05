import assert from "node:assert/strict";
import {
  access,
  mkdir,
  mkdtemp,
  readFile,
  rm,
  stat,
  symlink,
  utimes,
  writeFile,
} from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import test from "node:test";
import type { TestContext } from "node:test";

import { setupNuxtLintConfigGeneration } from "./generation.ts";

const issueSource = new URL("./fixtures/auto-init-vite-lint/vite.config.ts", import.meta.url);
const names = [
  "vite.config.js",
  "vite.config.mjs",
  "vite.config.ts",
  "vite.config.cjs",
  "vite.config.mts",
  "vite.config.cts",
];

async function rootFor(t: TestContext): Promise<string> {
  const root = await mkdtemp(path.join(os.tmpdir(), "vize-nuxt-auto-init-"));
  t.after(() => rm(root, { recursive: true, force: true }));
  return root;
}

async function absent(file: string): Promise<boolean> {
  try {
    await access(file);
    return false;
  } catch (error) {
    if ((error as NodeJS.ErrnoException).code === "ENOENT") return true;
    throw error;
  }
}

async function generate(rootDir: string, autoInit = true): Promise<string> {
  const nuxt = {
    _version: "3.19.3",
    options: { rootDir, buildDir: path.join(rootDir, ".nuxt") },
    hook() {},
  };
  const result = await setupNuxtLintConfigGeneration({ autoInit }, nuxt, {
    resolvePluginSpecifier: () => "./plugin.mjs",
    resolveProjectLintRules: () => undefined,
  });
  assert.ok(result);
  assert.equal(await absent(result.configFile), false);
  return result.configFile;
}

for (const name of names) {
  for (const ancestor of [false, true]) {
    void test(`auto-init preserves ${ancestor ? "ancestor" : "root"} ${name} ownership`, async (t) => {
      const parent = await rootFor(t);
      const root = ancestor ? path.join(parent, "project") : parent;
      await mkdir(root, { recursive: true });
      const file = path.join(parent, name);
      const source = await readFile(issueSource);
      await writeFile(file, source);
      const pinned = new Date("2001-02-03T04:05:06.000Z");
      await utimes(file, pinned, pinned);
      const before = await stat(file);

      await generate(root);

      assert.equal(await absent(path.join(root, "oxlint.config.mts")), true);
      assert.deepEqual(await readFile(file), source);
      const after = await stat(file);
      assert.equal(after.mtimeMs, before.mtimeMs);
      assert.equal(after.ino, before.ino);
    });
  }
}

void test("auto-init preserves Vite config ownership without evaluating dynamic exports", async (t) => {
  const root = await rootFor(t);
  const file = path.join(root, "vite.config.ts");
  const source =
    'throw new Error("must not evaluate");\nexport default async () => ({ lint: {} });\n';
  await writeFile(file, source);
  await generate(root);
  assert.equal(await readFile(file, "utf8"), source);
  assert.equal(await absent(path.join(root, "oxlint.config.mts")), true);
});

void test("auto-init conservatively preserves a Vite config without a literal lint block", async (t) => {
  const root = await rootFor(t);
  const file = path.join(root, "vite.config.js");
  const source = "export default { plugins: [] };\n";
  await writeFile(file, source);
  await generate(root);
  assert.equal(await readFile(file, "utf8"), source);
  assert.equal(await absent(path.join(root, "oxlint.config.mts")), true);
});

void test("auto-init preserves a symbolic Vite configuration without following its target", async (t) => {
  const root = await rootFor(t);
  const file = path.join(root, "vite.config.mts");
  await symlink(path.join(root, "missing-config.mts"), file);
  await generate(root);
  assert.equal(await absent(path.join(root, "oxlint.config.mts")), true);
});

void test("auto-init retains its default wrapper when there is no existing configuration", async (t) => {
  const root = await rootFor(t);
  await generate(root);
  assert.equal(await absent(path.join(root, "oxlint.config.mts")), false);
});

void test("explicit autoInit false still generates the artifact without a root wrapper", async (t) => {
  const root = await rootFor(t);
  await generate(root, false);
  assert.equal(await absent(path.join(root, "oxlint.config.mts")), true);
});
