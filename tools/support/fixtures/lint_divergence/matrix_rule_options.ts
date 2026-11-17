// Patina rule options the lint divergence matrix uses in place of Patina's
// own defaults.
//
// A configurable rule whose documented Patina default differs from
// eslint-plugin-vue's runs in the matrix with eslint-plugin-vue's default
// options, so both providers apply one policy. Each override is listed in the
// rule map under `matrixRuleOptions` with its reason, the source line that
// documents the default difference, and the test that keeps Patina's own
// default reporting the extra shapes. An override without that documentation
// is refused, so it cannot hide a real divergence.

import fs from "node:fs";
import os from "node:os";
import path from "node:path";

import { get, type Json } from "./value.ts";

export interface MatrixRuleOption {
  readonly upstreamRule: string;
  readonly patinaRule: string;
  readonly options: Json;
}

/** Every override in the rule map; throws on one that is not documented. */
export function matrixRuleOptions(root: string, ruleMap: Json): MatrixRuleOption[] {
  const entries = get(ruleMap, "entries");
  if (entries === null || typeof entries !== "object" || Array.isArray(entries))
    throw new Error("the rule map must carry entries");
  const overrides: MatrixRuleOption[] = [];
  for (const [upstreamRule, entry] of Object.entries(entries)) {
    const override = get(entry, "matrixRuleOptions");
    if (override === null) continue;
    const refuse = (detail: string) => new Error(`${upstreamRule} matrixRuleOptions ${detail}`);
    const patinaRule = get(entry, "patinaRule");
    if (get(entry, "status") !== "mapped" || typeof patinaRule !== "string")
      throw refuse("belongs to a mapped rule");
    const reason = get(override, "reason");
    if (typeof reason !== "string" || reason.trim() === "") throw refuse("needs a reason");
    const options = get(override, "options");
    if (options === null) throw refuse("needs eslint-plugin-vue's default options");
    const source = documentedSource(root, get(override, "documentation"), refuse);
    const guard = get(override, "defaultGuard");
    if (typeof guard !== "string" || !new RegExp(`\\bfn ${guard}\\s*\\(`, "u").test(source))
      throw refuse("needs a test in the documenting source that keeps Patina's default reporting");
    overrides.push({ upstreamRule, patinaRule, options });
  }
  return overrides;
}

/**
 * The documenting source. Its `line` must contain `text`, `text` must name
 * the default, and the module doc header must spell the upstream options.
 */
function documentedSource(
  root: string,
  documentation: Json,
  refuse: (detail: string) => Error,
): string {
  const file = get(documentation, "path"),
    line = get(documentation, "line"),
    text = get(documentation, "text"),
    upstream = get(documentation, "upstreamOptions");
  if (typeof file !== "string" || typeof line !== "number" || typeof text !== "string")
    throw refuse("documentation needs path, line and text");
  const source = fs.readFileSync(path.join(root, file), "utf8");
  const lines = source.split("\n");
  if (!lines[line - 1]?.includes(text))
    throw refuse(
      `documentation line ${String(line)} of ${file} does not contain the documented text`,
    );
  if (!/\bdefault\b/iu.test(text)) throw refuse("documentation text must document the default");
  const headerEnd = lines.findIndex((entry) => !entry.startsWith("//!"));
  const header = lines.slice(0, headerEnd < 0 ? lines.length : headerEnd).join("\n");
  if (typeof upstream !== "string" || !header.includes(upstream))
    throw refuse("documentation module doc header must spell eslint-plugin-vue's options");
  return source;
}

/** A config file that applies the overrides, or `null` when there are none. */
export function writeMatrixRuleOptionsConfig(
  overrides: readonly MatrixRuleOption[],
): string | null {
  if (overrides.length === 0) return null;
  const ruleOptions = Object.fromEntries(
    overrides.map(({ patinaRule, options }) => [patinaRule, options]),
  );
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-lint-matrix-options-"));
  const file = path.join(directory, "vize.lint-matrix.config.mjs");
  fs.writeFileSync(
    file,
    `export default ${JSON.stringify({ vue: { linter: { ruleOptions } } })};\n`,
  );
  return file;
}
