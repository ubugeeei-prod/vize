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
  "tests/_fixtures/differential/linter/v-on-hyphenation-suggestion-10822",
);
const recorded = JSON.parse(fs.readFileSync(path.join(fixture, "eslint-plugin-vue.json"), "utf8"));

test("recorded v-on-event-hyphenation findings and fixes are eslint-plugin-vue 10.9.2's", async () => {
  const runtime = resolveBaselineRuntime();
  assert.equal(runtime.version, "10.9.2");
  assert.equal(recorded.upstream.version, runtime.version);
  const eslint = new runtime.ESLint({
    cwd: repoRoot,
    overrideConfigFile: true,
    overrideConfig: baselineConfig(runtime, { [recorded.rule]: ["warn", ...recorded.options] }),
  });
  for (const name of Object.keys(recorded.files)) {
    const [result] = await eslint.lintText(fs.readFileSync(path.join(fixture, name), "utf8"), {
      filePath: path.join(repoRoot, name.replace(/\.txt$/, "")),
    });
    const findings = result.messages.map(
      ({ ruleId, line, column, endLine, endColumn, message, fix }) => ({
        ruleId,
        line,
        column,
        endLine,
        endColumn,
        message,
        fix: fix?.text ?? null,
      }),
    );
    assert.deepEqual(findings, recorded.files[name], name);
  }
});
