import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { nativePreparationIsActive } from "../../native/scripts/test-preparation.mjs";
import { loadCases, wholeJson } from "../../../tests/tooling/boolean-attribute-fix-reference.mjs";

const root = path.resolve(fileURLToPath(new URL("../../..", import.meta.url)));
const nativeDir = path.join(root, "npm/native");
assert.ok(
  nativePreparationIsActive(nativeDir),
  "this exact source's live native build is required",
);
const native = createRequire(import.meta.url)(path.join(nativeDir, "index.js"));
const { cases, directory } = loadCases(root);
const original = cases[0];
const expected =
  original.source.slice(0, original.source.indexOf("<template>")) +
  fs.readFileSync(path.join(directory, "original-requested-all-four.vue.fixture"), "utf8");
const fixture = path.join(root, "target/vize-tests/template-style-fixes-7905");
const file = path.join(fixture, "CardList.vue");
const capturePath = path.join(root, "target/template-style-fixes-7905.json");
const plugin = path.join(root, "npm/oxlint/dist/index.mjs");
const cli = path.join(root, "npm/oxlint/dist/cli.mjs");
const digest = (text) => createHash("sha256").update(text).digest("hex");
const capture = { schema: "vize.template-style-fixes-7905.v1", complete: false, observations: [] };
function save(observation) {
  capture.observations.push(observation);
  fs.writeFileSync(capturePath, JSON.stringify(capture, null, 2) + "\n");
}
const findings = [
  ["vue/html-self-closing", "<MyCard>", "Empty component should be self-closing"],
  ["vue/component-name-in-template-casing", "<my-card />", "Component should use PascalCase"],
  ["vue/v-slot-style", "v-slot:header", "Expected '#header' instead of 'v-slot:header'"],
  [
    "vue/no-boolean-attr-value",
    'disabled="disabled"',
    'Boolean attribute "disabled" should not have value "disabled"',
  ],
];
function position(source, index) {
  const lines = source.slice(0, index).split("\n");
  return {
    line: lines.length,
    column: lines.at(-1).length + 1,
    offset: Buffer.byteLength(source.slice(0, index)),
  };
}
const diagnostics = findings.map(([rule, target, message]) => {
  const start = original.source.indexOf(target);
  assert.ok(start >= 0);
  return {
    rule,
    severity: "warning",
    message,
    location: {
      start: position(original.source, start),
      end: position(original.source, start + target.length),
    },
    help: null,
  };
});
const ordered = (items) =>
  [...items].sort((a, b) => (a.rule ?? a.code).localeCompare(b.rule ?? b.code));
function publicNativeReport(after) {
  const report = wholeJson({ ...original, filename: file }, after, true);
  const source = after ? expected : original.source;
  const target = after ? "<input disabled />" : '<input disabled="disabled" />';
  const start = source.indexOf(target);
  assert.equal(source.indexOf(target, start + 1), -1);
  assert.ok(start >= 0);
  const from = position(source, start);
  const to = position(source, start + target.length);
  // Opinionated also enables the unchanged, nonfixable accessibility rule.
  // Preserve its complete finding instead of suppressing or fixing reporter input.
  report[0].messages.splice(after ? 0 : 3, 0, {
    ruleId: "a11y/form-control-has-label",
    ruleDocsPath: "docs/content/rules/accessibility.md",
    severity: 1,
    message: "[vize:a11y/form-control-has-label] <input> elements must have an associated label",
    line: from.line,
    column: from.column,
    endLine: to.line,
    endColumn: to.column,
  });
  report[0].warningCount += 1;
  return report;
}
const config =
  JSON.stringify(
    {
      categories: { correctness: "off" },
      plugins: ["vue"],
      jsPlugins: [plugin],
      settings: { vize: { helpLevel: "none", preset: "incremental" } },
      rules: Object.fromEntries(findings.map(([rule]) => [`vize/${rule}`, "warn"])),
    },
    null,
    2,
  ) + "\n";
fs.rmSync(fixture, { recursive: true, force: true });
fs.mkdirSync(path.join(fixture, ".git"), { recursive: true });
fs.writeFileSync(path.join(fixture, ".oxlintrc.json"), config);
fs.mkdirSync(path.join(fixture, "node_modules/oxlint/bin"), { recursive: true });
fs.symlinkSync(
  path.join(root, "node_modules/oxlint/bin/oxlint"),
  path.join(fixture, "node_modules/oxlint/bin/oxlint"),
);
const env = { ...process.env, VIZE_PREFER_WORKSPACE_BINDING: "1" };
delete env.GITHUB_ACTIONS;
try {
  save({
    original: original.source,
    expected,
    config,
    pluginSha256: digest(fs.readFileSync(plugin)),
    cliSha256: digest(fs.readFileSync(cli)),
    nativePreparation: JSON.parse(
      fs.readFileSync(path.join(nativeDir, ".artifacts/native/js-test-preparation.json"), "utf8"),
    ),
  });
  // The real plugin diagnoses the complete original file; this does not claim plugin writeback.
  for (const [source, wanted] of [
    [original.source, diagnostics],
    ...Array.from({ length: 3 }, () => [expected, []]),
  ]) {
    fs.writeFileSync(file, source);
    const actualNative = native.lintPatinaSfc(source, {
      filename: "CardList.vue",
      preset: "incremental",
      helpLevel: "none",
      enabledRules: findings.map(([rule]) => rule),
    });
    assert.deepEqual(
      { ...actualNative, diagnostics: ordered(actualNative.diagnostics) },
      {
        filename: "CardList.vue",
        errorCount: 0,
        warningCount: wanted.length,
        diagnostics: ordered(wanted),
      },
    );
    const actual = spawnSync(process.execPath, [cli, "-f", "json", "CardList.vue"], {
      cwd: fixture,
      env,
      encoding: "utf8",
      maxBuffer: 8 * 1024 * 1024,
    });
    save({
      producer: "public diagnostics and production plugin",
      source,
      native: actualNative,
      status: actual.status,
      signal: actual.signal,
      error: actual.error?.message ?? null,
      stdout: actual.stdout,
      stderr: actual.stderr,
      file: fs.readFileSync(file, "utf8"),
    });
    assert.equal(actual.error, undefined);
    assert.equal(actual.signal, null);
    assert.equal(actual.status, 0);
    assert.ok(
      [
        "",
        "WARNING: JS plugins are experimental and not subject to semver.\nBreaking changes are possible while JS plugins support is under development.\n",
      ].includes(actual.stderr),
      actual.stderr,
    );
    const report = JSON.parse(actual.stdout);
    assert.deepEqual(Object.keys(report).sort(), [
      "diagnostics",
      "number_of_files",
      "number_of_rules",
      "start_time",
      "threads_count",
    ]);
    assert.equal(report.number_of_files, 1);
    assert.ok(Number.isSafeInteger(report.number_of_rules) && report.number_of_rules > 0);
    assert.ok(Number.isSafeInteger(report.threads_count) && report.threads_count >= 0);
    assert.ok(Number.isFinite(report.start_time) && report.start_time >= 0);
    assert.deepEqual(
      ordered(report.diagnostics),
      ordered(
        wanted.map((d) => ({
          message: d.message,
          code: `vize(${d.rule})`,
          severity: "warning",
          filename: "CardList.vue",
          labels: [
            {
              span: {
                offset: d.location.start.offset,
                length: d.location.end.offset - d.location.start.offset,
                line: d.location.start.line,
                column: d.location.start.column,
              },
            },
          ],
        })),
      ),
    );
    assert.equal(fs.readFileSync(file, "utf8"), source);
  }
  fs.writeFileSync(file, original.source);
  for (let pass = 0; pass < 4; pass++) {
    const actual = native.lint([file], {
      format: "json",
      preset: "opinionated",
      helpLevel: "none",
      fix: pass > 0,
    });
    const bytes = fs.readFileSync(file, "utf8");
    save({
      producer: "source-built public native lint",
      pass,
      actual,
      bytes,
      sha256: digest(bytes),
    });
    assert.deepEqual(Object.keys(actual).sort(), [
      "errorCount",
      "fileCount",
      "output",
      "timeMs",
      "warningCount",
    ]);
    assert.equal(actual.errorCount, 0);
    assert.equal(actual.warningCount, pass === 0 ? 5 : 1);
    assert.equal(actual.fileCount, 1);
    assert.ok(Number.isFinite(actual.timeMs) && actual.timeMs >= 0);
    assert.deepEqual(JSON.parse(actual.output), publicNativeReport(pass > 0));
    assert.equal(bytes, pass === 0 ? original.source : expected);
    assert.equal(fs.readFileSync(path.join(fixture, ".oxlintrc.json"), "utf8"), config);
  }
  assert.equal(
    native.getPatinaRules().find((rule) => rule.name === "vapor/require-vapor-attribute").fixable,
    false,
  );
  const vapor = native.lintPatinaSfc(original.source, {
    filename: "CardList.vue",
    preset: "incremental",
    enabledRules: ["vapor/require-vapor-attribute"],
  });
  save({ producer: "unfinished Vapor placeholder", source: original.source, actual: vapor });
  assert.deepEqual(vapor, {
    filename: "CardList.vue",
    errorCount: 0,
    warningCount: 0,
    diagnostics: [],
  });
  capture.complete = true;
  save({ terminal: "whole original/public native/plugin/fixed-byte/idempotence controls passed" });
} finally {
  fs.rmSync(fixture, { recursive: true, force: true });
}
