import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { relocateTaskConfig } from "./config-paths.ts";
import { runNative } from "./runner.ts";

test("task configs retain original scopes outside the project directory", async () => {
  const root = process.cwd();
  const config = {
    ignores: ["gen/**"],
    entries: [
      { ignores: ["gen/**"] },
      { files: ["src/legacy/**/*.vue"], linter: { rules: { "a11y/alt-text": "off" as const } } },
      { basePath: "packages/ui", files: ["**/*.vue", "!generated/**"], ignores: ["fixtures/**"] },
      { basePath: path.resolve(root, "packages/absolute"), files: ["**/*.vue"] },
    ],
  };
  const original = structuredClone(config);
  let serialized = "";
  await runNative("lint", ["src"], { config, options: {} }, "native", async (_, args) => {
    serialized = args[3];
    assert.notEqual(path.dirname(serialized), root);
    const loaded = JSON.parse(readFileSync(serialized, "utf8"));
    assert.equal(loaded.ignores[0], `${root}/gen/**`);
    assert.ok(
      loaded.entries.every((entry: { basePath: string }) => path.isAbsolute(entry.basePath)),
    );
    const legacy = loaded.entries.find((entry: { files?: string[] }) =>
      entry.files?.includes("src/legacy/**/*.vue"),
    );
    assert.equal(legacy.basePath, root);
    assert.equal(legacy.linter.rules["a11y/alt-text"], "off");
    const ui = loaded.entries.find(
      (entry: { basePath: string }) => entry.basePath === path.join(root, "packages/ui"),
    );
    assert.deepEqual(ui.files, ["**/*.vue", "!generated/**"]);
    assert.deepEqual(ui.ignores, ["fixtures/**"]);
    return 0;
  });
  assert.deepEqual(config, original);
  assert.throws(() => readFileSync(serialized));
});

test("absolute and negated patterns survive relocation and empty configs work", () => {
  const root = process.cwd();
  const absolute = path.resolve(root, "src/**").replaceAll("\\", "/");
  assert.deepEqual(relocateTaskConfig({}, root).entries, []);
  const config = relocateTaskConfig({ ignores: [absolute, "!gen/keep.vue"], entries: [] }, root);
  assert.deepEqual(config.ignores, [absolute, `!${root.replaceAll("\\", "/")}/gen/keep.vue`]);
});
