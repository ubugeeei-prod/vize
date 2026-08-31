import { spawnSync } from "node:child_process";
import { cpus, totalmem } from "node:os";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const repoRoot = resolve(dirname(fileURLToPath(import.meta.url)), "..", "..");

/**
 * Provenance for a fixture-corpus run artifact: which commit produced it, on
 * which runtime, on what machine.
 *
 * A corpus report is release evidence, and a number measured on an unknown
 * commit or an unknown machine is not evidence — so every field is required and
 * a missing or malformed one is a hard error rather than a `null` that survives
 * into the artifact. Shared by every `tools/fixtures/*-report.mjs` so their
 * artifacts stay comparable field for field.
 */
export function collectRunEvidence() {
  const machineCpus = cpus();
  const logicalCpuCount = machineCpus.length;
  const totalMemoryBytes = totalmem();
  if (!Number.isSafeInteger(logicalCpuCount) || logicalCpuCount < 1) {
    throw new Error("Machine evidence requires at least one logical CPU");
  }
  if (!Number.isSafeInteger(totalMemoryBytes) || totalMemoryBytes < 1) {
    throw new Error("Machine evidence requires positive total memory");
  }
  return {
    commitSha: resolveCommitSha(),
    runtime: { name: "node", version: process.versions.node },
    machine: {
      platform: process.platform,
      arch: process.arch,
      cpuModel: machineCpus[0]?.model.trim() || "unknown",
      logicalCpuCount,
      totalMemoryBytes,
    },
  };
}

export function resolveCommitSha() {
  const environmentSha = process.env.GITHUB_SHA;
  if (environmentSha != null) return requireCommitSha(environmentSha, "GITHUB_SHA");
  const result = spawnSync("git", ["rev-parse", "HEAD"], {
    cwd: repoRoot,
    encoding: "utf8",
    timeout: 30_000,
  });
  if (result.error != null || result.status !== 0) {
    throw new Error("Unable to resolve fixture matrix commit SHA");
  }
  return requireCommitSha(result.stdout.trim(), "git rev-parse HEAD");
}

function requireCommitSha(value, source) {
  if (!/^[0-9a-f]{40}$/.test(value)) {
    throw new Error(`${source} must be a full lowercase commit SHA`);
  }
  return value;
}
