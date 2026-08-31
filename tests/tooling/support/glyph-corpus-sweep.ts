// Corpus sweep machinery for the glyph parse-preservation property. Formats
// every hydrated fixture file, compares each pair against the reference oracles
// (structural SFC equivalence for native HTML, the pinned Pug semantic oracle
// for `<template lang="pug">`), and records violations plus the per-file Pug
// evidence the Real Project Matrix publishes as an artifact.
import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";

import {
  collectProjectVueFiles,
  sha256,
  snapshotWorkspaceFiles,
  withFormattedWorkspace,
} from "../../../legacy-tools/fixtures/glyph-corpus.mjs";
import { comparePugTemplateEquivalence, isPugSfc } from "./pug-template-equivalence.ts";
import type { PugOracleComparison, PugOracleEvidence } from "./pug-template-equivalence.ts";
import { resolveSfcDialectPartition } from "./sfc-baseline-routes.ts";
import type { ResolvedSfcDialect, SfcDialectRoute } from "./sfc-baseline-routes.ts";
import { compareSfcWithDialectBaseline, comparisonHarnessFailure } from "./sfc-baselines.ts";
import type { SfcBaselineComparison } from "./sfc-baselines.ts";
import { compareSfcEquivalence } from "./sfc-equivalence.ts";

export type CorpusProject = {
  id: string;
  fixtureDir: string;
  hydrated: boolean;
  revision: string;
  vueGlobs: string[];
  sfcDialectRoutes?: SfcDialectRoute[];
};

export type Violation = { project: string; file: string; detail: string };

export type PugEvidence = {
  project: string;
  path: string;
  verdict: "clean" | "violation" | "baseline-unusable";
  /** True once a checked-in waiver absorbed this file's differences. */
  waived: boolean;
  differences: string[];
  oracle: PugOracleEvidence;
  fixedPoint: {
    sourceBytesEqual: boolean;
    differences: string[];
    oracle: PugOracleEvidence;
  };
};

export type SfcDialectEvidence = {
  project: string;
  revision: string;
  path: string;
  routeId: string;
  dialect: string;
  baselineId: string;
  originalSha256: string;
  formattedSha256: string;
  beforeSemanticSha256: string | null;
  afterSemanticSha256: string | null;
  verdict: string;
  reasonCode: string | null;
  differences: string[];
  failure: object | null;
  waiver: object | null;
  baseline: object;
};

type WaiverConsumption = {
  consume: (
    project: string,
    file: string,
    rule: null,
    category: "semantic-diff" | "baseline-unusable",
  ) => object | null;
};

/** Collecting sink so a sweep can report into caller-owned accumulators. */
export type SweepSink = {
  waiverConsumption: WaiverConsumption;
  violations: Violation[];
  baselineUnusable?: Violation[];
  counters: { files: number; skipped: number };
  waivedViolations?: Array<Violation & { waiver: object }>;
  pugEvidence?: PugEvidence[];
  sfcDialectEvidence?: SfcDialectEvidence[];
};

export function violationCategory(differences: string[]): "semantic-diff" | "baseline-unusable" {
  if (differences.some((difference) => difference.startsWith("comparison failed:"))) {
    return "baseline-unusable";
  }
  return "semantic-diff";
}

export function compareFile(original: string, formatted: string, filename: string): string[] {
  try {
    return compareSfcEquivalence(original, formatted, filename);
  } catch (error) {
    return [`comparison failed: ${error instanceof Error ? error.message : String(error)}`];
  }
}

export function pugContext(project: CorpusProject, file: string) {
  return {
    filename: path.join(project.fixtureDir, file),
    displayFilename: `${project.id}/${file}`,
    basedir: project.fixtureDir,
  };
}

export function compareCorpusFile(
  original: string,
  formatted: string,
  project: CorpusProject,
  file: string,
  route: ResolvedSfcDialect | null,
  compareDialect = compareSfcWithDialectBaseline,
): {
  differences: string[];
  category: "semantic-diff" | "baseline-unusable";
  pug: PugOracleComparison | null;
  dialect: SfcBaselineComparison | null;
} {
  const filename = path.join(project.fixtureDir, file);
  try {
    if (isPugSfc(original, filename)) {
      const pug = comparePugTemplateEquivalence(original, formatted, pugContext(project, file));
      return {
        differences: pug.differences,
        category: pug.baselineUsable ? "semantic-diff" : "baseline-unusable",
        pug,
        dialect: null,
      };
    }
    if (route != null) {
      const dialect = compareDialect(original, formatted, filename, route.dialect);
      return {
        differences: dialect.differences,
        category: dialect.verdict === "baseline-unusable" ? "baseline-unusable" : "semantic-diff",
        pug: null,
        dialect,
      };
    }
    const differences = compareSfcEquivalence(original, formatted, path.basename(file));
    return { differences, category: "semantic-diff", pug: null, dialect: null };
  } catch (error) {
    const dialect = route == null ? null : comparisonHarnessFailure(route.dialect, error);
    return {
      differences: dialect?.differences ?? [
        `comparison failed: ${error instanceof Error ? error.message : String(error)}`,
      ],
      category: "baseline-unusable",
      pug: null,
      dialect,
    };
  }
}

export function sweepProject(
  project: CorpusProject,
  launch: { command: string; prefix: string[] },
  sink: SweepSink,
): void {
  const files = collectProjectVueFiles(project) as string[];
  if (files.length === 0) return;
  // Only projects with an explicit machine-readable partition enter the
  // dialect artifact. The ordinary Vue 3 and Pug corpus paths remain owned by
  // their existing oracles, avoiding a second 34k-file parser pass.
  const partition =
    project.sfcDialectRoutes == null ? null : resolveSfcDialectPartition(project, files);
  withFormattedWorkspace(
    project,
    files,
    launch,
    (workspace: { workspaceDir: string; reformat: () => void }) => {
      const firstPass = snapshotWorkspaceFiles(workspace.workspaceDir, files) as Map<
        string,
        Buffer
      >;
      workspace.reformat();
      for (const file of files) {
        const originalBuffer = fs.readFileSync(path.join(project.fixtureDir, file));
        const original = originalBuffer.toString("utf8");
        const formattedBuffer = firstPass.get(file);
        assert.ok(formattedBuffer, `formatter snapshot omitted ${project.id}/${file}`);
        const formatted = formattedBuffer.toString("utf8");
        const route = partition?.get(file) ?? null;
        const result = compareCorpusFile(original, formatted, project, file, route);
        const differences = [...result.differences];
        let evidence: PugEvidence | null = null;

        if (result.pug != null) {
          const formattedAgainBuffer = fs.readFileSync(path.join(workspace.workspaceDir, file));
          const formattedAgain = formattedAgainBuffer.toString("utf8");
          const fixedPoint = comparePugTemplateEquivalence(
            formatted,
            formattedAgain,
            pugContext(project, file),
          );
          const sourceBytesEqual = formattedBuffer.equals(formattedAgainBuffer);
          if (!sourceBytesEqual) differences.push("formatter fixed point changed source bytes");
          differences.push(
            ...fixedPoint.differences.map((difference) => `fixed point ${difference}`),
          );
          evidence = {
            project: project.id,
            path: file,
            verdict: !result.pug.baselineUsable
              ? "baseline-unusable"
              : differences.length === 0
                ? "clean"
                : "violation",
            waived: false,
            differences,
            oracle: result.pug.evidence,
            fixedPoint: {
              sourceBytesEqual,
              differences: fixedPoint.differences,
              oracle: fixedPoint.evidence,
            },
          };
        }
        let waiver: object | null = null;
        if (differences.length === 0) {
          sink.counters.files += 1;
        } else {
          waiver = sink.waiverConsumption.consume(project.id, file, null, result.category);
        }
        // Record the waiver decision on both artifacts before publishing them.
        if (evidence) {
          evidence.waived = waiver != null;
          sink.pugEvidence?.push(evidence);
        }
        if (result.dialect != null && route != null) {
          sink.sfcDialectEvidence?.push({
            project: project.id,
            revision: project.revision,
            path: file,
            routeId: route.routeId,
            dialect: route.dialect,
            baselineId: result.dialect.baseline.id,
            originalSha256: sha256(originalBuffer),
            formattedSha256: sha256(formattedBuffer),
            beforeSemanticSha256: result.dialect.beforeSemanticSha256,
            afterSemanticSha256: result.dialect.afterSemanticSha256,
            verdict: result.dialect.verdict,
            reasonCode: result.dialect.reasonCode,
            differences: result.dialect.differences,
            failure: result.dialect.failure,
            waiver,
            baseline: result.dialect.baseline,
          });
        }
        if (differences.length === 0) continue;
        const detail = differences.map((difference) => `  ${difference}`).join("\n");
        if (waiver) {
          sink.waivedViolations?.push({ project: project.id, file, detail, waiver });
          sink.counters.skipped += 1;
          continue;
        }
        if (result.category === "baseline-unusable") {
          sink.baselineUnusable?.push({ project: project.id, file, detail });
          continue;
        }
        sink.violations.push({ project: project.id, file, detail });
      }
    },
  );
}

export function makeSyntheticProject(files: Array<[string, string]>): CorpusProject {
  const fixtureDir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-glyph-corpus-"));
  for (const [file, content] of files) {
    fs.mkdirSync(path.dirname(path.join(fixtureDir, file)), { recursive: true });
    fs.writeFileSync(path.join(fixtureDir, file), content);
  }
  return {
    id: "synthetic-parse-preservation",
    fixtureDir,
    hydrated: true,
    revision: "0".repeat(40),
    vueGlobs: ["src/**/*.vue"],
  };
}
