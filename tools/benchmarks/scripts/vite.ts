/**
 * Vite Plugin Benchmark: @vizejs/vite-plugin vs @vitejs/plugin-vue
 *
 * Copies exactly FILE_LIMIT .vue files into a dedicated working directory
 * so that both plugins compile the exact same number of files.
 *
 * Usage:
 *   1. Generate test files: node generate.mjs [count]
 *   2. Build vite-plugin: vp run --workspace-root build:vite-plugin
 *   3. Run benchmark: node tools/benchmarks/scripts/vite.ts [count]
 */

import {
  existsSync,
  readdirSync,
  statSync,
  mkdirSync,
  rmSync,
  writeFileSync,
  copyFileSync,
} from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, join, resolve } from "node:path";
import os from "node:os";

import {
  assertPrecompileCacheCold,
  clearPrecompileCache,
  readPrecompileCacheEntries,
} from "./vite-precompile-cache.mjs";

const __dirname = dirname(fileURLToPath(import.meta.url));
const repoRoot = resolve(__dirname, "..", "..", "..");
const INPUT_DIR = join(__dirname, "__in__");
const WORK_DIR = join(__dirname, "__vite_work__");
const TEMP_DIR = join(__dirname, "__vite_tmp__");
const CPU_COUNT = os.cpus().length;

// Check input files
if (!existsSync(INPUT_DIR)) {
  console.error(`Error: Input directory not found: ${INPUT_DIR}\nRun 'node generate.mjs' first.`);
  process.exit(1);
}

const allVueFiles = readdirSync(INPUT_DIR).filter((f) => f.endsWith(".vue"));
if (allVueFiles.length === 0) {
  console.error(`Error: No .vue files found in ${INPUT_DIR}\nRun 'node generate.mjs' first.`);
  process.exit(1);
}

// FILE_LIMIT: number of .vue files to use (full 15k causes OOM in rollup)
const FILE_LIMIT = parseInt(process.argv[2] || "1000", 10);
const vueFiles = allVueFiles.slice(0, FILE_LIMIT);

// Create working directory with only the files we need
rmSync(WORK_DIR, { recursive: true, force: true });
mkdirSync(WORK_DIR, { recursive: true });

for (const f of vueFiles) {
  copyFileSync(join(INPUT_DIR, f), join(WORK_DIR, f));
}

const totalSize = vueFiles.reduce((sum, f) => sum + statSync(join(WORK_DIR, f)).size, 0);

// Generate entry that imports all files in the working directory
const entryImports = vueFiles.map((f, i) => {
  const name = `C${i}`;
  return { name, imp: `import ${name} from './${f}'` };
});
const entryContent = `${entryImports.map((e) => e.imp).join("\n")}
import { createApp, h } from 'vue'
const app = createApp({
  render() { return h('div', [${entryImports.map((e) => `h(${e.name})`).join(",")}]) }
})
app.mount('#app')
`;
const ENTRY_FILE = join(WORK_DIR, "__entry__.ts");
writeFileSync(ENTRY_FILE, entryContent);

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

// Build with @vitejs/plugin-vue
async function buildWithOfficialPlugin(): Promise<number> {
  const { build } = await import("vite");
  const vuePlugin = (await import("@vitejs/plugin-vue")).default;

  const outDir = join(TEMP_DIR, "official");
  mkdirSync(outDir, { recursive: true });

  const start = performance.now();
  await build({
    root: WORK_DIR,
    plugins: [vuePlugin()],
    build: {
      outDir,
      write: true,
      minify: false,
      rollupOptions: {
        input: ENTRY_FILE,
        external: ["vue"],
      },
    },
    logLevel: "silent",
  });
  return performance.now() - start;
}

/**
 * Build with @vizejs/vite-plugin at a stated persistent-cache state.
 *
 * `cache: "cold"` clears the on-disk pre-compile manifest first and asserts it
 * is gone, so the build compiles every SFC exactly like `@vitejs/plugin-vue`
 * does. `cache: "warm"` deliberately keeps whatever the previous build left, and
 * its time is reported separately because the baseline plugin has no equivalent.
 */
async function buildWithVizePlugin(cache: "cold" | "warm"): Promise<number> {
  const { build } = await import("vite");

  let vizePlugin: any;
  try {
    vizePlugin = (await import(join(repoRoot, "npm", "builder/vite", "dist", "index.mjs"))).default;
  } catch {
    return -1;
  }

  if (cache === "cold") {
    clearPrecompileCache(WORK_DIR);
    assertPrecompileCacheCold(WORK_DIR, "@vizejs/vite-plugin");
  }

  const outDir = join(TEMP_DIR, "vize");
  mkdirSync(outDir, { recursive: true });

  const start = performance.now();
  await build({
    root: WORK_DIR,
    plugins: [
      vizePlugin({
        scanPatterns: ["*.vue"],
      }),
    ],
    build: {
      outDir,
      write: true,
      minify: false,
      rollupOptions: {
        input: ENTRY_FILE,
        external: ["vue"],
      },
    },
    logLevel: "silent",
  });
  return performance.now() - start;
}

// Main
console.log();
console.log("=".repeat(65));
console.log(" Vite Plugin Benchmark: @vizejs/vite-plugin vs @vitejs/plugin-vue");
console.log("=".repeat(65));
console.log();
console.log(` Files     : ${vueFiles.length.toLocaleString()} SFC files (all imported in entry)`);
console.log(` Total Size: ${(totalSize / 1024 / 1024).toFixed(1)} MB`);
console.log(` CPU Cores : ${CPU_COUNT}`);
console.log();
console.log("-".repeat(65));

// Ensure temp dir
mkdirSync(TEMP_DIR, { recursive: true });

// Warmup
console.log();
console.log(" Warming up...");
await buildWithOfficialPlugin();
await buildWithVizePlugin("warm");
// The warmup left a pre-compile manifest in WORK_DIR. Report what it holds so
// the reader can see the cache dimension exists rather than inferring it.
const warmupCacheEntries = readPrecompileCacheEntries(WORK_DIR);

// Benchmark
console.log();
console.log(" Build (Vite programmatic API):");
console.log();

const officialTime = await buildWithOfficialPlugin();
console.log(
  `   @vitejs/plugin-vue         : ${formatTime(officialTime).padStart(8)}  (${formatThroughput(vueFiles.length, officialTime)})`,
);

// Measured cold: the persistent pre-compile manifest is cleared first, so this
// build compiles all FILE_LIMIT files just as @vitejs/plugin-vue does. This is
// the comparable number and the one the summary uses.
const vizeTime = await buildWithVizePlugin("cold");
if (vizeTime >= 0) {
  const speedup = (officialTime / vizeTime).toFixed(1);
  console.log(
    `   @vizejs/vite-plugin (cold) : ${formatTime(vizeTime).padStart(8)}  (${formatThroughput(vueFiles.length, vizeTime)})  ${speedup}x faster`,
  );
  // The cold build above wrote the manifest, so this run restores from it.
  const vizeWarmTime = await buildWithVizePlugin("warm");
  console.log(
    `   @vizejs/vite-plugin (warm) : ${formatTime(vizeWarmTime).padStart(8)}  (${formatThroughput(vueFiles.length, vizeWarmTime)})  not comparable`,
  );
  console.log();
  console.log(
    "   (warm) reuses the on-disk pre-compile cache in node_modules/.vize; @vitejs/plugin-vue",
  );
  console.log("   has no equivalent, so only the (cold) row is an apples-to-apples comparison.");
  if (warmupCacheEntries.length > 0) {
    console.log(`   Warmup manifest that was cleared: ${warmupCacheEntries.join(", ")}`);
  }
} else {
  console.log(
    "   @vizejs/vite-plugin        : SKIPPED (plugin not built, run 'vp run --workspace-root build:vite-plugin')",
  );
}

// Cleanup
try {
  rmSync(TEMP_DIR, { recursive: true, force: true });
  rmSync(WORK_DIR, { recursive: true, force: true });
} catch {
  // ignore
}

// Summary
if (vizeTime >= 0) {
  console.log();
  console.log("-".repeat(65));
  console.log();
  console.log(" Summary:");
  console.log();
  const speedup = (officialTime / vizeTime).toFixed(1);
  console.log(`   @vitejs/plugin-vue vs @vizejs/vite-plugin (cold) : ${speedup}x`);
}

console.log();
