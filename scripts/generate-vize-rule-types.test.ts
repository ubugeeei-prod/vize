import assert from "node:assert/strict";
import fs from "node:fs";
import { test } from "node:test";

import {
  generateLintRuleTypesFromSnapshot,
  lintRuleTypesPath,
} from "./generate-vize-rule-types.ts";

void test("generated vize lint rule types are up to date", () => {
  const actual = fs.readFileSync(lintRuleTypesPath, "utf-8");
  const expected = generateLintRuleTypesFromSnapshot();

  assert.equal(actual, expected);
});
