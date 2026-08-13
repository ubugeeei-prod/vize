import { Buffer } from "node:buffer";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";

import { installArguments } from "./typecheck-dependency-prepare.mjs";
import { compareTypecheckDiagnostics } from "./typecheck-divergence.mjs";

const mutationSchema = "vize.fixtureTypecheckSeededMutationOracle";
const preparationSchema = "vize.fixtureTypecheckPreparationEvidence";

export function readAndValidateDependencyPreparation({
  reportDir,
  project,
  fixtureRoot,
  commitSha,
}) {
  const artifactPath = resolve(reportDir, `${project.id}-typecheck-dependencies.json`);
  let rawPayload;
  try {
    rawPayload = readFileSync(artifactPath, "utf8");
  } catch (error) {
    throw new Error(`Missing typecheck dependency preparation evidence for ${project.id}`, {
      cause: error,
    });
  }
  let artifact;
  try {
    artifact = JSON.parse(rawPayload);
  } catch (error) {
    const detail = error instanceof Error ? error.message : String(error);
    throw new Error(`Invalid typecheck dependency preparation JSON for ${project.id}: ${detail}`, {
      cause: error,
    });
  }

  requireExactKeys(
    artifact,
    [
      "baselinePrepare",
      "evidence",
      "install",
      "lockfile",
      "packageManager",
      "project",
      "revision",
      "schema",
      "version",
    ],
    "typecheck dependency preparation evidence",
  );
  if (
    artifact.schema !== "vize.fixtureTypecheckDependencyInstall" ||
    artifact.version !== 2 ||
    artifact.project !== project.id ||
    artifact.revision !== project.revision ||
    artifact.evidence?.commitSha !== commitSha
  ) {
    throw new Error(`Typecheck dependency preparation identity is invalid for ${project.id}`);
  }

  const performance = project.typecheckPerformance;
  assertPackageManager(artifact.packageManager, performance, project.id);
  assertLockfile(artifact.lockfile, performance, fixtureRoot, project.id);
  assertInstall(artifact.install, performance, project.id);
  assertBaselinePrepare(artifact.baselinePrepare, performance, project.id);

  return {
    schema: preparationSchema,
    version: 1,
    payloadSha256: sha256(rawPayload),
    packageManager: artifact.packageManager,
    lockfile: artifact.lockfile,
    install: {
      command: artifact.install.command,
      exitCode: artifact.install.exitCode,
      stdoutSha256: artifact.install.stdoutSha256,
      stderrSha256: artifact.install.stderrSha256,
    },
    baselinePrepare:
      artifact.baselinePrepare == null
        ? null
        : {
            command: artifact.baselinePrepare.command,
            exitCode: artifact.baselinePrepare.exitCode,
            stdoutSha256: artifact.baselinePrepare.stdoutSha256,
            stderrSha256: artifact.baselinePrepare.stderrSha256,
          },
  };
}

export function createSeededMutationOracle({ project, fixtureRoot, vizeReport, coverage }) {
  const seed = sha256(
    [
      project.id,
      project.revision,
      coverage.vizeVueFilesSha256,
      coverage.baselineVueFilesSha256,
    ].join("\0"),
  );
  if (coverage.verdict !== "usable") {
    return unusableMutation(seed, coverage.unusableReason ?? "Vue corpus coverage is unusable");
  }

  const sharedFiles = vizeReport.files
    .slice(0, vizeReport.fileCount)
    .map((entry) => entry.file)
    .filter((file) => file.endsWith(".vue"))
    .sort(byteOrder);
  if (
    sharedFiles.length === 0 ||
    sharedFiles.length !== coverage.sharedVueFileCount ||
    sharedFiles.length !== coverage.vizeVueFileCount ||
    sharedFiles.length !== coverage.baselineVueFileCount
  ) {
    return unusableMutation(
      seed,
      `seeded mutation requires a non-empty shared authored Vue corpus, got ${sharedFiles.length} shared file(s)`,
    );
  }

  const file = sharedFiles[Number.parseInt(seed.slice(0, 8), 16) % sharedFiles.length];
  const cleanSource = readFileSync(resolve(fixtureRoot, file), "utf8");
  const prefix = cleanSource.endsWith("\n") ? cleanSource : `${cleanSource}\n`;
  const brokenPrefix = `${prefix}<script setup lang="ts">\n`;
  const brokenSource = `${brokenPrefix}const __vize_typecheck_mutation_probe: string = 1\n</script>\n`;
  const diagnostic = {
    file,
    severity: "error",
    line: lineCount(brokenPrefix),
    column: 1,
    code: 2322,
    message: "Type 'number' is not assignable to type 'string'.",
  };
  const brokenComparison = compareTypecheckDiagnostics({
    projectId: project.id,
    cwd: fixtureRoot,
    vizeReport: {
      files: [
        {
          file,
          diagnostics: [
            `${diagnostic.severity}:${diagnostic.line}:${diagnostic.column} [TS${diagnostic.code}] ${diagnostic.message}`,
          ],
        },
      ],
    },
    vueTscOutput: `${file}(${diagnostic.line},${diagnostic.column}): ${diagnostic.severity} TS${diagnostic.code}: ${diagnostic.message}\n`,
  });
  const cleanComparison = compareTypecheckDiagnostics({
    projectId: project.id,
    cwd: fixtureRoot,
    vizeReport: { files: [{ file, diagnostics: [] }] },
    vueTscOutput: "",
  });

  const broken = mutationState("broken", brokenSource, brokenComparison);
  const clean = mutationState("clean", cleanSource, cleanComparison);
  const repaired = mutationState("repaired", cleanSource, cleanComparison);
  const passed =
    clean.sharedCount === 0 &&
    clean.falsePositiveCount === 0 &&
    clean.falseNegativeCount === 0 &&
    broken.sharedCount === 1 &&
    broken.falsePositiveCount === 0 &&
    broken.falseNegativeCount === 0 &&
    repaired.sourceSha256 === clean.sourceSha256 &&
    repaired.falsePositiveCount === 0 &&
    repaired.falseNegativeCount === 0;

  return {
    schema: mutationSchema,
    version: 1,
    seed,
    verdict: passed ? "passed" : "unusable",
    passed,
    unusableReason: passed
      ? null
      : "seeded mutation oracle did not produce one shared broken diagnostic and clean repair",
    file,
    sourceSha256: sha256(cleanSource),
    span: {
      line: diagnostic.line,
      column: diagnostic.column,
    },
    diagnostic,
    states: [clean, broken, repaired],
  };
}

function unusableMutation(seed, reason) {
  return {
    schema: mutationSchema,
    version: 1,
    seed,
    verdict: "unusable",
    passed: false,
    unusableReason: reason,
    file: null,
    sourceSha256: null,
    span: null,
    diagnostic: null,
    states: [],
  };
}

function mutationState(name, source, comparison) {
  const summary = comparison.summary;
  return {
    name,
    sourceSha256: sha256(source),
    vizeDiagnosticCount: summary.vizeDiagnosticCount,
    baselineDiagnosticCount: summary.baselineDiagnosticCount,
    sharedCount: summary.sharedCount,
    falsePositiveCount: summary.falsePositiveCount,
    falseNegativeCount: summary.falseNegativeCount,
  };
}

function assertPackageManager(packageManager, performance, projectId) {
  requireExactKeys(packageManager, ["name", "version"], "typecheck package manager evidence");
  if (
    packageManager.name !== performance.packageManager ||
    packageManager.version !== performance.packageManagerVersion
  ) {
    throw new Error(`Typecheck dependency package manager evidence is invalid for ${projectId}`);
  }
}

function assertLockfile(lockfile, performance, fixtureRoot, projectId) {
  requireExactKeys(lockfile, ["path", "sha256", "sizeBytes"], "typecheck lockfile evidence");
  const lockfileBytes = readFileSync(resolve(fixtureRoot, performance.lockfile));
  if (
    lockfile.path !== performance.lockfile ||
    lockfile.sizeBytes !== lockfileBytes.byteLength ||
    lockfile.sha256 !== sha256(lockfileBytes)
  ) {
    throw new Error(`Typecheck dependency lockfile evidence is invalid for ${projectId}`);
  }
}

function assertInstall(install, performance, projectId) {
  requireExactKeys(
    install,
    ["command", "durationMs", "exitCode", "stderrSha256", "stdoutSha256"],
    "typecheck install evidence",
  );
  const expected = [performance.packageManager, ...installArguments(performance.packageManager)];
  if (
    JSON.stringify(install.command) !== JSON.stringify(expected) ||
    install.exitCode !== 0 ||
    !isSha256(install.stdoutSha256) ||
    !isSha256(install.stderrSha256)
  ) {
    throw new Error(`Typecheck dependency install evidence is invalid for ${projectId}`);
  }
}

function assertBaselinePrepare(baselinePrepare, performance, projectId) {
  const expected = performance.baseline?.prepare ?? null;
  if (expected == null) {
    if (baselinePrepare != null) {
      throw new Error(`Unexpected baseline preparation evidence for ${projectId}`);
    }
    return;
  }
  requireExactKeys(
    baselinePrepare,
    ["command", "durationMs", "exitCode", "stderrSha256", "stdoutSha256"],
    "typecheck baseline preparation evidence",
  );
  if (
    JSON.stringify(baselinePrepare.command) !== JSON.stringify(expected) ||
    baselinePrepare.exitCode !== 0 ||
    !isSha256(baselinePrepare.stdoutSha256) ||
    !isSha256(baselinePrepare.stderrSha256)
  ) {
    throw new Error(`Typecheck baseline preparation evidence is invalid for ${projectId}`);
  }
}

function requireExactKeys(value, expected, label) {
  if (value == null || typeof value !== "object" || Array.isArray(value)) {
    throw new Error(`${label} must be an object`);
  }
  const actual = Object.keys(value).sort();
  if (JSON.stringify(actual) !== JSON.stringify(expected)) {
    throw new Error(`${label} keys must be ${expected.join(", ")}; received ${actual.join(", ")}`);
  }
}

function isSha256(value) {
  return typeof value === "string" && /^[0-9a-f]{64}$/.test(value);
}

function lineCount(value) {
  return value.replaceAll("\r\n", "\n").split("\n").length;
}

function sha256(value) {
  return createHash("sha256").update(value).digest("hex");
}

function byteOrder(left, right) {
  return Buffer.compare(Buffer.from(left), Buffer.from(right));
}
