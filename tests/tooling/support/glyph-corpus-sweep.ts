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
  snapshotWorkspaceFiles,
  withFormattedWorkspace,
} from "../../../tools/fixtures/glyph-corpus.mjs";
import { comparePugTemplateEquivalence, isPugSfc } from "./pug-template-equivalence.ts";
import type { PugOracleComparison, PugOracleEvidence } from "./pug-template-equivalence.ts";
import { compareSfcEquivalence } from "./sfc-equivalence.ts";

export type CorpusProject = {
  id: string;
  fixtureDir: string;
  hydrated: boolean;
  vueGlobs: string[];
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
  counters: { files: number; skipped: number };
  waivedViolations?: Array<Violation & { waiver: object }>;
  pugEvidence?: PugEvidence[];
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

function compareCorpusFile(
  original: string,
  formatted: string,
  project: CorpusProject,
  file: string,
): {
  differences: string[];
  category: "semantic-diff" | "baseline-unusable";
  pug: PugOracleComparison | null;
} {
  const filename = path.join(project.fixtureDir, file);
  try {
    if (isPugSfc(original, filename)) {
      const pug = comparePugTemplateEquivalence(original, formatted, pugContext(project, file));
      return {
        differences: pug.differences,
        category: pug.baselineUsable ? "semantic-diff" : "baseline-unusable",
        pug,
      };
    }
    const differences = compareSfcEquivalence(original, formatted, path.basename(file));
    return { differences, category: "semantic-diff", pug: null };
  } catch (error) {
    return {
      differences: [`comparison failed: ${error instanceof Error ? error.message : String(error)}`],
      category: "baseline-unusable",
      pug: null,
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
        const original = fs.readFileSync(path.join(project.fixtureDir, file), "utf8");
        const formattedBuffer = firstPass.get(file);
        assert.ok(formattedBuffer, `formatter snapshot omitted ${project.id}/${file}`);
        const formatted = formattedBuffer.toString("utf8");
        const result = compareCorpusFile(original, formatted, project, file);
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
        if (differences.length === 0) {
          sink.counters.files += 1;
          if (evidence) sink.pugEvidence?.push(evidence);
          continue;
        }
        const detail = differences.map((difference) => `  ${difference}`).join("\n");
        // Record the waiver decision on the evidence before publishing it, so a
        // reader can tell an enforced regression from an accepted waiver.
        const waiver = sink.waiverConsumption.consume(project.id, file, null, result.category);
        if (evidence) {
          evidence.waived = waiver != null;
          sink.pugEvidence?.push(evidence);
        }
        if (waiver) {
          sink.waivedViolations?.push({ project: project.id, file, detail, waiver });
          sink.counters.skipped += 1;
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
    vueGlobs: ["src/**/*.vue"],
  };
}
