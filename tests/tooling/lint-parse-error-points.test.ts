import assert from "node:assert/strict";
import { test } from "node:test";

import { pairRuleLocationDivergences } from "../../tools/support/fixtures/lint_divergence/records.ts";

function record(
  column: number,
  endColumn: number,
  message: string,
  severity = "error",
): Parameters<typeof pairRuleLocationDivergences>[0][number] {
  return {
    file: "src/Number.vue",
    ruleId: "parser/template",
    severity,
    line: 4,
    column,
    endLine: 4,
    endColumn,
    message,
  };
}

test("a zero-width parse error point inside Patina's token range pairs", () => {
  const pairs = pairRuleLocationDivergences(
    [
      record(
        74,
        75,
        "[vize:parser/template] Missing whitespace between attributes; inferred a new attribute boundary.",
      ),
      record(39, 46, "[vize:parser/template] Missing semicolon after character reference."),
      record(13, 18, "[vize:parser/template] Error parsing JavaScript expression."),
      record(52, 58, "[vize:parser/template] Invalid end tag."),
    ],
    [
      record(74, 74, "Parsing error: missing-whitespace-between-attributes."),
      record(46, 46, "Parsing error: missing-semicolon-after-character-reference."),
      record(18, 18, "Parsing error: Unexpected end of expression."),
      record(52, 52, "Parsing error: x-invalid-end-tag."),
    ],
  );
  assert.equal(pairs.length, 4);
});

test("a control-character point immediately before Patina's byte range pairs", () => {
  const pairs = pairRuleLocationDivergences(
    [record(39, 40, "[vize:parser/template] Control character in input stream.")],
    [record(38, 38, "Parsing error: control-character-in-input-stream.")],
  );
  assert.equal(pairs.length, 1);
});

test("parse errors with another code, position or severity stay apart", () => {
  const positives = () => [
    record(
      60,
      65,
      "[vize:parser/template] Duplicate attribute `class`. Keeping the repeated attribute so parsing can continue.",
      "warning",
    ),
    record(
      74,
      75,
      "[vize:parser/template] Missing whitespace between attributes; inferred a new attribute boundary.",
    ),
  ];
  assert.equal(
    pairRuleLocationDivergences(positives(), [
      record(60, 60, "Parsing error: duplicate-attribute."),
    ]).length,
    0,
  );
  assert.equal(
    pairRuleLocationDivergences(positives(), [
      record(80, 80, "Parsing error: missing-whitespace-between-attributes."),
    ]).length,
    0,
  );
  assert.equal(
    pairRuleLocationDivergences(positives(), [
      record(74, 74, "Parsing error: duplicate-attribute."),
    ]).length,
    0,
  );
});
