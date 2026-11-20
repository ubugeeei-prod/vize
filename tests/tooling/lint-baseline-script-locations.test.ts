import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

import {
  baselineConfig,
  realignScriptLocations,
} from "../../tools/support/compat/fixtures/lint-divergence-baseline.mjs";
import { resolveScorecardRuntime } from "../../tools/support/compat/fixtures/lint-divergence-scorecard.ts";

const root = fileURLToPath(new URL("../../", import.meta.url));
// Reduced from scalar's CodeInput.vue: vue-eslint-parser 10.4 places these
// `<script setup>` nodes 29 lines below their ranges, past the end of file.
const twoScripts = fs.readFileSync(
  path.join(root, "tests/_fixtures/lint-baseline-script-locations/TwoScripts.vue.txt"),
  "utf8",
);

function lineOf(code: string, offset: number): number {
  return code.slice(0, offset).split(/\r\n|[\r\n\u2028\u2029]/u).length;
}

function misplaced(ast: unknown, code: string): string[] {
  const found: string[] = [];
  const seen = new Set<unknown>();
  const visit = (node: any): void => {
    if (node === null || typeof node !== "object" || seen.has(node)) return;
    seen.add(node);
    if (Array.isArray(node)) return node.forEach(visit);
    if (
      Array.isArray(node.range) &&
      node.loc &&
      node.loc.start.line !== lineOf(code, node.range[0])
    )
      found.push(node.type);
    for (const [key, value] of Object.entries(node))
      if (!["parent", "templateBody", "loc", "range"].includes(key)) visit(value);
  };
  visit(ast);
  return found;
}

test("the baseline parser reports script nodes at the lines their ranges name", () => {
  const runtime = resolveScorecardRuntime();
  const [config] = baselineConfig(runtime, {});
  const options = {
    ...config.languageOptions.parserOptions,
    filePath: "TwoScripts.vue",
  };
  const raw = runtime.vueParser.parseForESLint(twoScripts, options);
  const realigned = config.languageOptions.parser.parseForESLint(twoScripts, options);
  // The pinned parser misplaces these nodes; realignment must undo exactly that.
  assert.notDeepEqual(misplaced(raw.ast, twoScripts), []);
  assert.deepEqual(misplaced(realigned.ast, twoScripts), []);
  assert.deepEqual(misplaced(realigned.ast.tokens, twoScripts), []);
  assert.deepEqual(misplaced(realigned.ast.comments, twoScripts), []);
});

test("realignment counts every ESLint line break and leaves the template alone", () => {
  const code = "a\r\nb\u2028c\rd";
  const template = { type: "VElement", range: [0, 1], loc: { start: { line: 9, column: 9 } } };
  const ast = {
    type: "Program",
    range: [0, 8],
    loc: { start: { line: 5, column: 0 }, end: { line: 5, column: 0 } },
    templateBody: template,
    body: [{ type: "Identifier", range: [7, 8], loc: { start: {}, end: {} } }],
  };
  realignScriptLocations(ast, code);
  assert.deepEqual(ast.loc, { start: { line: 1, column: 0 }, end: { line: 4, column: 1 } });
  assert.deepEqual(ast.body[0].loc, { start: { line: 4, column: 0 }, end: { line: 4, column: 1 } });
  assert.deepEqual(template.loc, { start: { line: 9, column: 9 } });
});
