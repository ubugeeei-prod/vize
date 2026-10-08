import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import test from "node:test";

import {
  createScopedMirror,
  loadScopedConfig,
  readScopedConfig,
  validateNestedConfigs,
  validateScopedConfig,
} from "./scoped-config.ts";

void test("discovered JSON and MTS retain object inheritance and authored bytes", async (t) => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-discovered-config-"));
  t.after(() => fs.rmSync(root, { recursive: true, force: true }));
  const configFile = path.join(root, "oxlint.config.mts");
  const file = path.join(root, "src/Panel.vue");
  fs.mkdirSync(path.dirname(file), { recursive: true });
  fs.writeFileSync(file, "<template />\n");
  const bytes =
    'export default { extends: [{ rules: { "no-debugger": "warn" }, overrides: [{ files: ["src/*.vue"], rules: { "no-debugger": "off" } }] }], settings: { vize: { preset: "incremental" } }, overrides: [{ files: ["src/Panel.vue"], rules: { "vize/vue/no-v-html": "off" } }] };\n';
  fs.writeFileSync(configFile, bytes);
  const discovered = readScopedConfig(root, ["src"]);
  assert.ok(discovered);
  assert.equal(discovered.discovered, true);
  const config = await loadScopedConfig(discovered);
  validateScopedConfig(config, []);
  validateNestedConfigs(config, [file]);
  const mirror = createScopedMirror(root, config, [file]);
  try {
    const loaded = await loadScopedConfig({ ...config, file: mirror.sibling });
    const copy = mirror.originalsToCopies.get(file)!;
    const prefix = copy.slice(0, -"src/Panel.vue".length);
    assert.deepEqual(loaded.value, {
      extends: [
        {
          rules: { "no-debugger": "warn" },
          overrides: [
            { files: ["src/*.vue"], rules: { "no-debugger": "off" }, excludeFiles: [copy] },
            { files: [`${prefix}src/*.vue`], rules: { "no-debugger": "off" }, excludeFiles: [] },
          ],
        },
      ],
      settings: { vize: { preset: "incremental" } },
      overrides: [
        { files: ["src/Panel.vue"], rules: { "vize/vue/no-v-html": "off" }, excludeFiles: [copy] },
        {
          files: [`${prefix}src/Panel.vue`],
          rules: { "vize/vue/no-v-html": "off" },
          excludeFiles: [],
        },
      ],
    });
    assert.equal(fs.readFileSync(configFile, "utf8"), bytes);
    assert.equal(fs.readFileSync(file, "utf8"), "<template />\n");
  } finally {
    mirror.cleanup();
  }
  fs.mkdirSync(path.join(root, "src/nested"));
  fs.writeFileSync(path.join(root, "src/nested/.oxlintrc.json"), "{}");
  assert.throws(
    () => validateNestedConfigs(config, [path.join(root, "src/nested/Child.vue")]),
    /nested configuration/u,
  );
  // An explicit config disables nested lookup in the actual engine as well.
  validateNestedConfigs({ ...config, discovered: false }, [
    path.join(root, "src/nested/Child.vue"),
  ]);
  fs.unlinkSync(configFile);
  fs.writeFileSync(path.join(root, ".oxlintrc.json"), '{"ignorePatterns":["dist/**"]}\n');
  assert.equal(readScopedConfig(root, ["src"])?.discovered, true);
});

void test("inherited project resolution refuses before a temporary source is written", () => {
  for (const parent of [{ plugins: ["import"] }, { options: { typeAware: true } }])
    assert.throws(
      () =>
        validateScopedConfig(
          {
            file: "/project/oxlint.config.mts",
            bytes: "",
            module: true,
            value: { extends: [parent] },
          },
          [],
        ),
      /cannot preserve/u,
    );
});
