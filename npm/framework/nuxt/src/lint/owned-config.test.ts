import assert from "node:assert/strict";
import { mkdtemp, readFile, rm, stat, symlink, utimes, writeFile } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import test from "node:test";

import { writeFileIfChanged } from "./generation.ts";
import { writeOwnedNuxtConfig } from "./owned-config.ts";

const initial =
  '{"settings":{"vize":{"generatedBy":"@vizejs/nuxt"}},"rules":{"no-console":"warn"}}\n';
const changed =
  '{"settings":{"vize":{"generatedBy":"@vizejs/nuxt"}},"rules":{"no-console":"error"}}\n';

void test("the reserved config preserves authored, malformed and symlink collisions", async (t) => {
  const root = await mkdtemp(path.join(os.tmpdir(), "vize-nuxt-owned-"));
  t.after(() => rm(root, { recursive: true, force: true }));
  const file = path.join(root, ".oxlint.vize.json");
  for (const authored of ['{"rules":{"no-console":"off"}}\n', "unfinished {\n", "null\n"]) {
    await writeFile(file, authored);
    await assert.rejects(
      writeOwnedNuxtConfig(file, initial, writeFileIfChanged),
      /Refusing to replace authored/,
    );
    assert.equal(await readFile(file, "utf8"), authored);
  }
  await rm(file);
  const target = path.join(root, "authored.json");
  await writeFile(target, initial);
  await symlink(target, file);
  await assert.rejects(
    writeOwnedNuxtConfig(file, changed, writeFileIfChanged),
    /must be a regular file/,
  );
  assert.equal(await readFile(target, "utf8"), initial);
});

void test("owned generation creates once, skips identical bytes and atomically updates", async (t) => {
  const root = await mkdtemp(path.join(os.tmpdir(), "vize-nuxt-owned-"));
  t.after(() => rm(root, { recursive: true, force: true }));
  const file = path.join(root, ".oxlint.vize.json");
  assert.equal(await writeOwnedNuxtConfig(file, initial, writeFileIfChanged), true);
  assert.equal(await readFile(file, "utf8"), initial);
  const pinned = new Date("2001-02-03T04:05:06.000Z");
  await utimes(file, pinned, pinned);
  assert.equal(await writeOwnedNuxtConfig(file, initial, writeFileIfChanged), false);
  assert.equal((await stat(file)).mtimeMs, pinned.getTime());
  assert.equal(await writeOwnedNuxtConfig(file, changed, writeFileIfChanged), true);
  assert.equal(await readFile(file, "utf8"), changed);
});
