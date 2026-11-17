// The config the lint divergence matrix passes to Patina: rule options used
// in place of Patina's own defaults, and the Vue dialect a user's config
// would select for the project.
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

/**
 * The Vue 2 or 2.7 dialect of a project whose compatibility-ledger evidence
 * (pinned packages, test oracles, registry notes) proves only that major, as
 * `vue.version` in its config would select it. A Vue 3 project, a project
 * that mixes majors, or one without evidence keeps Patina's default.
 */
export function matrixVueVersion(ledger: Json, fixturePath: string): "2" | "2.7" | null {
  const fixtures = get(ledger, "fixtures");
  if (!Array.isArray(fixtures)) return null;
  const fixture = fixtures.find((entry) => get(entry, "fixturePath") === fixturePath);
  const evidence = get(fixture ?? null, "dialectCoverage", "evidence");
  const majors = new Set<string>();
  for (const claim of Array.isArray(evidence) ? evidence : []) {
    const dialects = get(claim, "dialects");
    for (const dialect of Array.isArray(dialects) ? dialects : [])
      if (typeof dialect === "string" && /^vue[23](?:\.7)?-sfc$/u.test(dialect))
        majors.add(dialect);
  }
  if (majors.size !== 1) return null;
  return majors.has("vue2-sfc") ? "2" : majors.has("vue2.7-sfc") ? "2.7" : null;
}

/** A config file for one project, or `null` when it would be empty. */
export function writeMatrixConfig(
  overrides: readonly MatrixRuleOption[],
  vueVersion: string | null,
): string | null {
  if (overrides.length === 0 && vueVersion === null) return null;
  const ruleOptions = Object.fromEntries(
    overrides.map(({ patinaRule, options }) => [patinaRule, options]),
  );
  const vue = {
    ...(vueVersion === null ? {} : { version: vueVersion }),
    ...(overrides.length === 0 ? {} : { linter: { ruleOptions } }),
  };
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-lint-matrix-options-"));
  const file = path.join(directory, "vize.lint-matrix.config.mjs");
  fs.writeFileSync(file, `export default ${JSON.stringify({ vue })};\n`);
  return file;
}
