import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import { defineConfig, loadConfig, resolveConfigExport } from "../../npm/cli/src/config.ts";

function withTempDir<T>(prefix: string, run: (dir: string) => Promise<T>): Promise<T> {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), prefix));
  return (async () => {
    try {
      return await run(dir);
    } finally {
      fs.rmSync(dir, { force: true, recursive: true });
    }
  })();
}

test("loadConfig with mode 'none' ignores any present config", async () => {
  await withTempDir("vize-config-none-", async (dir) => {
    fs.writeFileSync(path.join(dir, "vize.config.json"), '{ "formatter": { "lineWidth": 1 } }');
    const config = await loadConfig(dir, { mode: "none" });
    assert.equal(config, null);
  });
});

test("loadConfig with mode 'auto' resolves a config from an ancestor directory", async () => {
  await withTempDir("vize-config-auto-", async (dir) => {
    const nested = path.join(dir, "packages", "app", "src");
    fs.mkdirSync(nested, { recursive: true });
    fs.writeFileSync(path.join(dir, "vize.config.json"), '{ "formatter": { "lineWidth": 7 } }');

    const auto = await loadConfig(nested, { mode: "auto" });
    assert.equal(auto?.formatter?.lineWidth, 7);

    // 'root' must only look in the starting directory, so the ancestor config
    // is invisible from the nested directory.
    const root = await loadConfig(nested, { mode: "root" });
    assert.equal(root, null);
  });
});

test("loadConfig resolves an explicit configFile by absolute and relative path", async () => {
  await withTempDir("vize-config-file-", async (dir) => {
    fs.writeFileSync(path.join(dir, "custom.json"), '{ "formatter": { "lineWidth": 71 } }');

    const absolute = await loadConfig("/nonexistent-root", {
      configFile: path.join(dir, "custom.json"),
    });
    assert.equal(absolute?.formatter?.lineWidth, 71);

    const relative = await loadConfig(dir, { configFile: "custom.json" });
    assert.equal(relative?.formatter?.lineWidth, 71);
  });
});

test("loadConfig returns null for a missing explicit configFile", async () => {
  await withTempDir("vize-config-missing-", async (dir) => {
    const config = await loadConfig(dir, { configFile: "does-not-exist.ts" });
    assert.equal(config, null);
  });
});

test("loadConfig prefers higher-priority config formats when several coexist", async () => {
  await withTempDir("vize-config-precedence-", async (dir) => {
    // .ts wins over .mjs wins over .json (matching CONFIG_FILE_NAMES order,
    // excluding .pkl which needs an external runtime).
    fs.writeFileSync(path.join(dir, "vize.config.json"), '{ "formatter": { "lineWidth": 80 } }');
    fs.writeFileSync(
      path.join(dir, "vize.config.mjs"),
      "export default { formatter: { lineWidth: 99 } }\n",
    );
    const mjsWins = await loadConfig(dir, { mode: "root" });
    assert.equal(mjsWins?.formatter?.lineWidth, 99, ".mjs should win over .json");

    fs.writeFileSync(
      path.join(dir, "vize.config.ts"),
      "export default { formatter: { lineWidth: 55 } }\n",
    );
    const tsWins = await loadConfig(dir, { mode: "root" });
    assert.equal(tsWins?.formatter?.lineWidth, 55, ".ts should win over .mjs");
  });
});

test("loadConfig loads ESM .mjs and CommonJS .js config files", async () => {
  await withTempDir("vize-config-mjs-", async (dir) => {
    fs.writeFileSync(
      path.join(dir, "vize.config.mjs"),
      "export default { formatter: { lineWidth: 91 } }\n",
    );
    const esm = await loadConfig(dir, { mode: "root" });
    assert.equal(esm?.formatter?.lineWidth, 91);
  });

  await withTempDir("vize-config-cjs-", async (dir) => {
    fs.writeFileSync(
      path.join(dir, "vize.config.js"),
      "module.exports = { formatter: { lineWidth: 33 } }\n",
    );
    const cjs = await loadConfig(dir, { mode: "root" });
    assert.equal(cjs?.formatter?.lineWidth, 33);
  });
});

test("resolveConfigExport passes the provided env to a function config", async () => {
  const resolved = await resolveConfigExport(
    (env) => ({ formatter: { lineWidth: env.command === "build" ? 100 : 80 } }),
    { command: "build", mode: "production" },
  );
  assert.equal(resolved.formatter?.lineWidth, 100);
});

test("resolveConfigExport falls back to the default env when none is provided", async () => {
  // The default env is { mode: "development", command: "serve" }.
  const resolved = await resolveConfigExport((env) => ({
    formatter: { lineWidth: env.mode === "development" && env.command === "serve" ? 11 : 22 },
  }));
  assert.equal(resolved.formatter?.lineWidth, 11);
});

test("resolveConfigExport returns object configs unchanged", async () => {
  const resolved = await resolveConfigExport({ formatter: { lineWidth: 5 } });
  assert.equal(resolved.formatter?.lineWidth, 5);
});

test("defineConfig returns its argument unchanged", () => {
  const config = { formatter: { lineWidth: 64 } };
  assert.equal(defineConfig(config), config);
});
