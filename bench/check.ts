/**
 * Type Check Benchmark: Vize (Corsa) vs vue-tsc
 *
 * Usage:
 *   1. Generate test files: node generate.mjs [count]
 *   2. Build CLI: vp run --workspace-root build:cli
 *   3. Run benchmark: node --experimental-strip-types bench/check.ts
 */

import { existsSync, mkdirSync, readdirSync, rmSync, symlinkSync, writeFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, join, relative } from "node:path";
import { execSync } from "node:child_process";
import os from "node:os";

const __dirname = dirname(fileURLToPath(import.meta.url));
const INPUT_DIR = join(__dirname, "__in__");
const E2E_NPMX_DIR = join(__dirname, "..", "tests", "_fixtures", "_git", "npmx.dev");
const CPU_COUNT = os.cpus().length;
const VIZE_BIN = join(__dirname, "..", "target", "release", "vize");
const FILE_LIMIT = parseInt(process.argv[2] || "0", 10) || Infinity;
const VUE_TSC_CANDIDATES = [
  join(__dirname, "node_modules", ".bin", "vue-tsc"),
  join(__dirname, "..", "node_modules", ".bin", "vue-tsc"),
];

// Check input files
if (!existsSync(INPUT_DIR)) {
  console.error(`Error: Input directory not found: ${INPUT_DIR}\nRun 'node generate.mjs' first.`);
  process.exit(1);
}

if (!existsSync(join(INPUT_DIR, "tsconfig.json"))) {
  console.error(
    `Error: tsconfig.json not found in ${INPUT_DIR}\nRun 'node generate.mjs' first to generate it.`,
  );
  process.exit(1);
}

const allVueFiles = readdirSync(INPUT_DIR).filter((f) => f.endsWith(".vue"));
const vueFiles = allVueFiles.filter((f) => f.endsWith(".vue")).slice(0, FILE_LIMIT);
if (vueFiles.length === 0) {
  console.error(`Error: No .vue files found in ${INPUT_DIR}\nRun 'node generate.mjs' first.`);
  process.exit(1);
}
const BENCH_INPUT_DIR = prepareBenchInputDir(vueFiles, allVueFiles.length);
const GLOB_PATTERN = join(BENCH_INPUT_DIR, "*.vue");
const TSCONFIG_PATH = join(BENCH_INPUT_DIR, "tsconfig.json");

function prepareBenchInputDir(selectedVueFiles: string[], totalVueFileCount: number): string {
  if (selectedVueFiles.length >= totalVueFileCount) {
    return INPUT_DIR;
  }

  const subsetDir = join(__dirname, "__agent_only", `check-${selectedVueFiles.length}`);
  rmSync(subsetDir, { recursive: true, force: true });
  mkdirSync(subsetDir, { recursive: true });

  for (const vueFile of selectedVueFiles) {
    symlinkSync(join(INPUT_DIR, vueFile), join(subsetDir, vueFile));
  }

  const tsconfigPath = join(subsetDir, "tsconfig.json");
  writeFileSync(
    tsconfigPath,
    `${JSON.stringify(
      {
        extends: relative(subsetDir, join(INPUT_DIR, "tsconfig.json")),
        include: selectedVueFiles,
      },
      null,
      2,
    )}\n`,
  );

  return subsetDir;
}

// Format helpers
function formatTime(ms: number): string {
  if (ms >= 1000) return `${(ms / 1000).toFixed(2)}s`;
  return `${ms.toFixed(0)}ms`;
}

function formatThroughput(fileCount: number, ms: number): string {
  const filesPerSec = (fileCount / ms) * 1000;
  if (filesPerSec >= 1000) return `${(filesPerSec / 1000).toFixed(1)}k files/s`;
  return `${filesPerSec.toFixed(0)} files/s`;
}

function runCommand(cmd: string, cwd: string = BENCH_INPUT_DIR): number {
  const start = performance.now();
  try {
    execSync(cmd, { stdio: "ignore", cwd });
  } catch {
    // vue-tsc may exit non-zero on type errors; still measure time
  }
  return performance.now() - start;
}

function benchmarkCommand(cmd: string, warmup: number = 0, cwd: string = BENCH_INPUT_DIR): number {
  // Warmup
  for (let i = 0; i < warmup; i++) {
    runCommand(cmd, cwd);
  }
  return runCommand(cmd, cwd);
}

function resolveVueTscBin(): string | null {
  for (const candidate of VUE_TSC_CANDIDATES) {
    if (existsSync(candidate)) {
      return candidate;
    }
  }
  return null;
}

// vue-tsc single-thread
function runVueTscSingleThread(): number {
  const vueTscBin = resolveVueTscBin();
  if (vueTscBin == null) return -1;
  return benchmarkCommand(`${vueTscBin} --noEmit -p ${TSCONFIG_PATH}`);
}

// vue-tsc multi-thread (default TS internal parallelism)
function runVueTscMultiThread(): number {
  const vueTscBin = resolveVueTscBin();
  if (vueTscBin == null) return -1;
  return benchmarkCommand(`${vueTscBin} --noEmit -p ${TSCONFIG_PATH}`);
}

// Vize (Corsa) single-thread
function runVizeCheckSingleThread(): number {
  return benchmarkCommand(
    `RAYON_NUM_THREADS=1 ${VIZE_BIN} check '${GLOB_PATTERN}' --quiet --servers 1 --tsconfig ${TSCONFIG_PATH}`,
  );
}

// Vize (Corsa) multi-thread
function runVizeCheckMultiThread(): number {
  return benchmarkCommand(
    `${VIZE_BIN} check '${GLOB_PATTERN}' --quiet --tsconfig ${TSCONFIG_PATH}`,
  );
}

function countVueFiles(dir: string): number {
  if (!existsSync(dir)) return 0;

  let count = 0;
  for (const entry of readdirSync(dir, { withFileTypes: true })) {
    if (entry.isDirectory()) {
      count += countVueFiles(join(dir, entry.name));
    } else if (entry.name.endsWith(".vue")) {
      count += 1;
    }
  }
  return count;
}

function runVizeE2eNpmxCheck(): number {
  return benchmarkCommand(
    `${VIZE_BIN} check app --quiet --tsconfig tsconfig.json`,
    1,
    E2E_NPMX_DIR,
  );
}

// Main
console.log();
console.log("=".repeat(65));
console.log(" Type Check Benchmark: Corsa vs vue-tsc");
console.log("=".repeat(65));
console.log();
console.log(` Files     : ${vueFiles.length.toLocaleString()} SFC files`);
console.log(` CPU Cores : ${CPU_COUNT}`);
console.log();
console.log("-".repeat(65));

// Single Thread
console.log();
console.log(" Single Thread:");
console.log();

const vueTscSingle = runVueTscSingleThread();
if (vueTscSingle >= 0) {
  console.log(
    `   vue-tsc       : ${formatTime(vueTscSingle).padStart(8)}  (${formatThroughput(vueFiles.length, vueTscSingle)})`,
  );
} else {
  console.log("   vue-tsc       : SKIPPED (not found)");
}

let vizeSingle = 0;
if (existsSync(VIZE_BIN)) {
  vizeSingle = runVizeCheckSingleThread();
  if (vueTscSingle >= 0) {
    const speedup = (vueTscSingle / vizeSingle).toFixed(1);
    console.log(
      `   Vize (Corsa)  : ${formatTime(vizeSingle).padStart(8)}  (${formatThroughput(vueFiles.length, vizeSingle)})  ${speedup}x faster`,
    );
  } else {
    console.log(
      `   Vize (Corsa)  : ${formatTime(vizeSingle).padStart(8)}  (${formatThroughput(vueFiles.length, vizeSingle)})`,
    );
  }
} else {
  console.log("   Vize (Corsa)  : SKIPPED (vize CLI not found)");
}

// Multi Thread
console.log();
console.log(` Multi Thread:`);
console.log();

const vueTscMulti = runVueTscMultiThread();
if (vueTscMulti >= 0) {
  console.log(
    `   vue-tsc       : ${formatTime(vueTscMulti).padStart(8)}  (${formatThroughput(vueFiles.length, vueTscMulti)})`,
  );
} else {
  console.log("   vue-tsc       : SKIPPED (not found)");
}

let vizeMulti = 0;
if (existsSync(VIZE_BIN)) {
  vizeMulti = runVizeCheckMultiThread();
  if (vueTscMulti >= 0) {
    const speedup = (vueTscMulti / vizeMulti).toFixed(1);
    console.log(
      `   Vize (Corsa)  : ${formatTime(vizeMulti).padStart(8)}  (${formatThroughput(vueFiles.length, vizeMulti)})  ${speedup}x faster`,
    );
  } else {
    console.log(
      `   Vize (Corsa)  : ${formatTime(vizeMulti).padStart(8)}  (${formatThroughput(vueFiles.length, vizeMulti)})`,
    );
  }
} else {
  console.log("   Vize (Corsa)  : SKIPPED (vize CLI not found)");
}

// Summary
if (vueTscSingle >= 0 && vizeSingle > 0 && vizeMulti > 0) {
  console.log();
  console.log("-".repeat(65));
  console.log();
  console.log(" Summary:");
  console.log();
  const stSpeedup = (vueTscSingle / vizeSingle).toFixed(1);
  const mtSpeedup = (vueTscMulti / vizeMulti).toFixed(1);
  const crossSpeedup = (vueTscSingle / vizeMulti).toFixed(1);
  console.log(`   vue-tsc ST vs Vize ST : ${stSpeedup}x`);
  console.log(`   vue-tsc MT vs Vize MT : ${mtSpeedup}x`);
  console.log(`   vue-tsc ST vs Vize MT : ${crossSpeedup}x  (user-facing speedup)`);
}

if (existsSync(VIZE_BIN) && existsSync(join(E2E_NPMX_DIR, "app"))) {
  const e2eFileCount = countVueFiles(join(E2E_NPMX_DIR, "app"));
  const e2eTime = runVizeE2eNpmxCheck();
  console.log();
  console.log("-".repeat(65));
  console.log();
  console.log(" Diagnostics-heavy e2e fixture:");
  console.log();
  console.log(
    `   npmx.dev app  : ${formatTime(e2eTime).padStart(8)}  (${formatThroughput(e2eFileCount, e2eTime)}, ${e2eFileCount} SFC files, non-zero diagnostics ignored)`,
  );
} else {
  console.log();
  console.log("-".repeat(65));
  console.log();
  console.log(" Diagnostics-heavy e2e fixture:");
  console.log();
  console.log("   npmx.dev app  : SKIPPED (fixture or vize CLI not found)");
}

console.log();
