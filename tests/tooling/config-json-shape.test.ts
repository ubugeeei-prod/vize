import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import { loadConfig } from "../../npm/cli/src/config.ts";

type JsonConfigResult = Awaited<ReturnType<typeof loadConfig>>;

/**
 * Write `content` as `vize.config.json` inside a fresh temp dir, load it through
 * the public config API in `root` mode, then clean up. The loader reads the file
 * and routes JSON through parseJsonConfig -> normalizeLoadedConfig, so this
 * exercises the real JSON-shape normalization path.
 */
async function loadJsonConfig(content: string): Promise<JsonConfigResult> {
  const tempDir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-config-json-shape-"));
  try {
    fs.writeFileSync(path.join(tempDir, "vize.config.json"), content);
    return await loadConfig(tempDir, { mode: "root" });
  } finally {
    fs.rmSync(tempDir, { force: true, recursive: true });
  }
}

test("JSON config with unknown keys is preserved (not stripped/validated)", async () => {
  const result = await loadJsonConfig(
    JSON.stringify({ unknownKey: 123, formatter: { printWidth: 9 } }),
  );

  // normalizeLoadedConfig does not validate against any schema: unknown
  // top-level keys survive onto both the resolved root and the root entry.
  const resolved = result as Record<string, unknown> | null;
  assert.equal(resolved?.unknownKey, 123);
  assert.deepEqual(resolved?.formatter, { printWidth: 9 });

  const entries = resolved?.entries as Array<Record<string, unknown>>;
  assert.equal(entries.length, 1);
  assert.equal(entries[0].unknownKey, 123);
  assert.deepEqual(entries[0].formatter, { printWidth: 9 });
});

test("empty JSON object config yields {entries: []}", async () => {
  const result = await loadJsonConfig("{}");

  // An empty {} produces an empty root entry that isEmptyConfigEntry filters out.
  assert.deepEqual(result, { entries: [] });
});

test("top-level JSON null normalizes to {entries: []} (stripNullish)", async () => {
  const result = await loadJsonConfig("null");

  // stripNullish maps top-level null to undefined; normalizeConfigObject({})
  // then yields the empty-entries shape.
  assert.deepEqual(result, { entries: [] });
});

test("empty JSON array config yields {entries: []}", async () => {
  const result = await loadJsonConfig("[]");

  // Top-level [] goes through normalizeConfigEntries producing entries: [].
  assert.deepEqual(result, { entries: [] });
});

test("nested null values are stripped from object config", async () => {
  const result = await loadJsonConfig(
    JSON.stringify({ formatter: { printWidth: 5, useTabs: null }, linter: null }),
  );

  // stripNullish recursively removes null-valued keys (nested useTabs) and
  // whole null sections (linter) before normalization.
  const resolved = result as Record<string, unknown> | null;
  const formatter = resolved?.formatter as Record<string, unknown>;
  assert.equal(formatter.printWidth, 5);
  assert.equal("useTabs" in formatter, false);
  assert.equal("linter" in (resolved ?? {}), false);

  // The full resolved shape: stripped root mirrored into a single entry.
  assert.deepEqual(result, {
    formatter: { printWidth: 5 },
    entries: [{ formatter: { printWidth: 5 } }],
  });
});

test("type-aware lint opt-in is exposed across config artifacts", () => {
  const schema = JSON.parse(
    fs.readFileSync(path.join("npm", "cli", "schemas", "vize.config.schema.json"), "utf8"),
  ) as {
    definitions: {
      LinterConfig: {
        properties: Record<string, { type?: string; description?: string }>;
      };
    };
  };
  const generatedTypes = fs.readFileSync(
    path.join("npm", "cli", "src", "types", "generated.ts"),
    "utf8",
  );
  const pklLinterConfig = fs.readFileSync(
    path.join("npm", "cli", "pkl", "LinterConfig.pkl"),
    "utf8",
  );
  const pklCompatConfig = fs.readFileSync(path.join("npm", "cli", "pkl", "vize.pkl"), "utf8");
  const pklSchemaGenerator = fs.readFileSync(
    path.join("npm", "cli", "pkl", "jsonschema", "generate.pkl"),
    "utf8",
  );

  assert.equal(schema.definitions.LinterConfig.properties.typeAware.type, "boolean");
  assert.match(generatedTypes, /typeAware\?: boolean;/);
  assert.match(pklLinterConfig, /typeAware: Boolean = false/);
  assert.match(pklCompatConfig, /typeAware: Boolean\? = null/);
  assert.match(pklSchemaGenerator, /\["typeAware"\] = new JsonSchema/);
});

test("JSX type-check opt-in is exposed across config artifacts", () => {
  const schema = JSON.parse(
    fs.readFileSync(path.join("npm", "cli", "schemas", "vize.config.schema.json"), "utf8"),
  ) as {
    definitions: {
      TypeCheckerConfig: {
        properties: Record<string, { type?: string; description?: string }>;
      };
    };
  };
  const generatedTypes = fs.readFileSync(
    path.join("npm", "cli", "src", "types", "generated.ts"),
    "utf8",
  );
  const pklTypeCheckerConfig = fs.readFileSync(
    path.join("npm", "cli", "pkl", "TypeCheckerConfig.pkl"),
    "utf8",
  );
  const pklCompatConfig = fs.readFileSync(path.join("npm", "cli", "pkl", "vize.pkl"), "utf8");
  const pklSchemaGenerator = fs.readFileSync(
    path.join("npm", "cli", "pkl", "jsonschema", "generate.pkl"),
    "utf8",
  );

  assert.equal(schema.definitions.TypeCheckerConfig.properties.jsxTypecheck.type, "boolean");
  assert.match(generatedTypes, /jsxTypecheck\?: boolean;/);
  assert.match(pklTypeCheckerConfig, /jsxTypecheck: Boolean = false/);
  assert.match(pklCompatConfig, /jsxTypecheck: Boolean\? = null/);
  assert.match(pklSchemaGenerator, /\["jsxTypecheck"\] = new JsonSchema/);
});
