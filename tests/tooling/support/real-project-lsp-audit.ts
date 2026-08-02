import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

import { collectVueInputPaths } from "../../../tools/fixtures/tool-matrix-inputs.mjs";
import { isDiagnosticsForUri, offsetToPosition } from "./lsp/assertions.ts";
import type {
  LspDiagnostic,
  LspInitializationOptions,
  PublishDiagnosticsParams,
} from "./lsp/protocol.ts";
import { LspSession } from "./lsp/session.ts";
import {
  assertFixtureFileCount,
  createLspReport,
  emptyLspEvidence,
  errorMessage,
  isHydrated,
  positiveInteger,
  selectProjects,
  writeLspReport,
  type DiagnosticEvidence,
  type FixtureProject,
  type LspProjectEvidence,
} from "./real-project-lsp-report.ts";

export { selectProjects } from "./real-project-lsp-report.ts";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../../..");
const registryPath = path.join(root, "tests", "_fixtures", "vue-ecosystem-fixtures.json");
const defaultShardTimeoutMs = 600_000;
const diagnosticsTimeoutMs = 120_000;
const probeName = "__vize_fixture_lsp_probe__";
const fixedProbeSource = `<script setup lang="ts">
const ${probeName}: number = 1
</script>
<template>{{ ${probeName} }}</template>
`;
const brokenProbeSource = fixedProbeSource.replace("= 1", '= "broken"');

type LspAuditSession = {
  initialize(workspaceDir: string, options?: LspInitializationOptions): Promise<unknown>;
  notify(method: string, params: unknown): void;
  shutdown(): Promise<void>;
  waitForNotification(
    method: string,
    predicate?: (params: unknown) => boolean,
    timeoutMs?: number,
  ): Promise<unknown>;
};

type AuditDependencies = {
  createSession?: () => LspAuditSession;
  now?: () => number;
};

export async function runRealProjectLspAudit(
  environment: NodeJS.ProcessEnv = process.env,
  dependencies: AuditDependencies = {},
) {
  const registry = JSON.parse(fs.readFileSync(registryPath, "utf8")) as {
    projects: FixtureProject[];
    requiredToolCoverage: string[];
  };
  assert.ok(registry.requiredToolCoverage.includes("lsp"));
  const selection = selectProjects(registry.projects, environment);
  if (!selection.enforced && selection.projects.length === 0) return { skipped: true } as const;

  const now = dependencies.now ?? Date.now;
  const timeoutMs = positiveInteger(
    environment.REAL_PROJECT_LSP_TIMEOUT_MS ?? String(defaultShardTimeoutMs),
    "REAL_PROJECT_LSP_TIMEOUT_MS",
  );
  const deadline = now() + timeoutMs;
  const evidence: LspProjectEvidence[] = [];

  for (const project of selection.projects) {
    assertBeforeDeadline(deadline, timeoutMs, now);
    const startedAt = now();
    const projectEvidence = emptyLspEvidence(project);
    try {
      assert.ok(project.coverage.includes("lsp"), `${project.id} does not declare lsp coverage`);
      const fixtureDir = path.resolve(root, project.fixturePath);
      assert.ok(isHydrated(fixtureDir), `${project.id} fixture is not hydrated`);
      const files = collectVueInputPaths(fixtureDir, project.vueGlobs);
      assertFixtureFileCount(project, files);
      projectEvidence.vueFileCount = files.length;
      projectEvidence.actualFile = files[0] ?? null;
      const remainingMs = () => {
        const remaining = deadline - now();
        assert.ok(remaining > 0, `LSP fixture shard exceeded ${timeoutMs}ms`);
        return Math.min(remaining, diagnosticsTimeoutMs);
      };
      const result = await exerciseLspLifecycle(
        dependencies.createSession?.() ?? new LspSession(),
        fixtureDir,
        files[0] ?? null,
        remainingMs,
      );
      Object.assign(projectEvidence, result, { status: "ok" as const });
    } catch (error) {
      projectEvidence.failure = errorMessage(error);
    }
    projectEvidence.durationMs = now() - startedAt;
    evidence.push(projectEvidence);
  }

  const failed = evidence.filter((project) => project.status === "failed");
  const report = createLspReport(selection, evidence, environment);
  writeLspReport(report, environment);
  assert.deepEqual(
    failed.map((project) => `${project.id}: ${project.failure}`),
    [],
  );
  process.stderr.write(
    `LSP lifecycle: ${report.summary.projectCount} project(s), ` +
      `${report.summary.actualFileCount} actual file(s), ` +
      `${report.summary.failedProjectCount} failed project(s)\n`,
  );
  return { evidence, report, skipped: false } as const;
}

export async function exerciseLspLifecycle(
  session: LspAuditSession,
  workspaceDir: string,
  actualFile: string | null,
  timeoutMs: () => number = () => diagnosticsTimeoutMs,
) {
  const openUris: string[] = [];
  let primaryError: unknown = null;
  let shutdownError: unknown = null;
  let result: {
    actualFileDiagnostics: DiagnosticEvidence | null;
    brokenProbeDiagnostics: DiagnosticEvidence;
    fixedProbeDiagnostics: DiagnosticEvidence;
    repairedProbeDiagnostics: DiagnosticEvidence;
  } | null = null;
  try {
    await session.initialize(workspaceDir, {
      completion: true,
      editor: true,
      typecheck: true,
    });

    let actualFileDiagnostics: DiagnosticEvidence | null = null;
    if (actualFile != null) {
      const absolute = path.join(workspaceDir, actualFile);
      const uri = pathToFileURL(absolute).href;
      const source = fs.readFileSync(absolute, "utf8");
      openDocument(session, uri, source, 1);
      openUris.push(uri);
      const diagnostics = await waitForDiagnostics(session, uri, 1, timeoutMs());
      actualFileDiagnostics = diagnosticEvidence(diagnostics.diagnostics);
    }

    const probeUri = pathToFileURL(path.join(workspaceDir, ".vize-fixture-lsp-probe.vue")).href;
    openDocument(session, probeUri, fixedProbeSource, 1);
    openUris.push(probeUri);
    const fixed = await waitForDiagnostics(session, probeUri, 1, timeoutMs());

    changeDocument(session, probeUri, brokenProbeSource, 2);
    const broken = await waitForDiagnostics(
      session,
      probeUri,
      2,
      timeoutMs(),
      (diagnostics) => injectedMismatches(diagnostics).length === 1,
    );
    const mismatches = injectedMismatches(broken.diagnostics);
    assert.equal(mismatches.length, 1, JSON.stringify(broken.diagnostics));
    assert.equal(String(mismatches[0]?.code).replace(/^TS/, ""), "2322");
    assert.equal(mismatches[0]?.source, "vize/types");
    assert.equal(mismatches[0]?.severity, 1);
    const probeOffset = brokenProbeSource.indexOf(probeName);
    const probeStart = offsetToPosition(brokenProbeSource, probeOffset);
    assert.deepEqual(mismatches[0]?.range?.start, probeStart);
    assert.deepEqual(mismatches[0]?.range?.end, {
      line: probeStart.line,
      character: probeStart.character + probeName.length,
    });

    changeDocument(session, probeUri, fixedProbeSource, 3);
    const repaired = await waitForDiagnostics(
      session,
      probeUri,
      3,
      timeoutMs(),
      (diagnostics) => injectedMismatches(diagnostics).length === 0,
    );
    assert.deepEqual(
      normalizeDiagnostics(repaired.diagnostics),
      normalizeDiagnostics(fixed.diagnostics),
      "repair must restore the fixed diagnostic baseline exactly",
    );

    result = {
      actualFileDiagnostics,
      brokenProbeDiagnostics: diagnosticEvidence(broken.diagnostics),
      fixedProbeDiagnostics: diagnosticEvidence(fixed.diagnostics),
      repairedProbeDiagnostics: diagnosticEvidence(repaired.diagnostics),
    };
  } catch (error) {
    primaryError = error;
  } finally {
    for (const uri of openUris.reverse()) {
      try {
        session.notify("textDocument/didClose", { textDocument: { uri } });
      } catch {
        // Shutdown below retains the primary protocol failure with server stderr.
      }
    }
    try {
      await session.shutdown();
    } catch (error) {
      shutdownError = error;
    }
  }
  if (primaryError != null) throw primaryError;
  if (shutdownError != null) throw shutdownError;
  assert.ok(result);
  return result;
}

function openDocument(session: LspAuditSession, uri: string, text: string, version: number): void {
  session.notify("textDocument/didOpen", {
    textDocument: { languageId: "vue", text, uri, version },
  });
}

function changeDocument(
  session: LspAuditSession,
  uri: string,
  text: string,
  version: number,
): void {
  session.notify("textDocument/didChange", {
    contentChanges: [{ text }],
    textDocument: { uri, version },
  });
}

async function waitForDiagnostics(
  session: LspAuditSession,
  uri: string,
  version: number,
  timeoutMs: number,
  predicate?: (diagnostics: LspDiagnostic[]) => boolean,
): Promise<PublishDiagnosticsParams> {
  return (await session.waitForNotification(
    "textDocument/publishDiagnostics",
    (params) => {
      if (!isDiagnosticsForUri(params, uri)) return false;
      const diagnostics = params as PublishDiagnosticsParams;
      return (
        diagnostics.version === version && (predicate == null || predicate(diagnostics.diagnostics))
      );
    },
    timeoutMs,
  )) as PublishDiagnosticsParams;
}

function injectedMismatches(diagnostics: LspDiagnostic[]): LspDiagnostic[] {
  return diagnostics.filter(
    (diagnostic) =>
      String(diagnostic.code).replace(/^TS/, "") === "2322" &&
      /string.*not assignable.*number/i.test(diagnostic.message ?? ""),
  );
}

export function normalizeDiagnostics(diagnostics: LspDiagnostic[]): string[] {
  return diagnostics
    .map((diagnostic) =>
      JSON.stringify({
        code: diagnostic.code,
        message: diagnostic.message,
        range: diagnostic.range,
        severity: diagnostic.severity,
        source: diagnostic.source,
      }),
    )
    .sort();
}

function diagnosticEvidence(diagnostics: LspDiagnostic[]): DiagnosticEvidence {
  const normalized = normalizeDiagnostics(diagnostics);
  return {
    count: normalized.length,
    sha256: createHash("sha256")
      .update(`${normalized.join("\n")}\n`)
      .digest("hex"),
  };
}

function assertBeforeDeadline(deadline: number, timeoutMs: number, now: () => number): void {
  assert.ok(now() < deadline, `LSP fixture shard exceeded ${timeoutMs}ms`);
}
