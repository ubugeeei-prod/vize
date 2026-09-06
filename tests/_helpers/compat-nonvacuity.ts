import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";

import { compareTypecheckDiagnostics } from "../../legacy-tools/fixtures/typecheck-divergence.mjs";
import {
  buildSeededMutation,
  seededMutationDiagnostic,
} from "../../legacy-tools/fixtures/typecheck-divergence-mutation-source.mjs";
import type { CompatProbe, CompatSummary } from "./compat-ratchet.ts";
import type { PinnedFixtureWorkspace } from "./realworld-patch.ts";
import { runVizeCheck, runVueTsc } from "./realworld-typecheck.ts";

export type CompatNonVacuity = {
  file: string | null;
  passed: boolean;
  reason: string | null;
  summary: CompatSummary | null;
};

export function assertCompatNonVacuity(
  fixtureId: string,
  summary: CompatSummary,
  nonVacuity: CompatNonVacuity | null,
): void {
  if (!isDiagnosticFreeSummary(summary)) return;
  assert.equal(
    nonVacuity?.passed,
    true,
    `${fixtureId}: the compat result is diagnostic-free on both tools, so ` +
      `the ratchet must prove the checked sources are live with a seeded TypeScript ` +
      `mutation (${nonVacuity?.reason ?? "probe was not run"})`,
  );
}

export function isDiagnosticFreeSummary(summary: CompatSummary): boolean {
  return summary.vizeDiagnosticCount === 0 && summary.baselineDiagnosticCount === 0;
}

export type TypecheckDivergence = {
  summary: CompatSummary & Record<string, number>;
  shared?: Array<{ code?: unknown; file?: unknown; severity?: unknown }>;
};

export function compatSummaryFromDivergence(
  summary: TypecheckDivergence["summary"],
): CompatSummary {
  return {
    vizeDiagnosticCount: summary.vizeDiagnosticCount,
    baselineDiagnosticCount: summary.baselineDiagnosticCount,
    sharedCount: summary.sharedCount,
    messageMismatchCount: summary.messageMismatchCount,
    documentedDifferenceCount: summary.documentedDifferenceCount,
    falsePositiveCount: summary.falsePositiveCount,
    falseNegativeCount: summary.falseNegativeCount,
    falsePositiveRatio: summary.falsePositiveRatio,
    falseNegativeRatio: summary.falseNegativeRatio,
  };
}

export function runCompatNonVacuityProbe({
  probe,
  fixture,
  corsaPath,
  vueTscPath,
  documentedDifferences,
}: {
  probe: CompatProbe;
  fixture: PinnedFixtureWorkspace;
  corsaPath: string;
  vueTscPath: string;
  documentedDifferences: unknown[];
}): CompatNonVacuity {
  let failed: CompatNonVacuity | null = null;
  for (const file of listVueFiles(fixture.workspaceDir)) {
    const cleanSource = fixture.read(file);
    const mutation = buildSeededMutation(cleanSource);
    if (mutation == null) continue;

    try {
      fixture.write(file, mutation.brokenSource);
      const vize = runVizeCheck(fixture.workspaceDir, corsaPath, []);
      assert.ok(
        vize.status === 0 || vize.status === 1,
        `${probe.fixtureId}: Vize non-vacuity probe must complete: exit ${vize.status}\n${vize.stderr}`,
      );
      const vueTsc = runVueTsc(fixture.workspaceDir, vueTscPath);
      assert.ok(
        vueTsc.status === 0 || vueTsc.status === 1 || vueTsc.status === 2,
        `${probe.fixtureId}: vue-tsc non-vacuity probe must complete: exit ${vueTsc.status}\n${vueTsc.stderr}`,
      );
      const divergence = compareTypecheckDiagnostics({
        projectId: probe.fixtureId,
        cwd: fixture.workspaceDir,
        vizeReport: vize.report,
        vueTscOutput: `${vueTsc.stdout}\n${vueTsc.stderr}`,
        documentedDifferences,
      }) as TypecheckDivergence;
      const summary = compatSummaryFromDivergence(divergence.summary);
      const expectedMatched = divergence.shared?.some(
        (record) =>
          record.file === file &&
          record.severity === seededMutationDiagnostic.severity &&
          record.code === seededMutationDiagnostic.code,
      );
      const passed =
        expectedMatched === true &&
        summary.sharedCount >= 1 &&
        summary.messageMismatchCount === 0 &&
        summary.documentedDifferenceCount === 0 &&
        summary.falseNegativeCount === 0;
      const reason = passed
        ? null
        : `seeded mutation produced shared=${summary.sharedCount}, messageMismatches=${summary.messageMismatchCount}, documented=${summary.documentedDifferenceCount}, falsePositives=${summary.falsePositiveCount}, falseNegatives=${summary.falseNegativeCount}`;
      const observed = { file, passed, reason, summary };
      if (passed) return observed;
      failed = observed;
    } catch (error) {
      failed = {
        file,
        passed: false,
        reason: error instanceof Error ? error.message : String(error),
        summary: null,
      };
    } finally {
      fixture.write(file, cleanSource);
    }
  }

  return (
    failed ?? {
      file: null,
      passed: false,
      reason: "no copied authored Vue file accepted a seeded TypeScript probe",
      summary: null,
    }
  );
}

function listVueFiles(root: string, relativeDir = ""): string[] {
  const dir = path.join(root, relativeDir);
  const files: string[] = [];
  for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
    if (entry.name === "node_modules") continue;
    const relativePath = relativeDir === "" ? entry.name : `${relativeDir}/${entry.name}`;
    if (entry.isDirectory()) files.push(...listVueFiles(root, relativePath));
    else if (entry.isFile() && relativePath.endsWith(".vue")) files.push(relativePath);
  }
  return files.sort();
}
