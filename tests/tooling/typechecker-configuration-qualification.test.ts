import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { gunzipSync } from "node:zlib";

import { typecheckerRunStatus } from "../../tools/support/compat/fixtures/typechecker-qualification.ts";
import { runTool } from "../../tools/support/compat/fixtures/tool-matrix-run.mjs";
import { validateTypecheckerOutput } from "../../tools/support/compat/fixtures/tool-matrix-typechecker.ts";
import { readAndValidateVizeRun } from "../../tools/support/compat/fixtures/typecheck-divergence-report-inputs.mjs";
import { reduceShards } from "../../tools/support/compat/davinci/lib/corpus-baseline-run.ts";

const configFile = "packages/frontend/editor-ui/tsconfig.json";
const root = fileURLToPath(new URL("../../", import.meta.url));
const evidence = new URL("./fixtures/typechecker-configuration/", import.meta.url);
const receipt = JSON.parse(fs.readFileSync(new URL("receipt.json", evidence), "utf8"));
const originalBytes = gunzipSync(fs.readFileSync(new URL("n8n-original.json.gz", evidence)));
const original = JSON.parse(originalBytes.toString("utf8"));

function parsed(files: readonly { file: string; diagnostics: readonly string[] }[]) {
  return { files };
}

void test("n8n configuration diagnostics do not qualify as semantic findings", () => {
  assert.equal(createHash("sha256").update(originalBytes).digest("hex"), receipt.payloadSha256);
  assert.deepEqual(JSON.parse(original.stdout), original.parsed);
  assert.deepEqual(
    original.parsed.files.filter((file: { diagnostics: string[] }) => file.diagnostics.length > 0),
    [receipt.diagnosticFile],
  );
  assert.deepEqual(
    validateTypecheckerOutput(
      { id: "n8n", expectedVueFileCount: receipt.requestedFileCount },
      original.parsed,
      original.exitCode,
      original.typecheckerCoverage.requested.files,
      original.typecheckerCoverage.checked.files,
    ),
    original.typecheckerCoverage,
  );
  assert.equal(typecheckerRunStatus(original.exitCode, original.parsed), "configuration");
});

void test("authored type errors qualify after configuration is prepared", () => {
  const packet = parsed([
    {
      file: "src/App.vue",
      diagnostics: ["error:1:1 [TS2322] Type 'string' is not assignable to type 'number'."],
    },
  ]);
  assert.equal(typecheckerRunStatus(1, packet), "findings");
  const mixed = parsed([...packet.files, receipt.diagnosticFile]);
  assert.equal(typecheckerRunStatus(1, mixed), "configuration");
  assert.deepEqual(mixed.files[0], packet.files[0]);
});

void test("project inputs and generated dependencies block qualification with exact codes", () => {
  for (const [file, diagnostic, expected] of [
    [
      "configs/build.jsonc",
      "error:1:1 [TS5023] Unknown compiler option 'missing'.",
      "configuration",
    ],
    [
      "src/App.vue",
      "error:1:1 [TS2688] Cannot find type definition file for 'vue'.",
      "configuration",
    ],
    ["src/App.vue", "error:1:1 [TS2307] Cannot find module '#imports'.", "configuration"],
    [
      "src/App.vue",
      "error:1:1 [TS6305] Output file has not been built from source file.",
      "configuration",
    ],
    ["src/App.vue", "error:1:1 [TS6059] File is not under 'rootDir'.", "configuration"],
    [
      "src/App.vue",
      "error:1:1 [TS6307] File is not listed within the file list of project.",
      "configuration",
    ],
    [
      "src/App.vue",
      "error:1:1 [TS2322] Type 'TS2688' is not assignable to type 'number'.",
      "findings",
    ],
    [configFile, "warning:1:1 [TS2688] A warning does not prevent semantic checking.", "findings"],
    ["src/App.vue", "hint:1:1 [TS2688] A hint does not prevent semantic checking.", "findings"],
  ] as const) {
    assert.equal(typecheckerRunStatus(1, parsed([{ file, diagnostics: [diagnostic] }])), expected);
  }
});

void test("matrix execution preserves full configuration packets and coverage", () => {
  const fixture = fs.mkdtempSync(path.join(root, "tests/_fixtures/qualification-"));
  const scratch = fs.mkdtempSync(path.join(os.tmpdir(), "vize-qualification-"));
  try {
    fs.mkdirSync(path.join(fixture, "src"));
    fs.writeFileSync(path.join(fixture, "src/App.vue"), "<template />\n");
    fs.writeFileSync(path.join(fixture, "tsconfig.json"), "{}\n");
    const output = {
      errorCount: 5,
      warningCount: 0,
      fileCount: 1,
      files: [{ file: "src/App.vue", diagnostics: [] }, receipt.diagnosticFile],
    };
    const executable = path.join(scratch, "vize.mjs");
    fs.writeFileSync(
      executable,
      `process.stdout.write(${JSON.stringify(JSON.stringify(output))}); process.exit(1);\n`,
    );
    const run = runTool(
      {
        id: "configuration",
        fixturePath: path.relative(root, fixture),
        vueGlobs: ["src/**/*.vue"],
        tsconfig: "tsconfig.json",
      },
      "typechecker",
      { dryRun: false, timeoutMs: 5_000 },
      { command: process.execPath, prefix: [executable], label: "fixture-vize" },
      scratch,
    );
    const payload = JSON.parse(
      fs.readFileSync(path.join(scratch, "configuration-typechecker.json"), "utf8"),
    );
    assert.deepEqual(payload, {
      schema: "vize.fixtureToolRun",
      version: 1,
      project: "configuration",
      tool: "typechecker",
      exitCode: 1,
      stdout: JSON.stringify(output),
      stderr: "",
      parsed: output,
      typecheckerCoverage: validateTypecheckerOutput(
        { id: "configuration" },
        output,
        1,
        ["src/App.vue"],
        ["src/App.vue"],
      ),
    });
    assert.deepEqual(
      { status: run.status, exitCode: run.exitCode, fileCount: run.fileCount },
      { status: "configuration", exitCode: 1, fileCount: 1 },
    );
    const project = {
      id: "configuration",
      fixturePath: path.relative(root, fixture),
      vueGlobs: ["src/**/*.vue"],
    };
    const summary = { projects: [{ id: project.id, runs: [run] }] };
    assert.throws(
      () => readAndValidateVizeRun(scratch, project, summary),
      new Error(
        "configuration-only typechecker run is not semantic qualification for configuration",
      ),
    );
    run.status = "findings";
    assert.throws(
      () => readAndValidateVizeRun(scratch, project, summary),
      new Error("Fixture matrix typechecker status is inconsistent for configuration"),
    );
    fs.writeFileSync(
      path.join(scratch, "summary.json"),
      JSON.stringify({
        ...summary,
        summary: {
          runCount: 1,
          okRuns: 0,
          findingsRuns: 1,
          failedRuns: 0,
          missingFixtureRuns: 0,
          plannedRuns: 0,
        },
      }),
    );
    assert.throws(
      () => reduceShards([scratch], ["typechecker"]),
      new Error(
        "configuration/typechecker payload cannot qualify semantic checking: unavailable project inputs",
      ),
    );
  } finally {
    fs.rmSync(fixture, { recursive: true, force: true });
    fs.rmSync(scratch, { recursive: true, force: true });
  }
});

void test("Rust matrix execution qualifies the actual producer route", () => {
  const tool = new URL("../../tools/commands/fixtures/tool-matrix-report.rs", import.meta.url);
  const result = spawnSync("rust-script", ["--test", fileURLToPath(tool)], { encoding: "utf8" });
  assert.equal(
    result.status,
    0,
    String(result.error ?? "") + String(result.stderr ?? "") + String(result.stdout ?? ""),
  );
});

void test("a clean packet stays ok only at exit zero", () => {
  const packet = parsed([{ file: "src/App.vue", diagnostics: [] }]);
  assert.equal(typecheckerRunStatus(0, packet), "ok");
  assert.equal(typecheckerRunStatus(1, packet), "findings");
});
