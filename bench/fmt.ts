/**
 * Format Benchmark: Vize (glyph) vs Prettier
 *
 * Usage:
 *   1. Build CLI: mise run build:cli
 *   2. Run benchmark: node --experimental-strip-types bench/fmt.ts
 *
 * Input files are regenerated before each format run to ensure
 * consistent (unformatted) input.
 */

import { existsSync, readdirSync, statSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, join } from "node:path";
import { spawnSync } from "node:child_process";
import os from "node:os";

const __dirname = dirname(fileURLToPath(import.meta.url));
const INPUT_DIR = join(__dirname, "__in__");
const CPU_COUNT = os.cpus().length;
const VIZE_BIN = join(__dirname, "..", "target", "release", "vize");
const FORMAT_PATTERN = "*.vue";
const GENERATE_SCRIPT = join(__dirname, "generate.mjs");
const FILE_COUNT = parseInt(process.argv[2] || "15000", 10) || 15000;
const PRETTIER_BIN_CANDIDATES = [
  join(__dirname, "node_modules", ".bin", "prettier"),
  join(__dirname, "..", "node_modules", ".bin", "prettier"),
];

// Regenerate input files (ensures fresh, unformatted state)
function regenerateInput(): void {
  runCommand(`node ${shellQuote(GENERATE_SCRIPT)} ${FILE_COUNT}`, __dirname);
}

// Initial generation to get file metadata
regenerateInput();

const vueFiles = readdirSync(INPUT_DIR).filter((f) => f.endsWith(".vue"));
if (vueFiles.length === 0) {
  console.error(`Error: No .vue files found in ${INPUT_DIR}\ngenerate.mjs failed.`);
  process.exit(1);
}

const totalSize = vueFiles.reduce((sum, f) => sum + statSync(join(INPUT_DIR, f)).size, 0);

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

function formatBytesPerSec(bytes: number, ms: number): string {
  const bps = (bytes / ms) * 1000;
  if (bps >= 1024 * 1024) return `${(bps / 1024 / 1024).toFixed(1)} MB/s`;
  if (bps >= 1024) return `${(bps / 1024).toFixed(1)} KB/s`;
  return `${bps.toFixed(0)} B/s`;
}

function shellQuote(value: string): string {
  return `'${value.replaceAll("'", "'\\''")}'`;
}

function resolvePrettierBin(): string | null {
  for (const candidate of PRETTIER_BIN_CANDIDATES) {
    if (existsSync(candidate)) return candidate;
  }
  return null;
}

type CommandResult = {
  ms: number;
  output: string;
};

function runCommand(
  cmd: string,
  cwd: string = INPUT_DIR,
  env: NodeJS.ProcessEnv = {},
): CommandResult {
  const start = performance.now();
  const result = spawnSync(cmd, {
    cwd,
    encoding: "utf8",
    env: { ...process.env, ...env },
    shell: true,
  });
  const ms = performance.now() - start;
  const output = `${result.stdout ?? ""}${result.stderr ?? ""}`;

  if (result.error) throw result.error;
  if (result.status !== 0) {
    const tail = output.trim().split("\n").slice(-20).join("\n");
    throw new Error(`Command failed (${result.status}): ${cmd}\n${tail}`);
  }

  return { ms, output };
}

function benchmarkFormatter(
  cmd: string,
  validate: (result: CommandResult) => void = () => {},
  env: NodeJS.ProcessEnv = {},
): number {
  regenerateInput();
  for (let i = 0; i < 3; i++) {
    validate(runCommand(cmd, INPUT_DIR, env));
    regenerateInput();
  }

  regenerateInput();
  const result = runCommand(cmd, INPUT_DIR, env);
  validate(result);
  return result.ms;
}

function assertVizeFileCount(result: CommandResult): void {
  const match = result.output.match(/Found (\d+) \.vue file\(s\)/);
  if (!match) {
    throw new Error("Vize fmt output did not include the collected file count.");
  }
  const found = Number.parseInt(match[1], 10);
  if (found !== vueFiles.length) {
    throw new Error(`Vize fmt matched ${found} files, expected ${vueFiles.length}.`);
  }
}

// Prettier CLI (inherently single-threaded)
function runPrettier(): number {
  const prettierBin = resolvePrettierBin();
  if (prettierBin == null) {
    return -1;
  }
  return benchmarkFormatter(
    `${shellQuote(prettierBin)} --write ${shellQuote(FORMAT_PATTERN)} --log-level error`,
  );
}

// Vize (glyph) single-thread
function runVizeFmtSingleThread(): number {
  return benchmarkFormatter(
    `${shellQuote(VIZE_BIN)} fmt --write ${shellQuote(FORMAT_PATTERN)}`,
    assertVizeFileCount,
    { RAYON_NUM_THREADS: "1" },
  );
}

// Vize (glyph) multi-thread
function runVizeFmtMultiThread(): number {
  return benchmarkFormatter(
    `${shellQuote(VIZE_BIN)} fmt --write ${shellQuote(FORMAT_PATTERN)}`,
    assertVizeFileCount,
  );
}

// Main
console.log();
console.log("=".repeat(65));
console.log(" Format Benchmark: glyph vs prettier");
console.log("=".repeat(65));
console.log();
console.log(` Files     : ${vueFiles.length.toLocaleString()} SFC files`);
console.log(` Total Size: ${(totalSize / 1024 / 1024).toFixed(1)} MB`);
console.log(` CPU Cores : ${CPU_COUNT}`);
console.log();
console.log("-".repeat(65));

console.log();

const prettierTime = runPrettier();
if (prettierTime >= 0) {
  console.log(
    `   Prettier (CLI)      : ${formatTime(prettierTime).padStart(8)}  (${formatThroughput(vueFiles.length, prettierTime)}, ${formatBytesPerSec(totalSize, prettierTime)})`,
  );
} else {
  console.log("   Prettier (CLI)      : SKIPPED (not found)");
}

let vizeSingle = 0;
let vizeMulti = 0;

if (existsSync(VIZE_BIN)) {
  vizeSingle = runVizeFmtSingleThread();
  if (prettierTime >= 0) {
    const stSpeedup = (prettierTime / vizeSingle).toFixed(1);
    console.log(
      `   Vize glyph (1T)     : ${formatTime(vizeSingle).padStart(8)}  (${formatThroughput(vueFiles.length, vizeSingle)}, ${formatBytesPerSec(totalSize, vizeSingle)})  ${stSpeedup}x faster`,
    );
  } else {
    console.log(
      `   Vize glyph (1T)     : ${formatTime(vizeSingle).padStart(8)}  (${formatThroughput(vueFiles.length, vizeSingle)}, ${formatBytesPerSec(totalSize, vizeSingle)})`,
    );
  }

  vizeMulti = runVizeFmtMultiThread();
  if (prettierTime >= 0) {
    const mtSpeedup = (prettierTime / vizeMulti).toFixed(1);
    console.log(
      `   Vize glyph (${CPU_COUNT}T)    : ${formatTime(vizeMulti).padStart(8)}  (${formatThroughput(vueFiles.length, vizeMulti)}, ${formatBytesPerSec(totalSize, vizeMulti)})  ${mtSpeedup}x faster`,
    );
  } else {
    console.log(
      `   Vize glyph (${CPU_COUNT}T)    : ${formatTime(vizeMulti).padStart(8)}  (${formatThroughput(vueFiles.length, vizeMulti)}, ${formatBytesPerSec(totalSize, vizeMulti)})`,
    );
  }
} else {
  console.log("   Vize (glyph)  : SKIPPED (vize CLI not found)");
}

// Summary
if (prettierTime >= 0 && vizeSingle > 0 && vizeMulti > 0) {
  console.log();
  console.log("-".repeat(65));
  console.log();
  console.log(" Summary:");
  console.log();
  const stSpeedup = (prettierTime / vizeSingle).toFixed(1);
  const mtSpeedup = (prettierTime / vizeMulti).toFixed(1);
  console.log(`   Prettier vs Vize ST : ${stSpeedup}x`);
  console.log(`   Prettier vs Vize MT : ${mtSpeedup}x`);
}

console.log();
