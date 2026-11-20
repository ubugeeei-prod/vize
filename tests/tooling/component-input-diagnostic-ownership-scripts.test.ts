import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const workflowPath = path.join(root, ".github/workflows/component-input-diagnostic-ownership.yml");
const workflowScriptRoot = path.join(root, ".github/scripts/component-input-diagnostic-ownership");
const historicalBefore = "78719553cf550bcffbf1d0fb891fda642f42de3a";

function readWorkflowScript(name: string): string {
  return fs.readFileSync(path.join(workflowScriptRoot, name), "utf8");
}

function assertAuthenticatedBaseComparison(source: string): void {
  assert.match(source, /BASE_SHA: \$\{\{ github\.event\.pull_request\.base\.sha \}\}/u);
  assert.match(source, /SOURCE_SHA: \$\{\{ github\.event\.pull_request\.head\.sha \}\}/u);
  assert.match(source, new RegExp(`HISTORICAL_CALLBACK_BEFORE: ${historicalBefore}`, "u"));
  assert.match(source, /\[\[ "\$BASE_SHA" =~ \^\[0-9a-f\]\{40\}\$ \]\]/u);
  assert.match(source, /\[\[ "\$\(git -C "\$baseline" rev-parse HEAD\)" == "\$BASE_SHA" \]\]/u);
  assert.match(source, /--comparison-source-sha "\$BASE_SHA"/u);
  assert.match(source, /--historical-comparison-source-sha "\$HISTORICAL_CALLBACK_BEFORE"/u);
  assert.match(source, /--comparison-role pull-request-base/u);
  assert.doesNotMatch(source, /--comparison-source-sha 78719553cf550bcffbf1d0fb891fda642f42de3a/u);
}

test("workflow logic stays in checked-in scripts instead of multiline YAML blocks", () => {
  const source = fs.readFileSync(workflowPath, "utf8");
  assert.doesNotMatch(source, /^\s+run:\s*\|/mu);
  const scripts = [...source.matchAll(/^\s+run: source (\.github\/scripts\/[^\s]+\.sh)$/gmu)].map(
    (match) => match[1],
  );
  assert.equal(scripts.length, 12);
  assert.equal(new Set(scripts).size, scripts.length);
  for (const script of scripts) {
    assert.ok(fs.statSync(path.join(root, script)).isFile(), script);
  }
});

test("callback packet custody binds the comparison CLI to the exact PR base", () => {
  const workflow = fs.readFileSync(workflowPath, "utf8");
  const start = workflow.indexOf("  published-sdk-packets:");
  const end = workflow.indexOf("  unchanged-projection-packets:");
  assert.ok(start >= 0 && end > start);
  const source = [
    workflow.slice(start, end),
    readWorkflowScript("build-authenticated-base.sh"),
    readWorkflowScript("retain-callback-packets.sh"),
  ].join("\n");
  assertAuthenticatedBaseComparison(source);

  for (const tampered of [
    source.replace(
      "BASE_SHA: ${{ github.event.pull_request.base.sha }}",
      `BASE_SHA: ${"0".repeat(40)}`,
    ),
    source.replace('[[ "$(git -C "$baseline" rev-parse HEAD)" == "$BASE_SHA" ]]', "true"),
    source.replace(
      '--comparison-source-sha "$BASE_SHA"',
      `--comparison-source-sha ${historicalBefore}`,
    ),
  ]) {
    assert.throws(() => assertAuthenticatedBaseComparison(tampered), assert.AssertionError);
  }
});
