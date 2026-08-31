/**
 * Version and backend provenance recorded with every benchmark artifact
 * (#3283: "Record the exact Vize and native TypeScript runtime versions, entry point, file count,
 * byte count, diagnostic count, and backend readiness with every artifact").
 *
 * tools/benchmarks/scripts/check-gate.mjs already records this for the gated timing artifact.
 * This module supplies the same facts to tools/benchmarks/scripts/compare-tools.mjs, whose
 * tool-comparison artifact feeds the published performance snapshot, so a
 * reader can reproduce a number without guessing which binaries produced it.
 *
 * Backend readiness is resolved, never assumed: `vize check` measured without
 * a resolvable native TypeScript engine is not a measurement of type checking,
 * which is the exact failure mode the upstream vue-benchmarks report hit.
 */

import { spawnSync } from "node:child_process";
import { existsSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

import { fileSha256 } from "./benchmark-binary.mjs";
import { packageVersion, resolveVuePackageDir, typescriptVersionNear } from "./check-gate-env.mjs";

const benchDir = dirname(fileURLToPath(import.meta.url));
const rootDir = resolve(benchDir, "..", "..", "..");

/** First line of `<binary> --version`, or null when the binary cannot answer. */
export function probeVersion(binaryPath) {
  if (!binaryPath || !existsSync(binaryPath)) return null;
  const probe = spawnSync(binaryPath, ["--version"], { encoding: "utf8" });
  if (probe.status !== 0) return null;
  const line = `${probe.stdout || probe.stderr || ""}`.trim().split("\n")[0];
  return line === "" ? null : line;
}

export function resolveFirstExisting(candidates) {
  for (const candidate of candidates) {
    if (candidate && existsSync(resolve(candidate))) return resolve(candidate);
  }
  return null;
}

export const TSGO_CANDIDATES = [
  process.env.VIZE_CHECK_GATE_TSGO,
  join(
    rootDir,
    "node_modules",
    "@typescript",
    `typescript-${process.platform}-${process.arch}`,
    "lib",
    `tsc${process.platform === "win32" ? ".exe" : ""}`,
  ),
  join(rootDir, "node_modules", ".bin", "tsgo"),
  join(rootDir, "tests", "node_modules", ".bin", "tsgo"),
];

/**
 * Resolve the native TypeScript engine that `vize check` will be
 * pointed at. `ready` is false when it cannot be resolved or cannot answer
 * --version; callers that intend to publish a type-check timing must refuse.
 */
export function resolveBackend(candidates = TSGO_CANDIDATES) {
  const corsaPath = resolveFirstExisting(candidates);
  if (corsaPath == null) {
    return {
      engine: "tsgo-native",
      corsaPath: null,
      corsaVersion: null,
      ready: false,
      reason: `no TypeScript 7/Corsa runtime at: ${candidates.filter(Boolean).join(", ")}`,
    };
  }
  const corsaVersion = probeVersion(corsaPath);
  return {
    engine: "tsgo-native",
    corsaPath,
    corsaVersion,
    ready: corsaVersion != null,
    reason:
      corsaVersion == null ? `TypeScript 7/Corsa runtime at ${corsaPath} failed --version` : null,
  };
}

/**
 * Every version a reader needs to reproduce a tool-comparison number. Optional
 * incumbent tools record `null` rather than being omitted, so the artifact
 * shape is stable across runs that skip a task.
 */
export function collectBinaryHashes({
  vizeBin,
  corsaPath,
  vueTscBin,
  verterTscBin,
  golarBin,
  eslintBin,
  prettierBin,
}) {
  return {
    vize: fileSha256(vizeBin),
    tsgo: fileSha256(corsaPath),
    vueTsc: fileSha256(vueTscBin),
    verterTsc: fileSha256(verterTscBin),
    golar: fileSha256(golarBin),
    eslint: fileSha256(eslintBin),
    prettier: fileSha256(prettierBin),
  };
}

export function collectVersions({
  vizeBin,
  corsaVersion,
  vueTscBin,
  verterTscBin,
  golarBin,
  eslintBin,
  prettierBin,
}) {
  const vuePackageDir = resolveVuePackageDir();
  return {
    vize: probeVersion(vizeBin),
    tsgo: corsaVersion,
    vueTsc: probeVersion(vueTscBin),
    verterTsc: probeVersion(verterTscBin),
    golar: probeVersion(golarBin),
    typescript: vueTscBin ? typescriptVersionNear(vueTscBin) : null,
    vue: vuePackageDir ? packageVersion(vuePackageDir) : null,
    eslint: probeVersion(eslintBin),
    prettier: probeVersion(prettierBin),
    node: process.version,
  };
}

export const UNRECORDED_PROVENANCE_LINE =
  "Versions and backend readiness: not recorded — this artifact predates tools/benchmarks/scripts/benchmark-provenance.mjs and cannot be reproduced from itself.";

export function renderProvenanceLines(data) {
  const versions = data.versions;
  const backend = data.backend;
  if (versions == null || backend == null) {
    return [UNRECORDED_PROVENANCE_LINE];
  }
  const show = (value) => (value == null ? "n/a" : `\`${value}\``);
  const lines = [
    `Versions: vize ${show(versions.vize)} · tsgo ${show(versions.tsgo)} · vue-tsc ${show(versions.vueTsc)} (typescript ${show(versions.typescript)}) · verter-tsc ${show(versions.verterTsc)} · Golar ${show(versions.golar)} · vue ${show(versions.vue)} · eslint ${show(versions.eslint)} · prettier ${show(versions.prettier)} · node ${show(versions.node)}`,
  ];
  lines.push(
    `Binaries (sha256): ${Object.entries(data.binaries ?? {})
      .map(([label, sha256]) => `${label} ${show(sha256)}`)
      .join(" ")}`,
  );
  lines.push(
    backend.ready
      ? `Backend: native TypeScript engine ready at ${show(backend.corsaPath)}. Planted-diagnostic gating for the type-check rows lives in tools/benchmarks/scripts/check-gate.mjs (.github/workflows/check-bench.yml).`
      : `Backend: native TypeScript engine NOT ready (${backend.reason}); no type-check timing may be published from this artifact.`,
  );
  return lines;
}
