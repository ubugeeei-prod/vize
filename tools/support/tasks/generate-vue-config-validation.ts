/** Regenerate the dependency-free canonical Vue validation graph from the public schema. */
import assert from "node:assert/strict";
import fs from "node:fs";

const root = new URL("../../../", import.meta.url);
const schema = JSON.parse(
  fs.readFileSync(new URL("npm/cli/schemas/vize.config.schema.json", root), "utf8"),
) as { properties: Record<string, unknown>; definitions: Record<string, unknown> };
const declarations = fs.readFileSync(new URL("npm/cli/src/types/runtime.ts", root), "utf8");
const optionKeys = /type VueOptionKeys =([\s\S]*?);/u.exec(declarations);
const entryType = /export type VueOptionsEntry = Pick<\s*VizeConfigEntry,\s*([^>]+)>/u.exec(
  declarations,
);
assert.ok(optionKeys, "VueOptionKeys declaration is missing");
assert.ok(entryType, "VueOptionsEntry declaration is missing");
const keys = (source: string): string[] =>
  [...source.matchAll(/"([^"]+)"/gu)].map((match) => match[1]!);
const properties: Record<string, unknown> = Object.fromEntries(
  keys(optionKeys[1]!).map((key) => {
    assert.ok(Object.hasOwn(schema.properties, key), `Public schema is missing Vue option ${key}`);
    return [key, schema.properties[key]];
  }),
);
const actionAliases = {
  compile: "compiler",
  typeCheck: "typeChecker",
  fmt: "formatter",
} as const;
for (const [action, section] of Object.entries(actionAliases)) {
  properties[action] = schema.properties[section];
}
properties.lint = {
  type: "object",
  required: ["jsPlugins", "plugins", "rules", "settings"],
  properties: {
    jsPlugins: { type: "array", items: { type: "string" } },
    plugins: { type: "array", items: { type: "string" } },
    rules: { type: "object", additionalProperties: true },
    settings: { type: "object", additionalProperties: true },
    ignorePatterns: { type: "array", items: { type: "string" } },
    overrides: { type: "array" },
  },
  additionalProperties: true,
};
properties.version = { $ref: "#/definitions/VueVersion" };
const entries = schema.definitions.VizeConfigEntry as { properties: Record<string, unknown> };
const entryProperties = Object.fromEntries(
  keys(entryType[1]!).map((key) => [key, entries.properties[key]]),
);
properties.entries = {
  type: "array",
  items: {
    type: "object",
    properties: entryProperties,
    additionalProperties: false,
  },
};
const definitions: Record<string, unknown> = {};
function collectDefinitions(value: unknown): void {
  if (value === null || typeof value !== "object") return;
  if ("$ref" in value && typeof value.$ref === "string") {
    const name = value.$ref.split("/").at(-1)!;
    if (!Object.hasOwn(definitions, name)) {
      assert.ok(Object.hasOwn(schema.definitions, name), `Public schema is missing ${name}`);
      definitions[name] = schema.definitions[name];
      collectDefinitions(definitions[name]);
    }
  }
  Object.values(value).forEach(collectDefinitions);
}
collectDefinitions(properties);
function withoutAnnotations(value: unknown, parentKey?: string): unknown {
  if (Array.isArray(value)) return value.map((item) => withoutAnnotations(item, parentKey));
  if (value === null || typeof value !== "object") return value;
  return Object.fromEntries(
    Object.entries(value)
      .filter(
        ([key]) =>
          parentKey === "properties" ||
          !["description", "default", "title", "$schema"].includes(key),
      )
      .map(([key, item]) => [key, withoutAnnotations(item, key)]),
  );
}
const graph = JSON.stringify(
  withoutAnnotations({ type: "object", properties, additionalProperties: false, definitions }),
);
const runtimePath = new URL("crates/vize_carton/src/config/loader/vite-runtime.mjs", root);
const source = fs.readFileSync(runtimePath, "utf8");
const pattern =
  /const vueOptionSchema = JSON.parse\(\s*(?:"(?:[^"\\]|\\.)*"|'(?:[^'\\]|\\.)*')\s*,?\s*\);/u;
assert.match(source, pattern, "Embedded Vue schema declaration is missing");
fs.writeFileSync(
  runtimePath,
  source.replace(pattern, () => `const vueOptionSchema = JSON.parse(${JSON.stringify(graph)});`),
);
