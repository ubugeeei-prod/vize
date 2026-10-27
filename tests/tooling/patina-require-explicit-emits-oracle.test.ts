import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";

import {
  baselineConfig,
  resolveBaselineRuntime,
} from "../../tools/support/compat/fixtures/lint-divergence-baseline.mjs";

const repoRoot = path.resolve(import.meta.dirname, "../..");
const fixture = path.join(
  repoRoot,
  "tests/_fixtures/differential/linter/require-explicit-emits-parity-10822",
);
const recorded = JSON.parse(fs.readFileSync(path.join(fixture, "eslint-plugin-vue.json"), "utf8"));

test("recorded require-explicit-emits findings are eslint-plugin-vue 10.9.2's", async () => {
  const runtime = resolveBaselineRuntime();
  assert.equal(runtime.version, "10.9.2");
  assert.equal(recorded.upstream.version, runtime.version);
  const eslint = new runtime.ESLint({
    cwd: repoRoot,
    overrideConfigFile: true,
    overrideConfig: baselineConfig(runtime, { [recorded.rule]: "warn" }),
  });
  const names = fs
    .readdirSync(fixture)
    .filter((name) => name.endsWith(".vue.txt"))
    .sort();
  assert.deepEqual(names, Object.keys(recorded.files).sort());
  for (const name of names) {
    const [result] = await eslint.lintText(fs.readFileSync(path.join(fixture, name), "utf8"), {
      filePath: path.join(repoRoot, name.replace(/\.txt$/, "")),
    });
    const findings = result.messages.map(
      ({ ruleId, line, column, endLine, endColumn, message }) => ({
        ruleId,
        line,
        column,
        endLine,
        endColumn,
        message,
      }),
    );
    assert.deepEqual(findings, recorded.files[name], name);
  }
});
