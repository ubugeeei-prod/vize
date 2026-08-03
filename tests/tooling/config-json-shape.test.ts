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

function bracedBody(source: string, declaration: RegExp): string {
  const match = declaration.exec(source);
  assert.ok(match, `missing declaration ${declaration}`);
  const open = source.indexOf("{", match.index);
  let depth = 0;
  for (let index = open; index < source.length; index += 1) {
    if (source[index] === "{") depth += 1;
    if (source[index] === "}" && --depth === 0) return source.slice(open + 1, index);
  }
  throw new Error(`unterminated declaration ${declaration}`);
}

function pklClassKeys(file: string, className: string): string[] {
  const body = bracedBody(fs.readFileSync(file, "utf8"), new RegExp(`class ${className}\\b`));
  return [...body.matchAll(/^\s{2}([\w$]+|`[^`]+`)\s*:/gm)]
    .map((match) => match[1].replaceAll("`", ""))
    .sort();
}

function typescriptInterfaceKeys(source: string, interfaceName: string): string[] {
  const body = bracedBody(source, new RegExp(`export interface ${interfaceName}\\b`));
  return [...body.matchAll(/^\s{2}([\w$]+)\??:/gm)].map((match) => match[1]).sort();
}

const configSections = ["CompilerConfig", "LinterConfig", "TypeCheckerConfig"] as const;

test("public config keys stay identical across generated artifacts", () => {
  const schema = JSON.parse(
    fs.readFileSync(path.join("npm", "cli", "schemas", "vize.config.schema.json"), "utf8"),
  ) as {
    definitions: Record<string, { properties: Record<string, unknown> }>;
  };
  const generatedTypes = fs.readFileSync(
    path.join("npm", "cli", "src", "types", "generated.ts"),
    "utf8",
  );
  const compatPkl = path.join("npm", "cli", "pkl", "vize.pkl");

  for (const section of configSections) {
    const primaryKeys = pklClassKeys(path.join("npm", "cli", "pkl", `${section}.pkl`), section);
    const artifactKeys = {
      primaryPkl: primaryKeys,
      compatPkl: pklClassKeys(compatPkl, section),
      jsonSchema: Object.keys(schema.definitions[section].properties).sort(),
      generatedTypes: typescriptInterfaceKeys(generatedTypes, section),
    };
    assert.deepEqual(
      artifactKeys,
      Object.fromEntries(Object.keys(artifactKeys).map((name) => [name, primaryKeys])),
      `${section} public key sets must be identical`,
    );
  }
});
