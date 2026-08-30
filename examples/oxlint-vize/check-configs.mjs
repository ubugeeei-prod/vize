/**
 * Package-local gate for this example's Oxlint configurations.
 *
 * The example's `lint` script intentionally exits non-zero, because it lints a
 * fixture that contains real violations. That means the exit code of `lint`
 * cannot tell CI whether the example still works, which is why the package used
 * to be exempted from the aggregate CI check entirely. This script covers the
 * part that can be verified without building anything: every `jsPlugins` entry
 * must resolve to the ESM entry that the Vize Oxlint plugin package actually
 * declares.
 *
 * That is the exact breakage this check exists for: a package-directory rename
 * once landed without updating these configs, so every plugin-loading script in
 * the example failed with "Cannot find module" and nothing in CI noticed.
 */
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const PLUGIN_PACKAGE_NAME = "oxlint-plugin-vize";

const exampleDir = path.dirname(fileURLToPath(import.meta.url));
const npmDir = path.resolve(exampleDir, "../../npm");

const readJson = (file) => JSON.parse(fs.readFileSync(file, "utf-8"));

/**
 * Locate the plugin package by manifest name rather than by hardcoded path, so
 * a future directory rename fails this check with a useful message instead of
 * silently leaving the configs pointing at a directory that no longer exists.
 */
const findPluginPackageDir = () => {
  const queue = [npmDir];
  const found = [];
  while (queue.length > 0) {
    const dir = queue.shift();
    for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
      if (!entry.isDirectory() || entry.name === "node_modules" || entry.name === "dist") continue;
      const child = path.join(dir, entry.name);
      const manifest = path.join(child, "package.json");
      if (fs.existsSync(manifest)) {
        if (readJson(manifest).name === PLUGIN_PACKAGE_NAME) found.push(child);
        continue;
      }
      queue.push(child);
    }
  }
  assert.equal(found.length, 1, `expected exactly one ${PLUGIN_PACKAGE_NAME} package under npm/`);
  return found[0];
};

const pluginPackageDir = findPluginPackageDir();
const pluginManifest = readJson(path.join(pluginPackageDir, "package.json"));
const declaredEntry = pluginManifest.exports?.["."]?.import;
assert.ok(declaredEntry, `${PLUGIN_PACKAGE_NAME} must declare exports["."].import`);
const expectedEntry = path.resolve(pluginPackageDir, declaredEntry);

const configFiles = fs
  .readdirSync(exampleDir)
  .filter((name) => name.startsWith(".oxlintrc") && name.endsWith(".json"))
  .sort();
assert.ok(configFiles.length > 0, "expected at least one .oxlintrc*.json in the example");

const report = [];
for (const name of configFiles) {
  const jsPlugins = readJson(path.join(exampleDir, name)).jsPlugins ?? [];
  for (const entry of jsPlugins) {
    const resolved = path.resolve(exampleDir, entry);
    assert.equal(
      resolved,
      expectedEntry,
      `${name}: jsPlugins entry ${JSON.stringify(entry)} resolves to ${resolved}, but ${PLUGIN_PACKAGE_NAME} declares ${expectedEntry}`,
    );
  }
  report.push(`${name}: ${jsPlugins.length === 0 ? "(no jsPlugins)" : jsPlugins.join(", ")}`);
}

console.log(report.join("\n"));
