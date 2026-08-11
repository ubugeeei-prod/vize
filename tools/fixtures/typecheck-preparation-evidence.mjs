import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { join, resolve } from "node:path";

import { installArguments } from "./typecheck-dependency-contract.mjs";

const shaPattern = /^[0-9a-f]{64}$/u;

export function readTypecheckPreparationEvidence(reportDir, project, matrixEvidence, fixtureRoot) {
  const artifactPath = join(reportDir, `${project.id}-typecheck-dependencies.json`);
  let raw;
  try {
    raw = readFileSync(artifactPath, "utf8");
  } catch {
    throw new Error(`Typecheck preparation evidence is missing for ${project.id}`);
  }
  let artifact;
  try {
    artifact = JSON.parse(raw);
  } catch {
    throw new Error(`Typecheck preparation evidence is not JSON for ${project.id}`);
  }
  validatePreparationArtifact(artifact, project, matrixEvidence, fixtureRoot);
  return { artifactSha256: sha256(raw), ...artifact };
}

function validatePreparationArtifact(artifact, project, matrixEvidence, fixtureRoot) {
  exactKeys(artifact, [
    "baselineConfig",
    "baselinePrepare",
    "evidence",
    "install",
    "lockfile",
    "packageManager",
    "project",
    "revision",
    "schema",
    "version",
  ]);
  if (
    artifact.schema !== "vize.fixtureTypecheckDependencyInstall" ||
    artifact.version !== 3 ||
    artifact.project !== project.id ||
    artifact.revision !== project.revision
  ) {
    throw new Error(`Typecheck preparation identity is invalid for ${project.id}`);
  }
  exactKeys(artifact.evidence, ["commitSha", "runtime"]);
  exactKeys(artifact.evidence.runtime, ["name", "version"]);
  if (
    artifact.evidence.commitSha !== matrixEvidence.commitSha ||
    canonicalJson(artifact.evidence.runtime) !== canonicalJson(matrixEvidence.runtime)
  ) {
    throw new Error(`Typecheck preparation commit or runtime is stale for ${project.id}`);
  }

  const performance = project.typecheckPerformance;
  exactKeys(artifact.packageManager, ["name", "version"]);
  if (
    artifact.packageManager.name !== performance.packageManager ||
    artifact.packageManager.version !== performance.packageManagerVersion
  ) {
    throw new Error(`Typecheck preparation package manager is invalid for ${project.id}`);
  }

  const lockfile = readFileSync(resolve(fixtureRoot, performance.lockfile));
  exactKeys(artifact.lockfile, ["path", "sha256", "sizeBytes"]);
  if (
    artifact.lockfile.path !== performance.lockfile ||
    artifact.lockfile.sizeBytes !== lockfile.byteLength ||
    artifact.lockfile.sha256 !== sha256(lockfile)
  ) {
    throw new Error(`Typecheck preparation lockfile is stale for ${project.id}`);
  }

  exactKeys(artifact.install, [
    "command",
    "durationMs",
    "exitCode",
    "stderrSha256",
    "stdoutSha256",
  ]);
  const expectedInstall = [
    performance.packageManager,
    ...installArguments(performance.packageManager),
  ];
  validateSuccessfulCommand(artifact.install, expectedInstall, "install", project.id);

  const expectedPrepare = performance.baseline?.prepare ?? null;
  if (expectedPrepare == null) {
    if (artifact.baselinePrepare !== null) {
      throw new Error(`Typecheck preparation command is unexpected for ${project.id}`);
    }
  } else {
    exactKeys(artifact.baselinePrepare, [
      "command",
      "durationMs",
      "exitCode",
      "stderrSha256",
      "stdoutSha256",
    ]);
    validateSuccessfulCommand(artifact.baselinePrepare, expectedPrepare, "baseline", project.id);
  }

  const baselinePath = performance.baseline?.tsconfig ?? project.tsconfig;
  const baselineConfig = readFileSync(resolve(fixtureRoot, baselinePath));
  exactKeys(artifact.baselineConfig, ["path", "sha256", "sizeBytes"]);
  if (
    artifact.baselineConfig.path !== baselinePath ||
    artifact.baselineConfig.sizeBytes !== baselineConfig.byteLength ||
    artifact.baselineConfig.sha256 !== sha256(baselineConfig)
  ) {
    throw new Error(`Typecheck preparation baseline config is stale for ${project.id}`);
  }
}

function validateSuccessfulCommand(value, command, phase, projectId) {
  if (
    canonicalJson(value.command) !== canonicalJson(command) ||
    value.exitCode !== 0 ||
    !Number.isSafeInteger(value.durationMs) ||
    value.durationMs < 0 ||
    !shaPattern.test(value.stdoutSha256 ?? "") ||
    !shaPattern.test(value.stderrSha256 ?? "")
  ) {
    throw new Error(`Typecheck preparation ${phase} evidence is invalid for ${projectId}`);
  }
}

function exactKeys(value, expected) {
  if (
    value == null ||
    typeof value !== "object" ||
    Array.isArray(value) ||
    canonicalJson(Object.keys(value).sort()) !== canonicalJson(expected.slice().sort())
  ) {
    throw new Error("Typecheck preparation evidence shape is invalid");
  }
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

function sha256(value) {
  return createHash("sha256").update(value).digest("hex");
}
