import assert from "node:assert/strict";
import { execFileSync, spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

import {
  typecheckProvider,
  verifyTypecheckProviderSource,
} from "../../tools/support/compat/fixtures/typecheck-baseline-provider.mjs";
import { typecheckPreparationIdentity } from "../../tools/support/compat/fixtures/typecheck-input-overlays.ts";
import { readAndValidateDependencyPreparation } from "../../tools/support/compat/fixtures/typecheck-divergence-provenance.mjs";
import { installArguments } from "../../tools/support/compat/fixtures/typecheck-package-manager.ts";

import { processResultArgs } from "./support/program-path.ts";
import { retainFixtureCommand } from "./support/typecheck-fixture-custody.ts";

test("fixture custody preserves binary failure streams and owned inputs before cleanup", () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-custody-test-"));
  try {
    const fixture = path.join(root, "fixture");
    fs.mkdirSync(fixture);
    const original = Buffer.from([0, 255, 128, 10]);
    fs.writeFileSync(path.join(fixture, "original.bin"), original);
    const args = [...processResultArgs({ stdout: [0, 255, 10], stderr: [128, 0], status: 1 })];
    const result = spawnSync(process.execPath, args, { encoding: "buffer" });
    const evidence = retainFixtureCommand(
      path.join(root, "retained"),
      { executable: process.execPath, args, cwd: root },
      result,
      { fixture },
    );
    fs.rmSync(fixture, { recursive: true });
    assert.equal(result.status, 1);
    assert.deepEqual(fs.readFileSync(path.join(evidence, "stdout.bin")), Buffer.from([0, 255, 10]));
    assert.deepEqual(fs.readFileSync(path.join(evidence, "stderr.bin")), Buffer.from([128, 0]));
    assert.deepEqual(fs.readFileSync(path.join(evidence, "fixture/original.bin")), original);
    const receipt = JSON.parse(fs.readFileSync(path.join(evidence, "command.json"), "utf8"));
    assert.deepEqual(receipt.command, { executable: process.execPath, args, cwd: root });
    assert.deepEqual(receipt.result, {
      pid: result.pid,
      status: 1,
      signal: null,
      error: null,
      stdoutCaptured: true,
      stderrCaptured: true,
    });
    const files = JSON.parse(fs.readFileSync(path.join(evidence, "files.json"), "utf8"));
    assert.deepEqual(
      files.map((file: { path: string }) => file.path),
      ["stdout.bin", "stderr.bin", "command.json", "fixture/original.bin"],
    );
    for (const file of files) {
      const bytes = fs.readFileSync(path.join(evidence, file.path));
      assert.equal(bytes.length, file.sizeBytes);
      assert.equal(createHash("sha256").update(bytes).digest("hex"), file.sha256);
    }
    assert.equal(fs.existsSync(path.join(evidence, "fixture/missing-report.json")), false);
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

test("fixture custody retains spawn errors without assuming a report exists", () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-custody-error-"));
  try {
    const executable = path.join(root, "missing-executable");
    const result = spawnSync(executable, [], { encoding: "buffer" });
    const evidence = retainFixtureCommand(
      path.join(root, "retained"),
      { executable, args: [], cwd: root, timeoutMs: 20_000 },
      result,
      {},
    );
    const receipt = JSON.parse(fs.readFileSync(path.join(evidence, "command.json"), "utf8"));
    assert.equal(receipt.result.status, null);
    assert.equal(receipt.result.error.code, "ENOENT");
    assert.equal(receipt.result.error.message, result.error!.message);
    assert.equal(receipt.result.error.stack, result.error!.stack);
    assert.equal(receipt.result.stdoutCaptured, false);
    assert.equal(receipt.result.stderrCaptured, false);
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

test("the unpublished oracle is explicit and never substitutes an unknown provider", () => {
  assert.equal(typecheckProvider(undefined), null);
  assert.throws(() => typecheckProvider("latest"), /Unknown pinned/);
  const provider = typecheckProvider("vue-language-tools-6371");
  assert.equal(provider.revision, "b77bba35e9fa78b4bdab6baf8588a4e4c9469ce7");
  assert.equal(provider.typescriptPeerVersion, "6.0.3");
  assert.match(provider.reason, /not qualification against the unmodified published/);
  assert.deepEqual(provider.install, [
    "--filter",
    "vue-tsc...",
    "--filter",
    "{.}",
    "install",
    "--frozen-lockfile",
    "--ignore-scripts",
    "--prefer-offline",
  ]);
});

test("source-provider evidence rejects a changed tree, revision or sealed source blob", () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-baseline-provider-"));
  const git = (...args: string[]) =>
    execFileSync("git", args, { cwd: root, encoding: "utf8" }).trim();
  try {
    git("init", "-q");
    fs.writeFileSync(path.join(root, "pnpm-lock.yaml"), "lockfileVersion: '9.0'\n");
    git("add", "pnpm-lock.yaml");
    git(
      "-c",
      "user.name=Fixture",
      "-c",
      "user.email=fixture@example.invalid",
      "commit",
      "-qm",
      "test: seal oracle source",
    );
    const revision = git("rev-parse", "HEAD");
    const provider = {
      revision,
      sourceBlobs: { "pnpm-lock.yaml": git("rev-parse", "HEAD:pnpm-lock.yaml") },
    };
    assert.doesNotThrow(() => verifyTypecheckProviderSource(root, provider));
    const original = fs.readFileSync(path.join(root, "pnpm-lock.yaml"));
    fs.writeFileSync(path.join(root, "pnpm-lock.yaml"), "lockfileVersion: '10.0'\n");
    assert.throws(() => verifyTypecheckProviderSource(root, provider), /authored source changed/);
    fs.writeFileSync(path.join(root, "pnpm-lock.yaml"), original);
    assert.throws(
      () =>
        verifyTypecheckProviderSource(root, {
          ...provider,
          revision: "0".repeat(40),
        }),
      /revision differs/,
    );
    assert.throws(
      () =>
        verifyTypecheckProviderSource(root, {
          ...provider,
          sourceBlobs: { "pnpm-lock.yaml": "0".repeat(40) },
        }),
      /source blob differs/,
    );
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

test("provider descriptors participate in fixture preparation custody", () => {
  const revision = "a".repeat(40);
  assert.equal(typecheckPreparationIdentity(revision, undefined), revision);
  const prepared = typecheckPreparationIdentity(revision, {
    provider: "vue-language-tools-6371",
    prepare: ["pnpm", "run", "build"],
  });
  assert.notEqual(prepared, revision);
  assert.notEqual(
    prepared,
    typecheckPreparationIdentity(revision, {
      provider: "vue-language-tools-6371",
      prepare: ["pnpm", "run", "other"],
    }),
  );
});

test("source-provider verification still executes through a linked tooling directory", () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-provider-linked-cli-"));
  try {
    const tools = fileURLToPath(new URL("../../tools", import.meta.url));
    const link = path.join(root, "tools");
    fs.symlinkSync(tools, link, process.platform === "win32" ? "junction" : "dir");
    const executable = path.join(link, "support/compat/fixtures/typecheck-baseline-provider.mjs");
    const result = spawnSync(process.execPath, [executable, "invalid"], { encoding: "utf8" });
    assert.equal(result.status, 1);
    assert.match(result.stderr, /Expected provider verify inputs/);
    assert.equal(result.stdout, "");
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

test("preparation provenance accepts sealed optional fields and rejects unknown evidence", () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-optional-preparation-"));
  const hash = (bytes: string) => createHash("sha256").update(bytes).digest("hex");
  try {
    const lockfile = "{}\n";
    fs.writeFileSync(path.join(root, "package-lock.json"), lockfile);
    const schema = "type Query { answer: Int! }\n";
    fs.writeFileSync(path.join(root, "schema.gql"), schema);
    const input = {
      source: "tests/_fixtures/typecheck-inputs/project/schema.gql",
      target: "schema.gql",
      sha256: hash(schema),
    };
    const baseline = { prepare: ["npm", "run", "codegen"], inputOverlays: [input] };
    const project = {
      id: "fixture",
      revision: "b".repeat(40),
      typecheckPerformance: {
        packageManager: "npm",
        packageManagerVersion: "11.0.0",
        lockfile: "package-lock.json",
        baseline,
      },
    };
    const preparation = {
      command: baseline.prepare,
      durationMs: 1,
      exitCode: 0,
      stdoutSha256: hash(""),
      stderrSha256: hash(""),
      inputOverlays: [{ ...input, sizeBytes: Buffer.byteLength(schema) }],
    };
    const artifact = {
      schema: "vize.fixtureTypecheckDependencyInstall",
      version: 3,
      project: project.id,
      revision: project.revision,
      evidence: { commitSha: "a".repeat(40) },
      packageManager: { name: "npm", version: "11.0.0" },
      lockfile: {
        path: "package-lock.json",
        sizeBytes: Buffer.byteLength(lockfile),
        sha256: hash(lockfile),
      },
      qualificationLockfile: null,
      install: {
        command: ["npm", ...installArguments("npm", root)],
        durationMs: 1,
        exitCode: 0,
        stdoutSha256: hash(""),
        stderrSha256: hash(""),
      },
      baselinePrepare: preparation,
    };
    const verify = (baselinePrepare: unknown, provider?: string) => {
      fs.writeFileSync(
        path.join(root, "fixture-typecheck-dependencies.json"),
        JSON.stringify({ ...artifact, baselinePrepare }),
      );
      return readAndValidateDependencyPreparation({
        reportDir: root,
        fixtureRoot: root,
        commitSha: artifact.evidence.commitSha,
        project: {
          ...project,
          typecheckPerformance: {
            ...project.typecheckPerformance,
            baseline: { ...baseline, ...(provider === undefined ? {} : { provider }) },
          },
        },
      });
    };
    assert.deepEqual(verify(preparation).baselinePrepare, {
      command: baseline.prepare,
      exitCode: 0,
      stdoutSha256: hash(""),
      stderrSha256: hash(""),
    });
    assert.throws(() => verify({ ...preparation, provider: {} }, "unknown"), {
      message: "Unknown pinned typecheck baseline provider",
    });
    assert.throws(() => verify({ ...preparation, unexpected: true }), {
      message:
        "typecheck baseline preparation evidence keys must be command, durationMs, exitCode, inputOverlays, stderrSha256, stdoutSha256; received command, durationMs, exitCode, inputOverlays, stderrSha256, stdoutSha256, unexpected",
    });
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});
