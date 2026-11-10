// Davinci P4-12c: Patina lints `<template lang="pug">` through the pug
// dialect, but the ESLint baseline (vue-eslint-parser) cannot parse pug, so
// the lint-divergence runner reports Patina findings inside a pug template
// body as outside the comparable surface instead of scoring them as false
// positives. Findings elsewhere in the same SFC are still compared.
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";
import { test } from "node:test";
import {
  baselineConfig,
  resolveBaselineRuntime,
} from "../../tools/support/compat/fixtures/lint-divergence-baseline.mjs";

const repoRoot = path.resolve(import.meta.dirname, "..", "..");
const parityRoot = path.join(repoRoot, "tests/_fixtures/pug-jade-surface-audit-8908");

function sha256(source: string): string {
  return createHash("sha256").update(source).digest("hex");
}

function gitBlobSha(source: string): string {
  const body = Buffer.from(source);
  return createHash("sha1").update(`blob ${body.byteLength}\0`).update(body).digest("hex");
}

function signature(
  messages: Array<{ ruleId?: string | null; severity?: number; message?: string }>,
) {
  const counts = new Map<string, number>();
  for (const message of messages) {
    if (message.ruleId == null || message.message === undefined) continue;
    const row = JSON.stringify({
      ruleId: message.ruleId,
      severity: message.severity === 2 ? "error" : "warning",
      message: message.message,
    });
    counts.set(row, (counts.get(row) ?? 0) + 1);
  }
  return [...counts]
    .map(([row, count]) => ({ ...JSON.parse(row), count }))
    .sort((left, right) => JSON.stringify(left).localeCompare(JSON.stringify(right)));
}

test("the TypeScript runner keeps pug template findings out of the scored surface", () => {
  fs.mkdirSync(path.join(repoRoot, "target"), { recursive: true });
  const fixtureDir = fs.mkdtempSync(path.join(repoRoot, "target", "lint-pug-fixture-"));
  const outputDir = fs.mkdtempSync(path.join(repoRoot, "target", "lint-pug-report-"));
  try {
    fs.writeFileSync(
      path.join(fixtureDir, "App.vue"),
      [
        '<template lang="pug">',
        "ul",
        '  li(v-for="item in items") {{ item }}',
        "</template>",
        "<script>",
        "export default { data: () => ({ items: [1] }) };",
        "</script>",
        "",
      ].join("\n"),
    );
    const fakeVize = path.join(fixtureDir, "fake-vize.mjs");
    fs.writeFileSync(
      fakeVize,
      [
        "#!/usr/bin/env node",
        "if (process.argv[2] !== 'lint') process.exit(2);",
        "process.stdout.write(JSON.stringify([{",
        '  file: "App.vue",',
        "  messages: [{",
        '    ruleId: "vue/require-v-for-key",',
        '    severity: "error",',
        "    line: 3, column: 6, endLine: 3, endColumn: 27,",
        "    message: \"[vize:vue/require-v-for-key] Elements in iteration expect to have 'v-bind:key' directives.\",",
        "  }],",
        "}]));",
        "",
      ].join("\n"),
    );
    fs.chmodSync(fakeVize, 0o755);
    const registryPath = path.join(fixtureDir, "registry.json");
    fs.writeFileSync(
      registryPath,
      JSON.stringify({
        projects: [
          {
            id: "lint-pug-fixture",
            revision: "0".repeat(40),
            fixturePath: path.relative(repoRoot, fixtureDir),
            vueGlobs: ["App.vue"],
            coverage: ["linter"],
          },
        ],
      }),
    );

    const result = spawnSync(
      process.execPath,
      [
        "tools/commands/fixtures/lint-divergence-report.ts",
        "--registry",
        registryPath,
        "--output-dir",
        outputDir,
        "--vize-bin",
        fakeVize,
        "--budget-mode",
        "enforce",
        "--timeout-ms",
        "30000",
      ],
      { cwd: repoRoot, encoding: "utf8", env: { ...process.env, LANG: "C", LC_ALL: "C" } },
    );

    assert.equal(result.status, 0, `${result.stdout}\n${result.stderr}`);
    const artifact = JSON.parse(
      fs.readFileSync(path.join(outputDir, "lint-pug-fixture-lint-divergence.json"), "utf8"),
    );
    const summary = artifact.divergence.summary;
    assert.equal(summary.falsePositiveCount, 0);
    assert.equal(summary.patinaOutsideBaselineSurfaceCount, 1);
    assert.equal(summary.patinaFindingCount, 1);
    assert.deepEqual(
      artifact.divergence.patinaOutsideBaselineSurface.map(
        (entry: { ruleId: string; line: number }) => [entry.ruleId, entry.line],
      ),
      [["vue/require-v-for-key", 3]],
    );
    assert.equal(artifact.budget.passed, true);
  } finally {
    fs.rmSync(fixtureDir, { recursive: true, force: true });
    fs.rmSync(outputDir, { recursive: true, force: true });
  }
});

test("a pinned dho Pug project measures eslint-plugin-vue parity through its HTML twin", async () => {
  const pugPath = path.join(parityRoot, "dho-current-balance.pug.vue.txt");
  const htmlPath = path.join(parityRoot, "dho-current-balance.html.vue.txt");
  const evidence = JSON.parse(
    fs.readFileSync(path.join(parityRoot, "dho-current-balance-lint-parity.json"), "utf8"),
  );
  const pugSource = fs.readFileSync(pugPath, "utf8");
  const htmlSource = fs.readFileSync(htmlPath, "utf8");
  assert.equal(sha256(pugSource), evidence.source.sha256);
  assert.equal(gitBlobSha(pugSource), evidence.source.blobSha);
  assert.equal(sha256(htmlSource), evidence.renderer.htmlTwinSha256);

  const requireFromTests = createRequire(path.join(repoRoot, "tests/package.json"));
  const pug = requireFromTests("pug") as {
    render(source: string, options: Record<string, unknown>): string;
  };
  const pugVersion = requireFromTests("pug/package.json").version;
  assert.equal(pugVersion, evidence.renderer.version);
  const template = /<template lang="pug">\n([\s\S]*?)\n<\/template>/u.exec(pugSource);
  assert.ok(template);
  const rendered = pug.render(template[1], {
    filename: pugPath,
    doctype: "html",
    pretty: false,
  });
  assert.equal(
    pugSource.replace(template[0], `<template>${rendered}</template>`),
    htmlSource,
    "the ESLint input must be the pinned Pug rendering, not a hand-authored twin",
  );

  const runtime = resolveBaselineRuntime();
  assert.equal(runtime.version, evidence.baseline.version);
  const eslint = new runtime.ESLint({
    cwd: repoRoot,
    overrideConfigFile: true,
    overrideConfig: baselineConfig(runtime, { "vue/require-v-for-key": "error" }),
  });
  const [result] = await eslint.lintText(htmlSource, {
    filePath: path.join(repoRoot, "target/dho-current-balance.vue"),
  });
  assert.ok(result);
  assert.deepEqual(signature(result.messages), evidence.signatures);
});
