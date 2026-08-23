import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { basename, dirname, isAbsolute, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

import { typecheckCorpusGlobs } from "./tool-matrix-command.mjs";
import { collectTypecheckerAuthoredPaths, collectVueInputPaths } from "./tool-matrix-inputs.mjs";
import {
  summarizeTypecheckerCoverage,
  validateTypecheckerOutput,
} from "./tool-matrix-typechecker.mjs";

/**
 * The matrix artifacts a divergence run is allowed to trust, split out of
 * `typecheck-divergence-report.mjs` so the report script stays inside the
 * per-file line budget.
 *
 * Every check here answers the same question the ledger got wrong before it had
 * them: is the Vize side of this comparison the run the shard actually made, at
 * the commit it claims, over the corpus it claims? A report that reads a stale
 * or hand-edited artifact measures nothing, so identity, path shape, exact key
 * set and re-derived coverage are all asserted rather than assumed.
 */

const repoRoot = resolve(dirname(fileURLToPath(import.meta.url)), "..", "..");

export function readAndValidateSummary(reportDir, project) {
  const summary = readJson(join(reportDir, "summary.json"));
  if (summary.schema !== "vize.fixtureToolMatrixReport" || summary.version !== 3) {
    throw new Error("Fixture matrix summary schema is unsupported");
  }
  if (!/^[0-9a-f]{40}$/.test(summary.evidence?.commitSha ?? "")) {
    throw new Error("Fixture matrix summary is missing exact commit evidence");
  }
  if (process.env.GITHUB_SHA != null && summary.evidence.commitSha !== process.env.GITHUB_SHA) {
    throw new Error("Fixture matrix summary commit does not match GITHUB_SHA");
  }
  const projectSummary = summary.projects?.filter((entry) => entry.id === project.id) ?? [];
  if (projectSummary.length !== 1 || projectSummary[0].revision !== project.revision) {
    throw new Error(`Fixture matrix summary does not contain pinned project ${project.id}`);
  }
  return summary;
}

export function readAndValidateVizeRun(reportDir, project, summary) {
  const projectSummary = summary.projects.find((entry) => entry.id === project.id);
  const runs = projectSummary.runs.filter((run) => run.tool === "typechecker");
  if (runs.length !== 1 || !["ok", "findings"].includes(runs[0].status)) {
    throw new Error(`Fixture matrix summary has no successful typechecker run for ${project.id}`);
  }
  const expectedName = `${project.id}-typechecker.json`;
  const reportedPath = runs[0].outputPath;
  const artifactPath =
    typeof reportedPath === "string" && !isAbsolute(reportedPath)
      ? resolve(repoRoot, reportedPath)
      : null;
  if (
    artifactPath !== resolve(reportDir, expectedName) ||
    basename(reportedPath ?? "") !== expectedName
  ) {
    throw new Error(`Fixture matrix typechecker output path is invalid for ${project.id}`);
  }
  const rawPayload = readFileSync(artifactPath, "utf8");
  const payload = JSON.parse(rawPayload);
  const expectedKeys = [
    "exitCode",
    "parsed",
    "project",
    "schema",
    "stderr",
    "stdout",
    "tool",
    "typecheckerCoverage",
    "version",
  ];
  if (JSON.stringify(Object.keys(payload).sort()) !== JSON.stringify(expectedKeys)) {
    throw new Error(`Fixture matrix typechecker artifact keys are invalid for ${project.id}`);
  }
  if (
    payload.schema !== "vize.fixtureToolRun" ||
    payload.version !== 1 ||
    payload.project !== project.id ||
    payload.tool !== "typechecker" ||
    payload.exitCode !== runs[0].exitCode
  ) {
    throw new Error(`Fixture matrix typechecker artifact identity is invalid for ${project.id}`);
  }
  let stdout;
  try {
    stdout = JSON.parse(payload.stdout);
  } catch {
    throw new Error(`Fixture matrix typechecker stdout is not JSON for ${project.id}`);
  }
  if (canonicalJson(stdout) !== canonicalJson(payload.parsed)) {
    throw new Error(
      `Fixture matrix typechecker stdout does not match parsed output for ${project.id}`,
    );
  }
  const fixtureRoot = resolve(repoRoot, project.fixturePath);
  const expectedCoverage = validateTypecheckerOutput(
    project,
    payload.parsed,
    payload.exitCode,
    collectVueInputPaths(fixtureRoot, typecheckCorpusGlobs(project)),
    collectTypecheckerAuthoredPaths(fixtureRoot),
  );
  if (canonicalJson(expectedCoverage) !== canonicalJson(payload.typecheckerCoverage)) {
    throw new Error(`Fixture matrix typechecker coverage is inconsistent for ${project.id}`);
  }
  const expectedStatus = payload.exitCode === 0 ? "ok" : "findings";
  if (runs[0].status !== expectedStatus) {
    throw new Error(`Fixture matrix typechecker status is inconsistent for ${project.id}`);
  }
  if (runs[0].fileCount !== payload.parsed.fileCount) {
    throw new Error(`Fixture matrix typechecker file count is inconsistent for ${project.id}`);
  }
  if (
    canonicalJson(runs[0].coverage) !==
    canonicalJson(summarizeTypecheckerCoverage(payload.typecheckerCoverage))
  ) {
    throw new Error(
      `Fixture matrix typechecker summary coverage is inconsistent for ${project.id}`,
    );
  }
  return {
    payload,
    source: {
      payloadSha256: createHash("sha256").update(rawPayload).digest("hex"),
      fileCount: payload.parsed.fileCount,
    },
  };
}

function readJson(path) {
  return JSON.parse(readFileSync(path, "utf8"));
}

function canonicalJson(value) {
  if (Array.isArray(value)) return `[${value.map(canonicalJson).join(",")}]`;
  if (value != null && typeof value === "object") {
    return `{${Object.keys(value)
      .sort()
      .map((key) => `${JSON.stringify(key)}:${canonicalJson(value[key])}`)
      .join(",")}}`;
  }
  return JSON.stringify(value);
}
