import assert from "node:assert/strict";
import fs from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";
import { test } from "node:test";

const repoRoot = path.resolve(import.meta.dirname, "../..");
const benchManifest = path.join(repoRoot, "tools", "benchmarks", "scripts", "package.json");
const requireFromBench = createRequire(benchManifest);
const { ESLint } = requireFromBench("eslint") as typeof import("eslint");
const pluginVue = requireFromBench("eslint-plugin-vue");
const vueParser = requireFromBench("vue-eslint-parser");
const typescriptParser = requireFromBench("@typescript-eslint/parser");
const cases = JSON.parse(
  fs.readFileSync(
    path.join(
      repoRoot,
      "crates",
      "vize_patina",
      "tests",
      "fixtures",
      "require-v-for-key-object-bind.json",
    ),
    "utf8",
  ),
) as Array<{ id: string; eslintDiagnostics: number; source: string }>;

test("require-v-for-key matrix stays pinned to eslint-plugin-vue 10.9.2", async () => {
  assert.equal(requireFromBench("eslint-plugin-vue/package.json").version, "10.9.2");
  assert.equal(requireFromBench("vue-eslint-parser/package.json").version, "10.4.1");

  const eslint = new ESLint({
    cwd: repoRoot,
    overrideConfigFile: true,
    overrideConfig: [
      {
        files: ["**/*.vue"],
        languageOptions: {
          parser: vueParser,
          parserOptions: {
            parser: typescriptParser,
            ecmaVersion: "latest",
            sourceType: "module",
          },
        },
        plugins: { vue: pluginVue },
        rules: { "vue/require-v-for-key": "error" },
      },
    ],
  });

  for (const fixture of cases) {
    const [result] = await eslint.lintText(fixture.source, {
      filePath: path.join(repoRoot, "oracle", `${fixture.id}.vue`),
    });
    const diagnostics = result.messages.filter(
      (message) => message.ruleId === "vue/require-v-for-key",
    );
    const parserErrors = result.messages.filter((message) => message.ruleId == null);

    assert.deepEqual(parserErrors, [], `${fixture.id}: parser errors`);
    assert.equal(diagnostics.length, fixture.eslintDiagnostics, fixture.id);
  }
});
