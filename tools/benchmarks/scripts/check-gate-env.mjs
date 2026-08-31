/**
 * Environment resolution and corpus preparation for the `vize check`
 * benchmark gate. Binary resolution fails closed: a missing vize or native
 * TypeScript runtime is an explicit error, and an explicit pin never falls
 * back to another binary.
 */

import { createRequire } from "node:module";
import { spawnSync } from "node:child_process";
import {
  copyFileSync,
  existsSync,
  mkdirSync,
  readdirSync,
  readFileSync,
  realpathSync,
  rmSync,
  statSync,
  symlinkSync,
  writeFileSync,
} from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const benchDir = dirname(fileURLToPath(import.meta.url));
const rootDir = resolve(benchDir, "..", "..", "..");

/**
 * Resolve a binary that MUST exist and answer --version; fail closed. An
 * explicit pin is exclusive: a benchmark asked to measure one binary must
 * never silently fall back to another.
 */
export function requireBinary(label, explicitPath, fallbacks) {
  const candidates = (explicitPath ? [explicitPath] : fallbacks).map((c) => resolve(c));
  const found = candidates.find((candidate) => existsSync(candidate));
  if (!found) {
    throw new Error(`check-gate: ${label} not found (looked at: ${candidates.join(", ")})`);
  }
  const probe = spawnSync(found, ["--version"], { encoding: "utf8" });
  if (probe.status !== 0) {
    throw new Error(`check-gate: ${label} at ${found} failed --version`);
  }
  return { path: found, version: (probe.stdout || probe.stderr).trim().split("\n")[0] };
}

export function resolveOptionalVueTsc() {
  const candidates = [
    process.env.VIZE_CHECK_GATE_VUE_TSC,
    join(benchDir, "node_modules", ".bin", "vue-tsc"),
    join(rootDir, "node_modules", ".bin", "vue-tsc"),
    join(rootDir, "tests", "node_modules", ".bin", "vue-tsc"),
  ].filter(Boolean);
  const found = candidates.find((candidate) => existsSync(candidate));
  if (!found) return null;
  const probe = spawnSync(found, ["--version"], { encoding: "utf8" });
  if (probe.status !== 0) return null;
  return { path: found, version: (probe.stdout || "").trim().split("\n")[0] };
}

export function resolveVuePackageDir() {
  const candidates = [
    join(benchDir, "node_modules", "vue"),
    join(rootDir, "node_modules", "vue"),
    join(rootDir, "tests", "node_modules", "vue"),
  ];
  return candidates.find((candidate) => existsSync(candidate)) ?? null;
}

export function packageVersion(packageDir) {
  try {
    return JSON.parse(readFileSync(join(packageDir, "package.json"), "utf8")).version ?? null;
  } catch {
    return null;
  }
}

export function typescriptVersionNear(vueTscPath) {
  try {
    const require = createRequire(join(dirname(realpathSync(vueTscPath)), "package.json"));
    return JSON.parse(readFileSync(require.resolve("typescript/package.json"), "utf8")).version;
  } catch {
    return null;
  }
}

export const CORPUS_TSCONFIG = {
  compilerOptions: {
    esModuleInterop: true,
    isolatedModules: true,
    lib: ["ESNext", "DOM"],
    module: "ESNext",
    moduleResolution: "bundler",
    noEmit: true,
    skipLibCheck: true,
    strict: true,
    target: "ESNext",
    types: [],
  },
  vueCompilerOptions: { strictTemplates: true },
  include: ["*.vue"],
};

/** Copy the measured corpus subset into a self-contained project dir. */
export function prepareCorpus(inputDir, fileCount, workRoot, vuePackageDir) {
  if (!existsSync(inputDir)) {
    throw new Error(
      `check-gate: input corpus not found: ${inputDir} (run tools/benchmarks/scripts/generate.mjs first)`,
    );
  }
  const files = readdirSync(inputDir)
    .filter((file) => file.endsWith(".vue"))
    .sort()
    .slice(0, fileCount);
  if (files.length === 0) throw new Error(`check-gate: no .vue files found in ${inputDir}`);
  const dir = join(workRoot, `corpus-${files.length}`);
  rmSync(dir, { recursive: true, force: true });
  mkdirSync(dir, { recursive: true });
  let totalBytes = 0;
  for (const file of files) {
    copyFileSync(join(inputDir, file), join(dir, file));
    totalBytes += statSync(join(dir, file)).size;
  }
  writeFileSync(join(dir, "tsconfig.json"), `${JSON.stringify(CORPUS_TSCONFIG, null, 2)}\n`);
  writeFileSync(
    join(dir, "package.json"),
    `${JSON.stringify({ name: "vize-check-gate-corpus", private: true, type: "module" }, null, 2)}\n`,
  );
  const nodeModules = join(dir, "node_modules");
  mkdirSync(nodeModules, { recursive: true });
  symlinkSync(vuePackageDir, join(nodeModules, "vue"), "dir");
  const vueNamespace = join(dirname(vuePackageDir), "@vue");
  if (existsSync(vueNamespace)) symlinkSync(vueNamespace, join(nodeModules, "@vue"), "dir");
  return { dir, files, totalBytes, tsconfigPath: join(dir, "tsconfig.json") };
}
