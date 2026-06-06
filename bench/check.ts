/**
 * Type Check Benchmark: Vize (Corsa) vs vue-tsc
 *
 * Usage:
 *   1. Generate test files: node generate.mjs [count]
 *   2. Build CLI: vp run --workspace-root build:cli
 *   3. Run benchmark: node bench/check.ts
 */

import { copyFileSync, existsSync, mkdirSync, readdirSync, rmSync, writeFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, join, relative } from "node:path";
import { execSync } from "node:child_process";
import os from "node:os";

const __dirname = dirname(fileURLToPath(import.meta.url));
const INPUT_DIR = join(__dirname, "__in__");
const GIT_FIXTURE_DIR = join(__dirname, "..", "tests", "_fixtures", "_git");
const CPU_COUNT = os.cpus().length;
const FILE_LIMIT = parseInt(process.argv[2] || "0", 10) || Infinity;
const BIN_EXT = process.platform === "win32" ? ".exe" : "";
const VIZE_RELEASE_BIN = join(__dirname, "..", "target", "release", `vize${BIN_EXT}`);
const VIZE_CI_BIN = join(__dirname, "..", "target", "ci", `vize${BIN_EXT}`);
const VIZE_DEBUG_BIN = join(__dirname, "..", "target", "debug", `vize${BIN_EXT}`);
const VIZE_BIN =
  process.env.VIZE_BIN ??
  [VIZE_CI_BIN, VIZE_RELEASE_BIN, VIZE_DEBUG_BIN].find((candidate) => existsSync(candidate)) ??
  VIZE_RELEASE_BIN;
const VUE_TSC_CANDIDATES = [
  join(__dirname, "node_modules", ".bin", "vue-tsc"),
  join(__dirname, "..", "node_modules", ".bin", "vue-tsc"),
];
const REAL_WORLD_TYPECHECK_FIXTURES: RealWorldTypecheckFixture[] = [
  {
    name: "voicevox",
    cwd: join(GIT_FIXTURE_DIR, "voicevox"),
    patterns: ["src/**/*.vue"],
    tsconfig: "tsconfig.json",
    timeoutMs: 300_000,
  },
  {
    name: "elk",
    cwd: join(GIT_FIXTURE_DIR, "elk"),
    patterns: ["app/**/*.vue"],
    tsconfig: "tsconfig.json",
    timeoutMs: 300_000,
  },
  {
    name: "misskey",
    cwd: join(GIT_FIXTURE_DIR, "misskey", "packages", "frontend"),
    patterns: ["src/**/*.vue"],
    tsconfig: "tsconfig.json",
    timeoutMs: 300_000,
  },
  {
    name: "vue-vben-admin",
    cwd: join(GIT_FIXTURE_DIR, "vue-vben-admin"),
    patterns: ["playground/src/**/*.vue", "apps/**/*.vue", "packages/**/*.vue"],
    timeoutMs: 300_000,
  },
  {
    name: "hoppscotch",
    cwd: join(GIT_FIXTURE_DIR, "hoppscotch"),
    patterns: ["packages/**/*.vue"],
    timeoutMs: 300_000,
  },
  {
    name: "element-plus",
    cwd: join(GIT_FIXTURE_DIR, "element-plus"),
    patterns: ["packages/**/*.vue", "docs/**/*.vue", "ssr-testing/**/*.vue"],
    tsconfig: "tsconfig.json",
    timeoutMs: 300_000,
  },
];

interface RealWorldTypecheckFixture {
  name: string;
  cwd: string;
  patterns: string[];
  tsconfig?: string;
  timeoutMs: number;
}

interface RealWorldTypecheckResult {
  name: string;
  status: "ok" | "skipped" | "crashed" | "timed-out";
  ms: number;
  fileCount: number;
  errorCount: number;
  reason?: string;
}

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

  const subsetDir = join(__dirname, "target", "vize-tests", `check-${selectedVueFiles.length}`);
  rmSync(subsetDir, { recursive: true, force: true });
  mkdirSync(subsetDir, { recursive: true });

  for (const vueFile of selectedVueFiles) {
    copyFileSync(join(INPUT_DIR, vueFile), join(subsetDir, vueFile));
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

function shellQuote(value: string): string {
  return `'${value.replaceAll("'", "'\\''")}'`;
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

function runVizeRealWorldTypecheck(fixture: RealWorldTypecheckFixture): RealWorldTypecheckResult {
  if (!existsSync(VIZE_BIN)) {
    return {
      name: fixture.name,
      status: "skipped",
      ms: 0,
      fileCount: 0,
      errorCount: 0,
      reason: "vize CLI not found",
    };
  }

  if (!existsSync(fixture.cwd)) {
    return {
      name: fixture.name,
      status: "skipped",
      ms: 0,
      fileCount: 0,
      errorCount: 0,
      reason: "fixture not found",
    };
  }

  const patterns = fixture.patterns.map(shellQuote).join(" ");
  const tsconfig = fixture.tsconfig ? ` --tsconfig ${shellQuote(fixture.tsconfig)}` : "";
  const cmd = `${shellQuote(VIZE_BIN)} check ${patterns} --format json --quiet${tsconfig}`;
  const start = performance.now();
  let stdout = "";

  try {
    stdout = execSync(cmd, {
      cwd: fixture.cwd,
      encoding: "utf8",
      maxBuffer: 100 * 1024 * 1024,
      timeout: fixture.timeoutMs,
    });
  } catch (error: unknown) {
    const commandError = error as {
      status?: number;
      stdout?: { toString(): string };
      stderr?: { toString(): string };
      signal?: string;
    };
    const ms = performance.now() - start;

    if (commandError.status === 1 && commandError.stdout) {
      stdout = commandError.stdout.toString();
    } else {
      return {
        name: fixture.name,
        status: commandError.signal === "SIGTERM" ? "timed-out" : "crashed",
        ms,
        fileCount: countVueFiles(fixture.cwd),
        errorCount: 0,
        reason: commandError.stderr?.toString().trim().split("\n").slice(-1)[0],
      };
    }
  }

  const ms = performance.now() - start;
  const parsed = JSON.parse(stdout) as { fileCount?: number; errorCount?: number };
  return {
    name: fixture.name,
    status: "ok",
    ms,
    fileCount: parsed.fileCount ?? countVueFiles(fixture.cwd),
    errorCount: parsed.errorCount ?? 0,
  };
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

console.log();
console.log("-".repeat(65));
console.log();
console.log(" Real-world typechecker fixtures:");
console.log();

for (const result of REAL_WORLD_TYPECHECK_FIXTURES.map(runVizeRealWorldTypecheck)) {
  if (result.status !== "ok") {
    const reason = result.reason ? ` (${result.reason})` : "";
    console.log(`   ${result.name.padEnd(15)}: ${result.status.toUpperCase()}${reason}`);
    continue;
  }

  console.log(
    `   ${result.name.padEnd(15)}: ${formatTime(result.ms).padStart(8)}  (${formatThroughput(result.fileCount, result.ms)}, ${result.fileCount} SFC files, diagnostics=${result.errorCount})`,
  );
}

console.log();
