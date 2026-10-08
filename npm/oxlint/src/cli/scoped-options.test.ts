import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import test from "node:test";

import { getLintTargets, withoutLintTargets } from "./args.ts";
import { prepareScopedSelection } from "./scoped-selection.ts";

void test("pinned scalar options retain each whole value and literal target separator", () => {
  const pairs = [
    ["-A", "all"],
    ["--allow", "all"],
    ["-D", "correctness"],
    ["--deny", "correctness"],
    ["-W", "restriction"],
    ["--warn", "restriction"],
    ["-c", "App.vue"],
    ["--config", "App.vue"],
    ["-f", "json"],
    ["--format", "json"],
    ["--debug", "timings"],
    ["--ignore-path", "Ignored.vue"],
    ["--ignore-pattern", "[slug].vue"],
    ["--max-warnings", "2"],
    ["--threads", "1"],
    ["--tsconfig", "Project.vue"],
    ["--report-unused-disable-directives-severity", "warn"],
  ];
  for (const pair of pairs) {
    assert.deepEqual(getLintTargets([...pair, "app/Original.vue"]), ["app/Original.vue"]);
    assert.deepEqual(withoutLintTargets([...pair, "app", "--", "-strange.vue"]), [...pair, "--"]);
    if (pair[0].startsWith("--")) {
      const equal = `${pair[0]}=${pair[1]}`;
      assert.deepEqual(withoutLintTargets([equal, "app"]), [equal]);
    }
  }
  assert.deepEqual(
    withoutLintTargets(["--allow", "all", "--config", "config.json", "--vue-plugin", "app"]),
    ["--allow", "all", "--config", "config.json", "--vue-plugin"],
  );
});

void test("pinned boolean and plugin switches never consume the following original target", () => {
  const flags = [
    "-h",
    "-V",
    "--help",
    "--version",
    "--init",
    "--fix",
    "--fix-suggestions",
    "--fix-dangerously",
    "--quiet",
    "--deny-warnings",
    "--no-ignore",
    "--silent",
    "--no-error-on-unmatched-pattern",
    "--print-config",
    "--rules",
    "--lsp",
    "--disable-nested-config",
    "--type-aware",
    "--type-check",
    "--type-check-only",
    "--suppress-all",
    "--prune-suppressions",
    "--report-unused-disable-directives",
    "--disable-unicorn-plugin",
    "--disable-oxc-plugin",
    "--disable-typescript-plugin",
    "--import-plugin",
    "--react-plugin",
    "--jsdoc-plugin",
    "--jest-plugin",
    "--vitest-plugin",
    "--jsx-a11y-plugin",
    "--nextjs-plugin",
    "--react-perf-plugin",
    "--promise-plugin",
    "--node-plugin",
    "--vue-plugin",
  ];
  for (const flag of flags) {
    assert.deepEqual(getLintTargets([flag, "Original.vue"]), ["Original.vue"]);
    assert.deepEqual(withoutLintTargets([flag, "Original.vue"]), [flag]);
  }
});

void test("unknown, compact and missing-value forms refuse rather than mutate argv", () => {
  for (const flag of [
    "--future-option",
    "--future-option=value",
    "-Acorrectness",
    "-fjson",
    "--vue-plugin=true",
  ])
    assert.throws(() => withoutLintTargets([flag, "Original.vue"]), {
      message: `Scoped Vue transport cannot preserve unsupported option form: ${flag}`,
    });
  for (const option of ["--allow", "--report-unused-disable-directives-severity"])
    for (const suffix of [[], ["--", "Original.vue"]])
      assert.throws(() => withoutLintTargets([option, ...suffix]), {
        message: `Scoped Vue transport requires a value for ${option}.`,
      });
});

void test("alternate inspection and suppression modes refuse before stock selection or owned writes", async (t) => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-scoped-modes-"));
  t.after(() => fs.rmSync(root, { recursive: true, force: true }));
  const file = path.join(root, "Original.vue");
  fs.writeFileSync(file, "<template />\n");
  const config = path.join(root, "config.json");
  fs.writeFileSync(config, '{"ignorePatterns":[]}\n');
  for (const args of [
    ["--debug", "files"],
    ["--debug=timings"],
    ["--rules"],
    ["--lsp"],
    ["--print-config"],
    ["--init"],
    ["-h"],
    ["--help"],
    ["-V"],
    ["--version"],
    ["--suppress-all"],
    ["--prune-suppressions"],
  ]) {
    await assert.rejects(
      prepareScopedSelection(
        root,
        ["-c", config, ...args, file],
        [file],
        async () => {
          throw new Error("Unexpected stock selection");
        },
        new Set([file]),
      ),
      {
        message: "Scoped Vue transport cannot combine linting with other Oxlint inspection modes.",
      },
    );
    assert.deepEqual(fs.readdirSync(root).sort(), ["Original.vue", "config.json"]);
    assert.equal(fs.readFileSync(file, "utf8"), "<template />\n");
    assert.equal(fs.readFileSync(config, "utf8"), '{"ignorePatterns":[]}\n');
  }
});
