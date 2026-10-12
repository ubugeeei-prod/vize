import assert from "node:assert/strict";
import { createHash } from "node:crypto";
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
import { originalViteSourceFamily } from "../differential/formatter-vite-source-family.mjs";

const root = fileURLToPath(new URL("../../", import.meta.url));
const custody = "tests/_fixtures/differential/config/vue-namespace/source-custody/";
const archivePath = custody + "original-vite-plus-config.test.ts.fixture";
const configPath = "npm/builder/vite/src/vite-plus/config.test.ts";
const read = (name) => fs.readFileSync(path.join(root, name));
const hash = (raw) => createHash("sha256").update(raw).digest("hex");
const sources = Object.keys(originalViteSources);

function withFamily(current, run) {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-vite-family-source-"));
  try {
    for (const file of [...sources, archivePath]) {
      const target = path.join(directory, file);
      fs.mkdirSync(path.dirname(target), { recursive: true });
      fs.writeFileSync(target, read(file === configPath && !current ? archivePath : file));
      fs.chmodSync(target, 0o644);
    }
    return run(directory);
  } finally {
    fs.rmSync(directory, { recursive: true, force: true });
  }
}

const originalRead =
  "  const files = Object.fromEntries(\n" +
  "    Object.entries(originalViteSources).map(([file, expected]) => {\n" +
  "      const raw = fs.readFileSync(path.join(root, file));\n" +
  "      assert.equal(hash(raw), expected, `original Vite vector changed: ${file}`);\n" +
  "      return [file, raw.toString()];\n" +
  "    }),\n" +
  "  );\n";
const successorRead = "  const files = originalViteSourceFamily(root);\n";
const successorImport =
  'import { originalViteSourceFamily } from "./formatter-vite-source-family.mjs";\n';

function helperInverse(current) {
  assert.equal(current.split(successorImport).length, 2);
  assert.equal(current.split(successorRead).length, 2);
  return current
    .replace(
      successorImport,
      'import { hash } from "../../npm/native/scripts/formatter-history-build.mjs";\n',
    )
    .replace(
      'import assert from "node:assert/strict";\n',
      'import assert from "node:assert/strict";\n' +
        'import fs from "node:fs";\n' +
        'import path from "node:path";\n',
    )
    .replace(successorRead, originalRead);
}

void test("whole original helper, caller, three digests and every VM and plan byte survive the finite reader succession", () => {
  const original = read(custody + "original-formatter-vite-source.mjs.fixture");
  assert.equal(hash(original), "9d1bd08a963a54e0b89c323b05007acf1bf96a6052507ea71c207fc403599f39");
  const current = read("tests/differential/formatter-vite-source.mjs").toString();
  assert.equal(helperInverse(current), original.toString());
  assert.notEqual(helperInverse(current + "\n"), original.toString());
  assert.notEqual(
    helperInverse(current.replace("plugin: false", "plugin: true")),
    original.toString(),
  );
  const caller = read(custody + "original-public-config-cli-history.test.ts.fixture");
  assert.equal(hash(caller), "6f4fb44b40e21a20f7f682eebb66dc78f037b0418cb6b48f41cbb03ca36466a0");
  assert.deepEqual(
    read("npm/builder/vite/src/vite-plus/public-config-cli-history.test.ts"),
    caller,
  );
  assert.equal(
    hash(read("tests/tooling/differential-formatter-vite-contract.test.mjs")),
    "d9c84807e4978fd09ac437baf48be67a77c733813e1724d6b58fbf9933083cc9",
  );
  assert.deepEqual(originalViteSources, {
    "npm/builder/vite/src/vite-plus/import-sorting.test.ts":
      "02b2910051683dca5108a47a622db63dc086f5a0a9b599b20ab33e043473832c",
    [configPath]: "82570d354aa7cfb9300bca05be49e7bd451694beea019b635cac1ccb7c8056a4",
    "crates/vize_glyph/tests/fixtures/sort-imports/UserCard.vue":
      "ae86278aa0a0b247d9e027b703699f02cf370ea37da9dd96368adf3ff334b106",
  });
});

void test("complete actual and original families feed identical whole historical inputs and all nine plans", () => {
  const current = originalViteSourceFamily(root);
  assert.deepEqual(Object.keys(current), sources);
  for (const file of sources) {
    assert.equal(hash(Buffer.from(current[file])), originalViteSources[file]);
    assert.equal(current[file], read(file === configPath ? archivePath : file).toString());
  }
  assert.equal(
    hash(read(configPath)),
    "0e6efef9ed98c4dfa593e692aadef9e3620c1f924d2c630deda6bfb8c070ddfd",
  );
  const input = originalViteInputs(root);
  const plans = viteConfigurationPlans(root);
  assert.equal(input.controls.length, 5);
  assert.equal(plans.length, 9);
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
  for (const family of [false, true]) {
    withFamily(family, (directory) => {
      assert.deepEqual(originalViteSourceFamily(directory), current);
      assert.deepEqual(originalViteInputs(directory), input);
      assert.deepEqual(viteConfigurationPlans(directory), plans);
    });
  }
});

void test("every complete family input and archive refuses missing, partial, changed or nonregular physical source", () => {
  for (const current of [false, true]) {
    withFamily(current, (directory) => {
      for (const file of [...sources, archivePath]) {
        const target = path.join(directory, file);
        const original = fs.readFileSync(target);
        const refuse = () => {
          assert.throws(() => originalViteSourceFamily(directory));
          assert.throws(() => originalViteInputs(directory));
          assert.throws(() => viteConfigurationPlans(directory));
        };
        fs.unlinkSync(target);
        refuse();
        fs.writeFileSync(target, original.subarray(0, original.length - 1));
        refuse();
        fs.writeFileSync(target, Buffer.concat([original, Buffer.from("\n")]));
        refuse();
        const changed = Buffer.from(original);
        changed[0] ^= 1;
        fs.writeFileSync(target, changed);
        refuse();
        fs.writeFileSync(target, original);
        fs.chmodSync(target, 0o755);
        refuse();
        fs.chmodSync(target, 0o644);
        fs.unlinkSync(target);
        fs.mkdirSync(target);
        refuse();
        fs.rmdirSync(target);
        const external = path.join(directory, "same-source-external.fixture");
        fs.writeFileSync(external, original);
        fs.symlinkSync(external, target);
        refuse();
        fs.unlinkSync(target);
        fs.unlinkSync(external);
        fs.writeFileSync(target, original);
        fs.chmodSync(target, 0o644);
        assert.equal(viteConfigurationPlans(directory).length, 9);
      }
    });
  }
});

void test("current and historical bodies cannot form an undeclared or hybrid source family", () => {
  for (const current of [false, true]) {
    withFamily(current, (directory) => {
      const config = path.join(directory, configPath);
      const archived = path.join(directory, archivePath);
      fs.writeFileSync(archived, read(configPath));
      assert.throws(() => originalViteSourceFamily(directory), /historical Vite vector changed/);
      fs.writeFileSync(archived, read(archivePath));
      fs.writeFileSync(
        config,
        read(archivePath)
          .toString()
          .replace("assert.equal(withVue, defineConfig);", "assert.ok(withVue !== defineConfig);"),
      );
      assert.throws(() => originalViteSourceFamily(directory), /original Vite vector changed/);
      fs.writeFileSync(config, read(configPath).toString() + read(archivePath).toString());
      assert.throws(() => originalViteSourceFamily(directory), /original Vite vector changed/);
      fs.writeFileSync(
        config,
        read(configPath)
          .toString()
          .replace(
            "assert.notEqual(withVue, defineConfig);",
            "assert.notEqual(withVize, defineConfig);",
          ),
      );
      assert.throws(() => originalViteSourceFamily(directory), /original Vite vector changed/);
    });
  }
});
