import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import { loadConfig } from "../../npm/vize/src/config.ts";

async function withTempConfig<T>(
  fileName: string,
  content: string,
  run: (dir: string) => Promise<T>,
): Promise<T> {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-config-alias-entries-"));
  try {
    fs.writeFileSync(path.join(dir, fileName), content);
    return await run(dir);
  } finally {
    fs.rmSync(dir, { force: true, recursive: true });
  }
}

test("lsp alias is rewritten to languageServer in resolved config", async () => {
  await withTempConfig(
    "vize.config.json",
    JSON.stringify({ lsp: { enabled: true, hover: false } }),
    async (dir) => {
      const config = await loadConfig(dir, { mode: "root" });
      assert.ok(config);
      assert.deepEqual(config.languageServer, { enabled: true, hover: false });
      assert.equal("lsp" in config, false);

      assert.ok(config.entries);
      assert.equal(config.entries.length, 1);
      assert.deepEqual(config.entries[0]?.languageServer, { enabled: true, hover: false });
      assert.equal("lsp" in (config.entries[0] ?? {}), false);
    },
  );
});

test("when both lsp and languageServer present, languageServer wins and lsp is dropped", async () => {
  await withTempConfig(
    "vize.config.json",
    JSON.stringify({ lsp: { enabled: false }, languageServer: { enabled: true } }),
    async (dir) => {
      const config = await loadConfig(dir, { mode: "root" });
      assert.ok(config);
      assert.deepEqual(config.languageServer, { enabled: true });
      assert.equal("lsp" in config, false);
    },
  );
});

test("array config preserves entry order and metadata (monorepo entries)", async () => {
  await withTempConfig(
    "vize.config.json",
    JSON.stringify([
      { name: "a", files: ["a/**"], formatter: { printWidth: 1 } },
      { name: "b", files: ["b/**"], formatter: { printWidth: 2 } },
    ]),
    async (dir) => {
      const config = await loadConfig(dir, { mode: "root" });
      assert.ok(config);
      assert.ok(config.entries);
      assert.equal(config.entries.length, 2);
      assert.equal(config.entries[0]?.name, "a");
      assert.equal(config.entries[1]?.name, "b");
      assert.equal(config.entries[0]?.formatter?.printWidth, 1);
      assert.equal(config.entries[1]?.formatter?.printWidth, 2);
      assert.deepEqual(config.entries[0]?.files, ["a/**"]);
    },
  );
});

test("array config: a global entry (no basePath/files/ignores) merges into resolved root", async () => {
  await withTempConfig(
    "vize.config.json",
    JSON.stringify([
      { formatter: { printWidth: 50 }, linter: { enabled: true } },
      { name: "scoped", files: ["src/**"], formatter: { printWidth: 80 } },
    ]),
    async (dir) => {
      const config = await loadConfig(dir, { mode: "root" });
      assert.ok(config);
      assert.equal(config.formatter?.printWidth, 50);
      assert.equal(config.linter?.enabled, true);
      assert.ok(config.entries);
      assert.equal(config.entries.length, 2);
      assert.equal(config.entries[1]?.name, "scoped");
    },
  );
});

test("object config with explicit entries: root becomes entries[0], explicit entries follow", async () => {
  await withTempConfig(
    "vize.config.json",
    JSON.stringify({
      formatter: { printWidth: 10 },
      entries: [{ name: "pkg-a", basePath: "packages/a", formatter: { printWidth: 20 } }],
    }),
    async (dir) => {
      const config = await loadConfig(dir, { mode: "root" });
      assert.ok(config);
      assert.equal(config.formatter?.printWidth, 10);
      assert.ok(config.entries);
      assert.equal(config.entries.length, 2);
      assert.deepEqual(config.entries[0], { formatter: { printWidth: 10 } });
      assert.equal(config.entries[1]?.name, "pkg-a");
      assert.equal("entries" in (config.entries[1] ?? {}), false);
    },
  );
});

test("function-style .ts config that returns an array produces monorepo entries", async () => {
  await withTempConfig(
    "vize.config.ts",
    [
      "export default () => ([",
      "  { name: 'x', files: ['x/**'], formatter: { printWidth: 1 } },",
      "  { name: 'y', files: ['y/**'], formatter: { printWidth: 2 } },",
      "]);",
      "",
    ].join("\n"),
    async (dir) => {
      const config = await loadConfig(dir, { mode: "root" });
      assert.ok(config);
      assert.ok(config.entries);
      assert.equal(config.entries.length, 2);
      assert.equal(config.entries[0]?.name, "x");
      assert.equal(config.entries[1]?.name, "y");
      assert.equal(config.entries[0]?.formatter?.printWidth, 1);
    },
  );
});
