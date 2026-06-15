import assert from "node:assert/strict";
import test from "node:test";

import { hasCiBlockingVrtResult } from "./commands.ts";
import type { VrtSummary } from "../vrt.ts";

function summary(overrides: Partial<VrtSummary>): VrtSummary {
  return {
    total: 1,
    passed: 1,
    failed: 0,
    new: 0,
    skipped: 0,
    duration: 25,
    ...overrides,
  };
}

void test("VRT CI blocks on visual diffs", () => {
  assert.equal(hasCiBlockingVrtResult(summary({ failed: 1, passed: 0 })), true);
});

void test("VRT CI blocks on capture errors", () => {
  assert.equal(hasCiBlockingVrtResult(summary({ skipped: 1, passed: 0 })), true);
});

void test("VRT CI allows clean and newly-created baselines", () => {
  assert.equal(hasCiBlockingVrtResult(summary({})), false);
  assert.equal(hasCiBlockingVrtResult(summary({ passed: 0, new: 1 })), false);
});
