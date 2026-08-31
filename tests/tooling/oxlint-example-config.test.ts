import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

import { checkedPackagesViaVpRun } from "../../tools/config/vite-plus/task-inputs.ts";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const exampleDir = path.join(root, "examples/oxlint-vize");
const pluginDir = path.join(root, "npm/oxlint");

interface Manifest {
  exports?: Record<string, { import?: string } | string>;
  name?: string;
  scripts?: Record<string, string>;
}

interface OxlintConfig {
  jsPlugins?: string[];
}

const readJson = <T>(file: string): T => JSON.parse(fs.readFileSync(file, "utf-8")) as T;

const exampleConfigNames = (): string[] =>
  fs
    .readdirSync(exampleDir)
    .filter((name) => name.startsWith(".oxlintrc") && name.endsWith(".json"))
    .sort();

test("oxlint example configs load the plugin package's declared ESM entry", () => {
  const manifest = readJson<Manifest>(path.join(pluginDir, "package.json"));
  assert.equal(manifest.name, "oxlint-plugin-vize");

  const declared = manifest.exports?.["."];
  assert.ok(declared != null && typeof declared === "object", "expected an exports['.'] object");
  const declaredEntry = declared.import;
  assert.ok(declaredEntry, "expected exports['.'].import");
  const expectedEntry = path.resolve(pluginDir, declaredEntry);

  // Resolve each config's jsPlugins entries the way Oxlint does: relative to the
  // directory holding the config file. A directory rename in npm/ must show up
  // here as a mismatch instead of as a runtime "Cannot find module".
  const resolved: Record<string, string[]> = {};
  for (const name of exampleConfigNames()) {
    const config = readJson<OxlintConfig>(path.join(exampleDir, name));
    resolved[name] = (config.jsPlugins ?? []).map((entry) => path.resolve(exampleDir, entry));
  }

  assert.deepEqual(resolved, {
    ".oxlintrc.help.json": [expectedEntry],
    ".oxlintrc.json": [expectedEntry],
    ".oxlintrc.short-help.json": [expectedEntry],
    ".oxlintrc.unused-vars.json": [],
  });
});

test("oxlint example README documents the plugin path its configs actually use", () => {
  const readme = fs.readFileSync(path.join(exampleDir, "README.md"), "utf-8");
  const documented = [...readme.matchAll(/`(\.\.\/\.\.\/npm\/[^`]*\/dist\/index\.mjs)`/g)].map(
    (match) => match[1],
  );

  const used = new Set<string>();
  for (const name of exampleConfigNames()) {
    for (const entry of readJson<OxlintConfig>(path.join(exampleDir, name)).jsPlugins ?? []) {
      used.add(entry);
    }
  }

  assert.deepEqual(documented, ["../../npm/oxlint/dist/index.mjs"]);
  assert.deepEqual([...used].sort(), ["../../npm/oxlint/dist/index.mjs"]);
});

test("oxlint example ships a package check that does not depend on lint's exit code", () => {
  const manifest = readJson<Manifest>(path.join(exampleDir, "package.json"));
  assert.equal(manifest.scripts?.check, "vp exec node ./check-configs.mjs");

  // The aggregate CI package check used to skip this example on the grounds that
  // its `lint` script intentionally exits non-zero. The aggregate runs `check`,
  // never `lint`, so the exemption only hid real breakage.
  assert.equal(checkedPackagesViaVpRun.includes("./examples/oxlint-vize"), true);

  const result = spawnSync(process.execPath, ["./check-configs.mjs"], {
    cwd: exampleDir,
    encoding: "utf-8",
  });

  assert.equal(result.status, 0, result.stderr);

  // Assert the observable behavior: the check succeeds and reports every config
  // it discovered. The exact line formatting is incidental, so leave it alone.
  const reported = result.stdout
    .split("\n")
    .map((line) => line.trim())
    .filter((line) => line !== "");
  const names = exampleConfigNames();
  for (const name of names) {
    assert.equal(
      reported.some((line) => line.startsWith(`${name}:`)),
      true,
      `expected ${name} to be reported, got:\n${result.stdout}`,
    );
  }
  assert.equal(reported.length, names.length, result.stdout);
});
