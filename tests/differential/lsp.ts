import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { pathToFileURL } from "node:url";
import { expectedBuildIdentity, validateBuildReceipt } from "./build-receipt.mjs";
import { assertExactRows, sha256 } from "./harness.mjs";
import { INITIALIZE_CAPABILITIES, loadLspManifest, NATIVE_REASON } from "./lsp-manifest.ts";
import { inspectLspObservation, validateLspReport } from "./lsp-report.ts";
import { LspWire } from "./lsp-wire.ts";
import {
  errorText,
  hasCapabilities,
  type LspFixture,
  type LspRow,
  type LspReport,
  type BuildIdentity,
  type BuildReceipt,
} from "./lsp-types.ts";

export { loadLspManifest, validateLspReport };

async function runLspCase(
  fixture: LspFixture,
  binaryPath: string | undefined,
  binaryFailure?: string,
): Promise<LspRow> {
  const row: LspRow = {
    id: fixture.id,
    legacy: { state: "failed", verdict: "failed", comparator: "complete-jsonrpc-response" },
    native: { state: "unsupported", reason: NATIVE_REASON },
    comparison: { state: "not-compared", reason: NATIVE_REASON },
  };
  let workspace;
  let wire;
  try {
    if (binaryFailure) throw new Error(binaryFailure);
    assert(typeof binaryPath === "string");
    workspace = fs.mkdtempSync(path.join(os.tmpdir(), "vize-differential-lsp-"));
    for (const file of fixture.files) {
      const destination = path.join(workspace, file.runtimePath);
      fs.mkdirSync(path.dirname(destination), { recursive: true });
      fs.writeFileSync(destination, file.bytes);
    }
    workspace = fs.realpathSync(workspace);
    const workspaceUri = pathToFileURL(workspace).href;
    const entryUri = pathToFileURL(path.join(workspace, fixture.entry)).href;
    wire = new LspWire(binaryPath, ["lsp"], workspace);
    const initialized = await wire.request("initialize", {
      processId: process.pid,
      rootUri: workspaceUri,
      capabilities: INITIALIZE_CAPABILITIES,
      initializationOptions: fixture.initializationOptions,
      workspaceFolders: [{ uri: workspaceUri, name: path.basename(workspace) }],
    });
    assert(hasCapabilities(initialized), "initialization must succeed");
    wire.notify("initialized", {});
    wire.notify("textDocument/didOpen", {
      textDocument: {
        uri: entryUri,
        languageId: "vue",
        version: fixture.data.documentVersion,
        text: fixture.files
          .find((file) => file.runtimePath === fixture.entry)
          ?.bytes.toString("utf8"),
      },
    });
    await wire.waitFor(
      (message) =>
        message.method === "textDocument/publishDiagnostics" &&
        message.params?.uri === entryUri &&
        message.params?.version === fixture.data.documentVersion &&
        Array.isArray(message.params?.diagnostics),
    );
    for (const request of fixture.requests) {
      await wire.request(fixture.data.method, {
        textDocument: { uri: entryUri },
        ...request.params,
      });
    }
    await wire.finish();
    row.legacy.observation = { ...wire.observation(), workspaceUri };
    for (const stream of ["client", "server"] as const) {
      row.legacy.observation[`${stream}WireSha256`] = sha256(
        Buffer.from(row.legacy.observation[`${stream}WireBase64`], "base64"),
      );
    }
    row.legacy.responses = inspectLspObservation(fixture, row.legacy.observation);
    row.legacy.state = "completed";
    row.legacy.verdict = row.legacy.responses.every((response) => response.state === "equal")
      ? "matched-reference"
      : "baseline-drift";
  } catch (error) {
    row.legacy.error = errorText(error);
  } finally {
    if (wire) {
      await wire.stop().catch((error) => {
        row.legacy.state = "failed";
        row.legacy.verdict = "failed";
        row.legacy.error = error.message;
      });
      if (!row.legacy.observation) row.legacy.observation = wire.observation();
    }
    if (workspace) fs.rmSync(workspace, { recursive: true, force: true });
  }
  return row;
}

export async function runLspPack({
  manifestPath,
  binaryPath,
  sourceRevision,
  repoRoot,
}: {
  manifestPath: string;
  binaryPath?: string;
  sourceRevision: string;
  repoRoot?: string;
}): Promise<LspReport> {
  const loaded = loadLspManifest(manifestPath);
  assert.match(sourceRevision, /^[a-f0-9]{40}$/);
  const report: LspReport = {
    schema: "vize.differential.result",
    version: 1,
    product: "lsp",
    sourceRevision,
    manifestSha256: loaded.manifestSha256,
    argv: ["lsp"],
    buildReceipt: null,
    binary: null,
    rows: [],
    summary: {
      plannedCases: loaded.cases.length,
      legacyMatches: 0,
      legacyFailures: 0,
      baselineDrift: 0,
      nativeUnsupported: loaded.cases.length,
      pairedComparisons: 0,
      nativeHandled: 0,
      nativeEquivalent: 0,
    },
  };
  let build: BuildIdentity = { sourceRevision };
  let binaryFailure;
  try {
    assert(typeof binaryPath === "string", "explicit source-built executable required");
    assert(path.isAbsolute(binaryPath), "absolute executable path required; no PATH fallback");
    assert(fs.statSync(binaryPath).isFile(), "executable must be a file");
    fs.accessSync(binaryPath, fs.constants.X_OK);
    assert(typeof repoRoot === "string", "source checkout is required");
    const observedBuild = expectedBuildIdentity(repoRoot);
    build = observedBuild;
    assert.equal(build.sourceRevision, sourceRevision, "checkout changed before LSP execution");
    assert.equal(
      fs.realpathSync(binaryPath),
      fs.realpathSync(path.join(repoRoot, observedBuild.binaryPath)),
    );
    const receipt = JSON.parse(
      fs.readFileSync(`${binaryPath}.differential-build.json`, "utf8"),
    ) as BuildReceipt;
    validateBuildReceipt(receipt, build);
    const version = spawnSync(binaryPath, ["--version"], { timeout: 30_000 });
    report.binaryProbe = {
      exitStatus: version.status,
      signal: version.signal,
      stdoutBase64: version.stdout?.toString("base64") ?? "",
      stderrBase64: version.stderr?.toString("base64") ?? "",
      processError: version.error?.message ?? null,
    };
    assert.equal(version.status, 0, "source-built executable version probe must succeed");
    assert.equal(version.signal, null);
    assert.equal(version.error, undefined);
    assert.equal(version.stdout.toString().trim(), build.cliVersion);
    report.buildReceipt = receipt;
    report.binary = {
      path: observedBuild.binaryPath,
      sha256: observedBuild.binarySha256,
      version: observedBuild.cliVersion,
    };
  } catch (error) {
    binaryFailure = errorText(error);
  }
  for (const fixture of loaded.cases) {
    report.rows.push(await runLspCase(fixture, binaryPath, binaryFailure));
  }
  assertExactRows(loaded, report.rows);
  report.summary = {
    plannedCases: loaded.cases.length,
    legacyMatches: report.rows.filter((row) => row.legacy.verdict === "matched-reference").length,
    legacyFailures: report.rows.filter((row) => row.legacy.state === "failed").length,
    baselineDrift: report.rows.filter((row) => row.legacy.verdict === "baseline-drift").length,
    nativeUnsupported: loaded.cases.length,
    pairedComparisons: 0,
    nativeHandled: 0,
    nativeEquivalent: 0,
  };
  validateLspReport(loaded, report, build);
  return report;
}
