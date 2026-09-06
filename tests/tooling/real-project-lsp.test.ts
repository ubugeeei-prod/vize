import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import type { LspDiagnostic, PublishDiagnosticsParams } from "./support/lsp/protocol.ts";
import {
  exerciseLspLifecycle,
  runRealProjectLspAudit,
  selectProjects,
} from "./support/real-project-lsp-audit.ts";
import { createLspReport } from "./support/real-project-lsp-report.ts";
import type { AuthoredLspEvidence, LspProjectEvidence } from "./support/real-project-lsp-report.ts";

test("production LSP opens and repairs a probe in every selected real project", async () => {
  const result = await runRealProjectLspAudit();
  if (result.skipped) return;
  assert.ok(result.report.summary.projectCount > 0);
  assert.equal(result.report.summary.failedProjectCount, 0);
  const summary = result.report.summary;
  assert.equal(
    summary.authoredFeatureProjectCount + summary.missingAuthoredFeatureProjectIds.length,
    summary.projectCount,
  );
  for (const id of summary.missingAuthoredFeatureProjectIds) {
    assert.ok(
      result.evidence.some((project) => project.id === id),
      `${id} is reported as missing authored features but is not an audited project`,
    );
  }
});

test("LSP report summarizes authored feature coverage from project evidence", () => {
  const report = createLspReport(
    { enforced: true, projects: [], shardCount: 2, shardIndex: 1 },
    [
      projectEvidence("with-oracle", authoredEvidence()),
      projectEvidence("missing-oracle", null),
      projectEvidence("also-missing-oracle", null),
    ],
    { GITHUB_SHA: "0".repeat(40) },
  );

  assert.equal(report.summary.projectCount, 3);
  assert.equal(report.version, 5);
  assert.equal(report.commitSha, "0".repeat(40));
  assert.equal(report.evidence.commitSha, "0".repeat(40));
  assert.deepEqual(report.evidence.runtime, { name: "node", version: process.versions.node });
  assert.equal(report.summary.authoredFeatureProjectCount, 1);
  assert.equal(report.summary.authoredAnchorCount, 6);
  assert.deepEqual(report.summary.missingAuthoredFeatureProjectIds, [
    "missing-oracle",
    "also-missing-oracle",
  ]);
});

test("LSP report refuses to count incomplete authored feature evidence", () => {
  const missingAnchorEvidence = authoredEvidence();
  missingAnchorEvidence.authoredAnchors = { count: 0, sha256: "1".repeat(64) };
  const missingHoverEvidence = authoredEvidence();
  missingHoverEvidence.hover = { count: 0, durationMs: 0, sha256: "2".repeat(64) };
  const report = createLspReport(
    { enforced: true, projects: [], shardCount: 1, shardIndex: 0 },
    [
      projectEvidence("complete-oracle", authoredEvidence()),
      projectEvidence("missing-anchor-evidence", missingAnchorEvidence),
      projectEvidence("missing-hover-evidence", missingHoverEvidence),
    ],
    { GITHUB_SHA: "0".repeat(40) },
  );

  assert.equal(report.summary.authoredFeatureProjectCount, 1);
  assert.equal(report.summary.authoredAnchorCount, 6);
  assert.deepEqual(report.summary.missingAuthoredFeatureProjectIds, [
    "missing-anchor-evidence",
    "missing-hover-evidence",
  ]);
});

test("LSP report records resolved server binary provenance", () => {
  const workspace = fs.mkdtempSync(path.join(os.tmpdir(), "vize-lsp-report-provenance-"));
  try {
    const binary = path.join(workspace, "vize");
    fs.writeFileSync(binary, "fake vize binary");
    const report = createLspReport(
      { enforced: true, projects: [], shardCount: 1, shardIndex: 0 },
      [],
      { GITHUB_SHA: "1".repeat(40), VIZE_LSP_BIN: binary },
      { resolveLaunchCommand: () => [binary, "lsp"] },
    );
    const relativeBinary = path.relative(process.cwd(), binary);
    assert.deepEqual(report.evidence.vizeBinary, {
      command: [binary, "lsp"],
      path: relativeBinary.startsWith("..") ? binary : relativeBinary,
      sha256: createHash("sha256").update("fake vize binary").digest("hex"),
    });
  } finally {
    fs.rmSync(workspace, { recursive: true, force: true });
  }
});

test("real-project LSP lifecycle opens an authored file and restores exact diagnostics", async () => {
  const workspace = fs.mkdtempSync(path.join(os.tmpdir(), "vize-real-project-lsp-"));
  fs.writeFileSync(path.join(workspace, "App.vue"), "<template><main /></template>\n");
  const session = new FakeLspSession();
  try {
    const result = await exerciseLspLifecycle(session, workspace, "App.vue");
    assert.deepEqual(result.actualFileDiagnostics, {
      count: 0,
      sha256: "01ba4719c80b6fe911b091a7c05124b64eeece964e09c058ef8f9805daca546b",
    });
    assert.equal(result.fixedProbeDiagnostics.count, 0);
    assert.equal(result.brokenProbeDiagnostics.count, 1);
    assert.deepEqual(result.repairedProbeDiagnostics, result.fixedProbeDiagnostics);
    assert.deepEqual(session.events, [
      "initialize",
      "textDocument/didOpen:1:App.vue",
      "textDocument/publishDiagnostics:1:App.vue",
      "textDocument/didOpen:1:.vize-fixture-lsp-probe.vue",
      "textDocument/publishDiagnostics:1:.vize-fixture-lsp-probe.vue",
      "textDocument/didChange:2:.vize-fixture-lsp-probe.vue",
      "textDocument/publishDiagnostics:2:.vize-fixture-lsp-probe.vue",
      "textDocument/didChange:3:.vize-fixture-lsp-probe.vue",
      "textDocument/publishDiagnostics:3:.vize-fixture-lsp-probe.vue",
      "textDocument/didClose:.vize-fixture-lsp-probe.vue",
      "textDocument/didClose:App.vue",
      "shutdown",
    ]);
  } finally {
    fs.rmSync(workspace, { recursive: true, force: true });
  }
});

test("real-project LSP lifecycle rejects diagnostic drift after repair", async () => {
  const session = new FakeLspSession(true);
  await assert.rejects(
    () => exerciseLspLifecycle(session, os.tmpdir(), null),
    /repair must restore the fixed diagnostic baseline exactly/,
  );
  assert.equal(session.events.at(-1), "shutdown");
});

test("real-project LSP shards are strict, balanced, and fail closed", () => {
  const projects = Array.from({ length: 5 }, (_, index) => fixtureProject(`project-${index}`));
  const selected = selectProjects(projects, {
    FIXTURE_SHARD_COUNT: "2",
    FIXTURE_SHARD_INDEX: "1",
  });
  assert.deepEqual(
    selected.projects.map((project) => project.id),
    ["project-1", "project-3"],
  );
  assert.throws(
    () => selectProjects(projects, { FIXTURE_SHARD_COUNT: "2" }),
    /must be set together/,
  );
  assert.throws(
    () =>
      selectProjects(projects, {
        FIXTURE_SHARD_COUNT: "2",
        FIXTURE_SHARD_INDEX: "2",
      }),
    /must be less than/,
  );
  assert.throws(
    () =>
      selectProjects(projects, {
        FIXTURE_SHARD_COUNT: "8",
        FIXTURE_SHARD_INDEX: "7",
      }),
    /selected no projects/,
  );
});

class FakeLspSession {
  readonly events: string[] = [];
  private readonly documents = new Map<string, { text: string; version: number }>();
  private readonly driftAfterRepair: boolean;

  constructor(driftAfterRepair = false) {
    this.driftAfterRepair = driftAfterRepair;
  }

  async initialize(): Promise<unknown> {
    this.events.push("initialize");
    return {};
  }

  async request(): Promise<unknown> {
    throw new Error("fake lifecycle session received an unexpected request");
  }

  notify(method: string, params: unknown): void {
    const payload = params as {
      contentChanges?: Array<{ text: string }>;
      textDocument?: { text?: string; uri: string; version?: number };
    };
    const document = payload.textDocument;
    const file = document == null ? "" : path.basename(new URL(document.uri).pathname);
    if (method === "textDocument/didOpen" && document != null) {
      this.documents.set(document.uri, {
        text: document.text ?? "",
        version: document.version ?? 0,
      });
      this.events.push(`${method}:${document.version}:${file}`);
    } else if (method === "textDocument/didChange" && document != null) {
      this.documents.set(document.uri, {
        text: payload.contentChanges?.[0]?.text ?? "",
        version: document.version ?? 0,
      });
      this.events.push(`${method}:${document.version}:${file}`);
    } else if (method === "textDocument/didClose" && document != null) {
      this.events.push(`${method}:${file}`);
      this.documents.delete(document.uri);
    }
  }

  async waitForNotification(
    method: string,
    predicate?: (params: unknown) => boolean,
  ): Promise<unknown> {
    assert.equal(method, "textDocument/publishDiagnostics");
    const [uri, document] = [...this.documents.entries()].at(-1) ?? [];
    assert.ok(uri && document);
    const diagnostics = document.text.includes('= "broken"')
      ? [injectedMismatch()]
      : this.driftAfterRepair && document.version === 3
        ? [unrelatedDiagnostic()]
        : [];
    const payload: PublishDiagnosticsParams = {
      diagnostics,
      uri,
      version: document.version,
    };
    assert.ok(predicate?.(payload), "fake notification must satisfy the requested predicate");
    this.events.push(`${method}:${document.version}:${path.basename(new URL(uri).pathname)}`);
    return payload;
  }

  async shutdown(): Promise<void> {
    this.events.push("shutdown");
  }
}

function injectedMismatch(): LspDiagnostic {
  return {
    code: 2322,
    message: "Type 'string' is not assignable to type 'number'.",
    range: {
      start: { line: 1, character: 6 },
      end: { line: 1, character: 32 },
    },
    severity: 1,
    source: "vize/types",
  };
}

function unrelatedDiagnostic(): LspDiagnostic {
  return {
    code: 2304,
    message: "Cannot find name 'drift'.",
    range: {
      start: { line: 0, character: 0 },
      end: { line: 0, character: 5 },
    },
    severity: 1,
    source: "vize/types",
  };
}

function projectEvidence(
  id: string,
  authoredFeatures: AuthoredLspEvidence | null,
): LspProjectEvidence {
  return {
    actualFile: null,
    actualFileDiagnostics: null,
    authoredFeatures,
    brokenProbeDiagnostics: null,
    durationMs: 0,
    fixedProbeDiagnostics: null,
    id,
    repairedProbeDiagnostics: null,
    revision: "0".repeat(40),
    status: "ok",
    vueFileCount: 1,
  };
}

function authoredEvidence(): AuthoredLspEvidence {
  const diagnostic = { count: 0, sha256: "0".repeat(64) };
  const response = () => ({ count: 1, durationMs: 0, sha256: "0".repeat(64) });
  return {
    authoredAnchors: { count: 6, sha256: "0".repeat(64) },
    completion: response(),
    componentDefinition: response(),
    componentFile: "FeatureChild.vue",
    definition: response(),
    dependencyCompletion: {
      baselineContainsProbe: false,
      changedContainsProbe: true,
      repairedContainsProbe: false,
    },
    fileLifecycle: {
      copiedFile: "Copied.vue",
      createdDefinition: response(),
      createdWorkspaceSymbols: response(),
      deletedDefinition: { ...response(), count: 0 },
      deletedDocumentSymbols: { ...response(), count: 0 },
      deletedImporterDiagnostics: diagnostic,
      deletedWorkspaceSymbols: { ...response(), count: 0 },
      repairedDiagnostics: diagnostic,
      renameEdit: response(),
      renamedDefinition: response(),
      renamedFile: "Renamed.vue",
      renamedWorkspaceSymbols: response(),
      restoredDefinition: response(),
      staleCopiedDocumentSymbols: { ...response(), count: 0 },
    },
    hover: response(),
    importerFile: "FeatureParent.vue",
    prepareRename: response(),
    references: response(),
    rename: response(),
    templateBindingFile: "Binding.vue",
  };
}

function fixtureProject(id: string) {
  return {
    coverage: ["lsp"],
    expectedVueFileCount: 1,
    fixturePath: `tests/_fixtures/_git/${id}`,
    id,
    revision: "0".repeat(40),
    vueGlobs: ["**/*.vue"],
  };
}
