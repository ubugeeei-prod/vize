import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const toolPath = path.join(
  root,
  "tools",
  "commands",
  "fixtures",
  "real-project-surface-verdict.rs",
);
const realProjectSurfaceNames = [
  "waiver-audit",
  "typecheck-dependencies",
  "core-tools",
  "lsp",
  "lint-divergence",
  "syntax-highlighter",
  "glyph",
  "typecheck-divergence",
];

const successfulResults = realProjectSurfaceNames.map((name) => ({ name, outcome: "success" }));

function runVerdict(
  results: Array<{ name: string; outcome: string }>,
  env: NodeJS.ProcessEnv = {},
) {
  const outputDir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-surface-verdict-"));
  const output = path.join(outputDir, "surface-verdict.json");
  const result = spawnSync(
    "rust-script",
    [
      toolPath,
      "--output",
      output,
      ...results.flatMap(({ name, outcome }) => ["--surface", `${name}=${outcome}`]),
    ],
    { cwd: root, encoding: "utf8", env: { ...process.env, ...env } },
  );
  const verdict = fs.existsSync(output) ? JSON.parse(fs.readFileSync(output, "utf8")) : null;
  fs.rmSync(outputDir, { recursive: true, force: true });
  return { result, verdict };
}

function runWorkflowVerdict(env: NodeJS.ProcessEnv) {
  const outputDir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-surface-verdict-"));
  const output = path.join(outputDir, "surface-verdict.json");
  const result = spawnSync("rust-script", [toolPath, "--from-workflow-env", "--output", output], {
    cwd: root,
    encoding: "utf8",
    env: { ...process.env, ...env },
  });
  const verdict = fs.existsSync(output) ? JSON.parse(fs.readFileSync(output, "utf8")) : null;
  fs.rmSync(outputDir, { recursive: true, force: true });
  return { result, verdict };
}

test("the real-project surface verdict accepts only a complete successful set", () => {
  const { result, verdict } = runVerdict(successfulResults, {
    GITHUB_SHA: "0123456789abcdef",
    FIXTURE_SHARD_INDEX: "7",
  });
  assert.equal(result.status, 0, result.stderr);
  assert.equal(verdict.status, "success");
  assert.equal(verdict.sourceCommit, "0123456789abcdef");
  assert.equal(verdict.shardIndex, "7");
  assert.deepEqual(verdict.failedSurfaceNames, []);
  assert.deepEqual(verdict.surfaces, successfulResults);
});

for (const outcome of ["failure", "cancelled", "skipped"] as const) {
  test(`the real-project surface verdict fails closed on ${outcome}`, () => {
    const { result, verdict } = runVerdict(
      successfulResults.map((result) =>
        result.name === "core-tools" ? { ...result, outcome } : result,
      ),
    );
    assert.equal(result.status, 1);
    assert.equal(verdict.status, "failure");
    assert.deepEqual(verdict.failedSurfaceNames, ["core-tools"]);
  });
}

test("the real-project surface verdict rejects missing, duplicate, unknown, and empty outcomes", () => {
  for (const [results, message] of [
    [successfulResults.slice(1), /missing real-project surface verdict.*waiver-audit/],
    [[...successfulResults, successfulResults[0]], /duplicate real-project surface/],
    [
      [...successfulResults.slice(1), { name: "unknown", outcome: "success" }],
      /unknown real-project surface/,
    ],
    [
      successfulResults.map((result) =>
        result.name === "glyph" ? { ...result, outcome: "" } : result,
      ),
      /invalid outcome for glyph/,
    ],
  ] as const) {
    const { result, verdict } = runVerdict([...results]);
    assert.equal(result.status, 1);
    assert.equal(verdict, null);
    assert.match(result.stderr, message);
  }
});

test("workflow surface inputs preserve enforce modes and soften only record-only failures", () => {
  const { result, verdict } = runWorkflowVerdict({
    VIZE_WAIVER_AUDIT_OUTCOME: "success",
    TYPECHECK_DEPENDENCIES_MODE: "record-only",
    VIZE_TYPECHECK_DEPENDENCIES_OUTCOME: "failure",
    CORE_TOOLS_MODE: "enforce",
    VIZE_CORE_TOOLS_OUTCOME: "success",
    LSP_MODE: "record-only",
    VIZE_LSP_OUTCOME: "cancelled",
    LINT_DIVERGENCE_MODE: "record-only",
    VIZE_LINT_DIVERGENCE_OUTCOME: "failure",
    VIZE_SYNTAX_HIGHLIGHTER_OUTCOME: "success",
    VIZE_GLYPH_OUTCOME: "success",
    TYPECHECK_DIVERGENCE_MODE: "record-only",
    VIZE_TYPECHECK_DIVERGENCE_OUTCOME: "failure",
  });
  assert.equal(result.status, 1);

  assert.deepEqual(verdict.surfaces, [
    { name: "waiver-audit", outcome: "success" },
    { name: "typecheck-dependencies", outcome: "success" },
    { name: "core-tools", outcome: "success" },
    { name: "lsp", outcome: "cancelled" },
    { name: "lint-divergence", outcome: "success" },
    { name: "syntax-highlighter", outcome: "success" },
    { name: "glyph", outcome: "success" },
    { name: "typecheck-divergence", outcome: "success" },
  ]);
});
