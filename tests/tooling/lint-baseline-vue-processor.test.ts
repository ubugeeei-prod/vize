import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { createRequire } from "node:module";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { fileURLToPath } from "node:url";
import { test } from "node:test";

const root = fileURLToPath(new URL("../../", import.meta.url));
const benchPackageJson = join(root, "tools/benchmarks/scripts/package.json");
const requireFromBench = createRequire(benchPackageJson);
const { ESLint } = requireFromBench("eslint");
const plugin = requireFromBench("eslint-plugin-vue");
const vueParser = requireFromBench("vue-eslint-parser");
const scriptParser = requireFromBench("@typescript-eslint/parser");
const rust = readFileSync(join(root, "tools/commands/fixtures/lint-divergence-report.rs"), "utf8");
const start = rust.indexOf("fn run_eslint_baseline(");
assert.ok(start >= 0);
const match = /let script = r#"([\s\S]*?)"#;/u.exec(rust.slice(start));
assert.ok(match, "Use the actual canonical embedded MJS collector");
const collector = match[1];
const rules = {
  "vue/comment-directive": 1,
  "vue/jsx-uses-vars": 1,
  "vue/no-v-html": 1,
  "vue/no-v-text": 2,
};
const cases = [
  {
    name: "ordinary full diagnostics retain their coordinates and severities",
    text: '<template>\n<div v-html="html" />\n<p v-text="text" />\n</template>\n',
    expectedRules: ["vue/no-v-html", "vue/no-v-text"],
  },
  {
    name: "next-line directive suppresses its rule while an unrelated diagnostic remains",
    text: '<template>\n<!-- eslint-disable-next-line vue/no-v-html -->\n<div v-html="html" />\n<p v-text="text" />\n</template>\n',
    expectedRules: ["vue/no-v-text"],
  },
  {
    name: "same-line directive ends before the next genuine diagnostic",
    text: '<template>\n<div v-html="html" /> <!-- eslint-disable-line vue/no-v-html -->\n<div v-html="html" />\n</template>\n',
    expectedRules: ["vue/no-v-html"],
  },
  {
    name: "block disable and enable retain the later genuine diagnostic",
    text: '<template>\n<!-- eslint-disable vue/no-v-html -->\n<div v-html="html" />\n<!-- eslint-enable vue/no-v-html -->\n<div v-html="html" />\n<p v-text="text" />\n</template>\n',
    expectedRules: ["vue/no-v-html", "vue/no-v-text"],
  },
  {
    name: "unqualified next-line directive consumes all controls for only that line",
    text: '<template>\n<!-- eslint-disable-next-line -->\n<div v-html="html" v-text="text" />\n<p v-text="text" />\n</template>\n',
    expectedRules: ["vue/no-v-text"],
  },
  {
    name: "a real parser error keeps its complete original diagnostic",
    text: "<script>const = ;</script>\n<template><div /></template>\n",
    expectedRules: [null],
  },
];

function canonical(cwd: string, script = collector) {
  return JSON.parse(
    execFileSync(process.execPath, ["--input-type=module", "--eval", script], {
      input: JSON.stringify({ benchPackageJson, cwd, files: ["Original.vue"], rules }),
      encoding: "utf8",
      maxBuffer: 4 * 1024 * 1024,
    }),
  );
}

async function ordinaryProvider(cwd: string) {
  const eslint = new ESLint({
    cwd,
    overrideConfigFile: true,
    overrideConfig: [
      ...plugin.configs["flat/base"],
      {
        files: ["**/*.vue"],
        languageOptions: {
          parser: vueParser,
          parserOptions: {
            parser: scriptParser,
            ecmaVersion: "latest",
            sourceType: "module",
            ecmaFeatures: { jsx: true },
            extraFileExtensions: [".vue"],
          },
        },
        linterOptions: { reportUnusedDisableDirectives: "off" },
        rules,
      },
    ],
    errorOnUnmatchedPattern: false,
  });
  return JSON.parse(JSON.stringify(await eslint.lintFiles(["Original.vue"])));
}

for (const fixture of cases) {
  test("canonical Vue baseline: " + fixture.name, async () => {
    const cwd = mkdtempSync(join(tmpdir(), "vize-eslint-vue-processor-"));
    try {
      writeFileSync(join(cwd, "Original.vue"), fixture.text);
      const actual = canonical(cwd);
      const reference = await ordinaryProvider(cwd);
      assert.deepEqual(actual.results, reference, "Complete ordinary provider result equality");
      assert.equal(actual.version, "10.9.2");
      assert.equal(actual.droppedConfigMessageCount, 0);
      const messages = actual.results[0].messages;
      assert.deepEqual(
        messages.map((message: { ruleId: string | null }) => message.ruleId),
        fixture.expectedRules,
      );
      assert.ok(messages.every((message: { column: number }) => message.column > 0));
      assert.equal(readFileSync(join(cwd, "Original.vue"), "utf8"), fixture.text);
    } finally {
      rmSync(cwd, { recursive: true, force: true });
    }
  });
}

test("canonical Vue baseline: the original missing processor exposes genuine internal controls", () => {
  const cwd = mkdtempSync(join(tmpdir(), "vize-eslint-vue-missing-processor-"));
  try {
    const line = '    processor: "vue/vue",\n';
    assert.equal(collector.split(line).length, 2);
    writeFileSync(join(cwd, "Original.vue"), cases[1].text);
    const actual = canonical(cwd, collector.replace(line, ""));
    const messages = actual.results[0].messages;
    assert.ok(
      messages.some(
        (message: { ruleId: string; column: number }) =>
          message.ruleId === "vue/comment-directive" && message.column === 0,
      ),
    );
    assert.ok(messages.some((message: { ruleId: string }) => message.ruleId === "vue/no-v-html"));
    assert.equal(readFileSync(join(cwd, "Original.vue"), "utf8"), cases[1].text);
  } finally {
    rmSync(cwd, { recursive: true, force: true });
  }
});
