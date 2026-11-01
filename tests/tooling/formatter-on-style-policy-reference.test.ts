// Ordinary Actions executes this real pinned provider; no native parity is asserted.
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { createRequire } from "node:module";
import { dirname, join } from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { asRecord } from "../../tools/support/json-value.ts";

const root = fileURLToPath(new URL("../../", import.meta.url));
interface ReferenceLinter {
  verify(source: string, config: unknown[], options: { filename: string }): unknown[];
  verifyAndFix(source: string, config: unknown[], options: { filename: string }): unknown;
}

test("pinned official v-on longform fixes the complete original configured SFC and is a fixed point", () => {
  const require = createRequire(join(root, "tools/benchmarks/scripts/package.json"));
  for (const [name, version] of Object.entries({
    eslint: "10.4.1",
    "eslint-plugin-vue": "10.9.2",
    "vue-eslint-parser": "10.4.1",
    "@typescript-eslint/parser": "8.65.0",
  })) {
    assert.equal(asRecord(require(name + "/package.json")).version, version);
  }
  const pluginRoot = dirname(require.resolve("eslint-plugin-vue/package.json"));
  assert.equal(
    createHash("sha256")
      .update(readFileSync(join(pluginRoot, "dist/rules/v-on-style.js")))
      .digest("hex"),
    "286179656ee0f4db8d55e3bf476803385d6c2ee7e54ee3f1332c7cac4da46500",
  );
  const Linter = asRecord(require("eslint")).Linter as new () => ReferenceLinter;
  const linter = new Linter();
  const config = [
    {
      files: ["**/*.vue"],
      languageOptions: {
        parser: require("vue-eslint-parser"),
        parserOptions: {
          parser: require("@typescript-eslint/parser"),
          ecmaVersion: 2022,
          sourceType: "module",
        },
      },
      plugins: { vue: require("eslint-plugin-vue") },
      rules: { "vue/v-on-style": ["error", "longform"] },
    },
  ];
  const directory = join(
    root,
    "tests/_fixtures/differential/linter/formatter-on-style-policy-6098",
  );
  const source = readFileSync(join(directory, "Original.vue.txt"), "utf8");
  const fixed = readFileSync(join(directory, "Fixed.vue.txt"), "utf8");
  const findings = linter.verify(source, config, { filename: "src/Events.vue" });
  const repaired = linter.verifyAndFix(source, config, { filename: "src/Events.vue" });
  const clean = linter.verify(fixed, config, { filename: "src/Events.vue" });
  const again = linter.verifyAndFix(fixed, config, { filename: "src/Events.vue" });
  console.log(JSON.stringify({ source, findings, repaired, clean, again }));
  assert.equal(findings.length, 1);
  assert.deepEqual(repaired, { fixed: true, messages: [], output: fixed });
  assert.deepEqual(clean, []);
  assert.deepEqual(again, { fixed: false, messages: [], output: fixed });
});
