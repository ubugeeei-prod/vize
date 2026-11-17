import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

import {
  matrixRuleOptions,
  writeMatrixRuleOptionsConfig,
} from "../../tools/support/fixtures/lint_divergence/matrix_rule_options.ts";

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
  const config = writeMatrixRuleOptionsConfig(overrides);
  assert.ok(config);
  assert.match(
    fs.readFileSync(config, "utf8"),
    /"vue\/v-on-handler-style":\["method","inline-function"\]/u,
  );
  fs.rmSync(path.dirname(config), { recursive: true, force: true });
  assert.equal(writeMatrixRuleOptionsConfig([]), null);
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
