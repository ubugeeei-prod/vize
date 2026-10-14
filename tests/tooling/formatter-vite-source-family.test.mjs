import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { test } from "node:test";
import {
  originalViteInputs,
  originalViteSources,
  viteConfigurationPlans,
} from "../differential/formatter-vite-source.mjs";
import {
  originalViteArchives,
  originalViteSourceFamily,
} from "../differential/formatter-vite-source-family.mjs";

const root = fileURLToPath(new URL("../../", import.meta.url));
const archives = Object.values(originalViteArchives);

function withArchives(run) {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-vite-family-source-"));
  try {
    for (const file of archives) {
      const target = path.join(directory, file);
      fs.mkdirSync(path.dirname(target), { recursive: true });
      fs.copyFileSync(path.join(root, file), target);
    }
    return run(directory);
  } finally {
    fs.rmSync(directory, { recursive: true, force: true });
  }
}

void test("the archived family feeds the five original controls and all nine plans", () => {
  const family = originalViteSourceFamily(root);
  assert.deepEqual(Object.keys(family), Object.keys(originalViteSources));
  const input = originalViteInputs(root);
  const plans = viteConfigurationPlans(root);
  assert.equal(input.controls.length, 5);
  assert.deepEqual(
    plans.map(({ id }) => id),
    [
      "original-script-0",
      "original-script-1",
      "original-script-2",
      "original-script-3",
      "original-script-4",
      "public-shared",
      "public-false",
      "public-object",
      "public-inheritance",
    ],
  );
  // Only the archives are read: a tree without the live Vite tests or glyph
  // fixture produces the same inputs and plans.
  withArchives((directory) => {
    assert.deepEqual(originalViteSourceFamily(directory), family);
    assert.deepEqual(originalViteInputs(directory), input);
    assert.deepEqual(viteConfigurationPlans(directory), plans);
  });
});

void test("a changed, truncated, missing or non-file archive is refused", () => {
  withArchives((directory) => {
    for (const file of archives) {
      const target = path.join(directory, file);
      const original = fs.readFileSync(target);
      const changed = Buffer.from(original);
      changed[0] ^= 1;
      fs.writeFileSync(target, changed);
      assert.throws(() => originalViteSourceFamily(directory), /historical Vite vector changed/);
      fs.writeFileSync(target, original.subarray(0, -1));
      assert.throws(() => originalViteSourceFamily(directory), /historical Vite vector changed/);
      fs.unlinkSync(target);
      assert.throws(() => originalViteSourceFamily(directory), /ENOENT/);
      fs.mkdirSync(target);
      assert.throws(() => originalViteSourceFamily(directory), /historical Vite vector changed/);
      fs.rmdirSync(target);
      fs.writeFileSync(target, original);
    }
    assert.equal(viteConfigurationPlans(directory).length, 9);
  });
});
