import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";

import { collectVueInputPaths } from "../../../tools/support/compat/fixtures/tool-matrix-inputs.mjs";
import { root } from "./lsp/paths.ts";
import { LspSession } from "./lsp/session.ts";
import { exerciseLspLifecycle } from "./real-project-lsp-audit.ts";
import { assertOracleFilesAreInCorpus } from "./real-project-lsp-authored-oracle.ts";
import { hasCompleteAuthoredFeatureEvidence } from "./real-project-lsp-report-authored.ts";
import {
  emptyLspEvidence,
  type FixtureProject,
  type LspAuthoredOracle,
} from "./real-project-lsp-report.ts";

assert.equal(process.env.VIZE_LSP_REQUIRE_SOURCE_BUILD, "1");
const fixture = path.join(root, "tests/_fixtures/differential/lsp/n8n-registry-authored-3952");
const output = path.resolve(process.argv[2] ?? "target/n8n-cli-config/authored-registry");
fs.mkdirSync(output, { recursive: true });
const provenance = JSON.parse(fs.readFileSync(path.join(fixture, "provenance.json"), "utf8")) as {
  revision: string;
  registryBeforeSha256: string;
  originalProjectCount: number;
  originalAuthoredProjectCount: number;
  files: Array<{ fixture: string; upstreamPath: string; sha256: string; bytes: number }>;
};
const oracle = JSON.parse(
  fs.readFileSync(path.join(fixture, "oracle.json"), "utf8"),
) as LspAuthoredOracle;
const before = fs.readFileSync(path.join(fixture, "registry-before.json.txt"));
assert.equal(createHash("sha256").update(before).digest("hex"), provenance.registryBeforeSha256);
const liveRegistry = fs.readFileSync(
  path.join(root, "tests/_fixtures/vue-ecosystem-fixtures.json"),
);
assert.deepEqual(
  liveRegistry,
  before,
  "observe current behavior before changing the original registry",
);
const registry = JSON.parse(liveRegistry.toString()) as {
  lspAuthoredOracleGate: { minimumProjectCount: number };
  projects: FixtureProject[];
};
assert.equal(registry.projects.length, provenance.originalProjectCount);
assert.equal(
  registry.projects.filter((entry) => entry.lspAuthoredOracle != null).length,
  provenance.originalAuthoredProjectCount,
);
assert.equal(
  registry.lspAuthoredOracleGate.minimumProjectCount,
  provenance.originalAuthoredProjectCount,
);
const project = registry.projects.find((entry) => entry.id === "n8n");
assert.ok(project);
assert.equal(
  project.lspAuthoredOracle,
  undefined,
  "n8n registration remains held until current packets pass",
);
assert.equal(project.revision, provenance.revision);
const startedAt = Date.now();
const deadline = startedAt + 600_000;
const remainingMs = () => {
  const remaining = deadline - Date.now();
  assert.ok(remaining > 0, "LSP fixture shard exceeded 600000ms");
  return Math.min(remaining, 120_000);
};
const workspace = path.join(root, project.fixturePath);
assert.equal(
  execFileSync("git", ["-C", workspace, "rev-parse", "HEAD"], { encoding: "utf8" }).trim(),
  provenance.revision,
);
const files = collectVueInputPaths(workspace, project.vueGlobs);
assertOracleFilesAreInCorpus(project, files, oracle);
const sha256 = (bytes: Buffer) => createHash("sha256").update(bytes).digest("hex");
const write = (file: string, value: unknown) =>
  fs.writeFileSync(path.join(output, file), `${JSON.stringify(value, null, 2)}\n`);
const originalInputs = provenance.files.map((input) => {
  const bytes = fs.readFileSync(path.join(workspace, input.upstreamPath));
  assert.equal(bytes.length, input.bytes);
  assert.equal(sha256(bytes), input.sha256);
  assert.deepEqual(bytes, fs.readFileSync(path.join(fixture, input.fixture)));
  const target = path.join(output, "original-inputs", input.upstreamPath);
  fs.mkdirSync(path.dirname(target), { recursive: true });
  fs.writeFileSync(target, bytes);
  return { file: input.upstreamPath, bytes: bytes.length, sha256: sha256(bytes) };
});
const checkout = execFileSync("git", ["rev-parse", "HEAD"], { cwd: root, encoding: "utf8" }).trim();
write("source-inputs-before.json", {
  checkout,
  n8nRevision: provenance.revision,
  originalInputs,
  oracle,
});

// This is the existing ecosystem provider, with its original requests,
// deadlines, dependency edit and create/rename/delete/repair assertions.
// Registration stays separate from observing the current producer's behavior.
const session = new LspSession();
const evidence = emptyLspEvidence(project);
evidence.vueFileCount = files.length;
evidence.actualFile = oracle.templateBinding.file;
write("process.json", { checkout, pid: session.processId });
let failure: unknown;
try {
  const result = await exerciseLspLifecycle(
    session,
    workspace,
    oracle.templateBinding.file,
    remainingMs,
    oracle,
  );
  Object.assign(evidence, result);
  assert.ok(hasCompleteAuthoredFeatureEvidence(evidence));
} catch (error) {
  failure = error;
  evidence.failure = String(error);
} finally {
  try {
    const after = originalInputs.map((input) => {
      const bytes = fs.readFileSync(path.join(workspace, input.file));
      return { file: input.file, bytes: bytes.length, sha256: sha256(bytes) };
    });
    write("source-inputs-after.json", after);
    assert.deepEqual(
      after,
      originalInputs,
      "the provider must retain all original n8n source bytes",
    );
    for (const relative of [oracle.fileLifecycle.copiedFile, oracle.fileLifecycle.renamedFile]) {
      assert.equal(
        fs.existsSync(path.join(workspace, relative)),
        false,
        "the lifecycle must remove its reserved files",
      );
    }
  } catch (error) {
    evidence.failure = [evidence.failure, `Source custody: ${String(error)}`]
      .filter(Boolean)
      .join("\n");
    failure ??= error;
  }
  evidence.status = failure ? "failed" : "ok";
  evidence.durationMs = Date.now() - startedAt;
  fs.writeFileSync(path.join(output, "stderr.txt"), session.stderrText);
  write("complete.json", evidence);
}
if (failure) {
  // Shutdown has observed child exit; let its existing close/stdio handlers
  // finish the raw capture before Node ends this failed provider process.
  console.error(failure);
  process.exitCode = 1;
} else {
  process.stdout.write(
    "n8n: existing authored LSP and file-lifecycle provider passed on original pinned sources\n",
  );
}
