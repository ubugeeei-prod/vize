import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const historicalBefore = "78719553cf550bcffbf1d0fb891fda642f42de3a";

function assertAuthenticatedBaseComparison(source: string): void {
  const start = source.indexOf("  published-sdk-packets:");
  const end = source.indexOf("  unchanged-projection-packets:");
  assert.ok(start >= 0 && end > start);
  const job = source.slice(start, end);
  assert.match(job, /BASE_SHA: \$\{\{ github\.event\.pull_request\.base\.sha \}\}/u);
  assert.match(job, /SOURCE_SHA: \$\{\{ github\.event\.pull_request\.head\.sha \}\}/u);
  assert.match(job, new RegExp(`HISTORICAL_CALLBACK_BEFORE: ${historicalBefore}`, "u"));
  assert.match(job, /\[\[ "\$BASE_SHA" =~ \^\[0-9a-f\]\{40\}\$ \]\]/u);
  assert.match(
    job,
    /\[\[ "\$\(git -C "\$RUNNER_TEMP\/callback-cfa-base" rev-parse HEAD\)" == "\$BASE_SHA" \]\]/u,
  );
  assert.match(job, /--comparison-source-sha "\$BASE_SHA"/u);
  assert.match(job, /--historical-comparison-source-sha "\$HISTORICAL_CALLBACK_BEFORE"/u);
  assert.match(job, /--comparison-role pull-request-base/u);
  assert.doesNotMatch(job, /--comparison-source-sha 78719553cf550bcffbf1d0fb891fda642f42de3a/u);
}

function assertConstructorPacketCustody(source: string): void {
  const start = source.indexOf("  unchanged-projection-packets:");
  assert.ok(start >= 0);
  const job = source.slice(start).replace(/\\\r?\n\s*/gu, " ");
  assert.doesNotMatch(job, /\bcmp\b[^\n]*legacy/u);
  assert.match(job, /constructor-projection-parity\.ts/u);
  assert.match(job, /base-legacy\.json/u);
  assert.match(job, /head-legacy\.json/u);
}

test("callback packet custody binds the comparison CLI to the exact PR base", () => {
  const source = fs.readFileSync(
    path.join(root, ".github/workflows/component-input-diagnostic-ownership.yml"),
    "utf8",
  );
  assertAuthenticatedBaseComparison(source);

  for (const tampered of [
    source.replace(
      "BASE_SHA: ${{ github.event.pull_request.base.sha }}",
      `BASE_SHA: ${"0".repeat(40)}`,
    ),
    source.replace(
      '[[ "$(git -C "$RUNNER_TEMP/callback-cfa-base" rev-parse HEAD)" == "$BASE_SHA" ]]',
      "true",
    ),
    source.replace(
      '--comparison-source-sha "$BASE_SHA"',
      `--comparison-source-sha ${historicalBefore}`,
    ),
  ]) {
    assert.throws(() => assertAuthenticatedBaseComparison(tampered), assert.AssertionError);
  }
});

test("constructor packet custody delegates certified changes to complete parity", () => {
  const source = fs.readFileSync(
    path.join(root, ".github/workflows/component-input-diagnostic-ownership.yml"),
    "utf8",
  );
  assertConstructorPacketCustody(source);
  assert.throws(
    () =>
      assertConstructorPacketCustody(
        `${source}\n  cmp "$RUNNER_TEMP/base-legacy.json" \\\n+    "$RUNNER_TEMP/head-legacy.json"\n`,
      ),
    assert.AssertionError,
  );
});
