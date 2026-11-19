import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

import {
  matrixRuleOptions,
  matrixVueVersion,
  writeMatrixConfig,
} from "../../tools/support/fixtures/lint_divergence/matrix_rule_options.ts";
import { toolArgs } from "../../tools/support/compat/fixtures/tool-matrix-command.ts";

const root = fileURLToPath(new URL("../../", import.meta.url));
const ruleMap = JSON.parse(
  fs.readFileSync(path.join(root, "tests/_fixtures/patina-eslint-vue-rule-map.json"), "utf8"),
);

test("every matrix override is documented and keeps Patina's default tested", () => {
  const overrides = matrixRuleOptions(root, ruleMap);
  assert.deepEqual(
    overrides.map(({ upstreamRule }) => upstreamRule),
    ["vue/v-on-handler-style"],
  );
  const config = writeMatrixConfig(overrides, null);
  assert.ok(config);
  assert.equal(
    fs.readFileSync(config, "utf8"),
    `export default {"vue":{"linter":{"ruleOptions":{"vue/v-on-handler-style":["method","inline-function"]}}}};\n`,
  );
  fs.rmSync(path.dirname(config), { recursive: true, force: true });
  assert.equal(writeMatrixConfig([], null), null);
});

test("the core-tool linter accepts the same generated matrix config", () => {
  assert.deepEqual(
    toolArgs({ vueGlobs: ["src/**/*.vue"] }, "linter", "unused", {
      linterConfigPath: "/tmp/vize-lint-matrix.config.mjs",
    }),
    [
      "lint",
      "src/**/*.vue",
      "--format",
      "json",
      "--preset",
      "all",
      "--config",
      "/tmp/vize-lint-matrix.config.mjs",
    ],
  );
});

test("a Vue 2 or 2.7 project is linted with the dialect its packages pin", () => {
  const ledger = JSON.parse(
    fs.readFileSync(path.join(root, "tests/_fixtures/fixture-compatibility-ledger.json"), "utf8"),
  );
  assert.equal(matrixVueVersion(ledger, "tests/_fixtures/_git/vue2-elm"), "2");
  assert.equal(matrixVueVersion(ledger, "tests/_fixtures/_git/habitica"), "2.7");
  assert.equal(matrixVueVersion(ledger, "tests/_fixtures/_git/airi"), null);
  // Mixed majors keep Patina's default.
  assert.equal(matrixVueVersion(ledger, "tests/_fixtures/_git/gogocode"), null);
  assert.equal(matrixVueVersion(ledger, "tests/_fixtures/_git/missing"), null);
  const config = writeMatrixConfig([], "2.7");
  assert.ok(config);
  assert.equal(fs.readFileSync(config, "utf8"), `export default {"vue":{"version":"2.7"}};\n`);
  fs.rmSync(path.dirname(config), { recursive: true, force: true });
});

function project(header: string, guard = "fn keeps_default() {}") {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-matrix-options-"));
  fs.writeFileSync(path.join(directory, "rule.rs"), `${header}\n\nuse x;\n${guard}\n`);
  return directory;
}

function map(documentation: Record<string, unknown>, extra: Record<string, unknown> = {}) {
  return {
    entries: {
      "vue/example": {
        status: "mapped",
        patinaRule: "vue/example",
        matrixRuleOptions: {
          options: ["a"],
          reason: "Patina's default differs.",
          documentation: { path: "rule.rs", line: 2, upstreamOptions: '["a"]', ...documentation },
          defaultGuard: "keeps_default",
          ...extra,
        },
      },
    },
  };
}

const header =
  '//! vue/example\n//! Everything is reported by the default policy;\n//! `["a"]` selects upstream.';

test("an override whose doc header documents the default difference is accepted", () => {
  const directory = project(header);
  assert.equal(
    matrixRuleOptions(directory, map({ text: "reported by the default policy" })).length,
    1,
  );
});

test("an override without a documented default difference is refused", () => {
  const directory = project(header);
  assert.throws(
    () => matrixRuleOptions(directory, map({ text: "vue/example", line: 1 })),
    /must document the default/u,
  );
  assert.throws(
    () => matrixRuleOptions(directory, map({ text: "not on this line" })),
    /does not contain the documented text/u,
  );
  assert.throws(
    () =>
      matrixRuleOptions(
        project("//! vue/example\n//! Everything is reported by the default policy;"),
        map({ text: "reported by the default policy" }),
      ),
    /must spell eslint-plugin-vue's options/u,
  );
});

test("an override without a guard for Patina's default is refused", () => {
  assert.throws(
    () =>
      matrixRuleOptions(
        project(header, "fn unrelated() {}"),
        map({ text: "reported by the default policy" }),
      ),
    /keeps Patina's default reporting/u,
  );
});
