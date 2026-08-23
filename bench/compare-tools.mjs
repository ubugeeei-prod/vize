#!/usr/bin/env node
/**
 * Compare Vize against the incumbent Vue tooling stack and emit stable
 * Markdown/JSON that can be used in PR comments and documentation snapshots.
 */

import { spawnSync } from "node:child_process";
import { createRequire } from "node:module";
import os from "node:os";
import {
  copyFileSync,
  existsSync,
  mkdirSync,
  readdirSync,
  readFileSync,
  rmSync,
  statSync,
  writeFileSync,
} from "node:fs";
import { basename, delimiter, dirname, join, parse, relative, resolve, sep } from "node:path";
import { performance } from "node:perf_hooks";
import { fileURLToPath, pathToFileURL } from "node:url";
import { Worker } from "node:worker_threads";

import { renderProvenanceLines, resolveBackend } from "./benchmark-provenance.mjs";
import { createLargeSfcSource } from "./compare-tools-large-sfc.mjs";
import { buildMetadata } from "./compare-tools-metadata.mjs";
import { DEFAULT_MUSEA_FILE_COUNT, measureMuseaSurface } from "./compare-tools-musea.mjs";
import {
  createSurface,
  ENGINE_CLASSES_BY_SURFACE,
  renderEngineClassSections,
  renderSurfaceTable,
} from "./compare-tools-report.mjs";
import {
  createTypecheckToolVariants,
  prepareTypecheckDir,
  typecheckToolBins,
} from "./compare-tools-typecheck.mjs";
import { createNativeBatchSequenceVariants, measureNativeBatchCompile } from "./native-batch.mjs";
import { linkColdNodeModules } from "./nuxt-build-cache.mjs";

export { createSurface } from "./compare-tools-report.mjs";

const require = createRequire(import.meta.url);
const benchDir = dirname(fileURLToPath(import.meta.url));
const rootDir = dirname(benchDir);
const workRoot = join(rootDir, "target", "tool-benchmark");
const cpuCount = os.cpus().length;
const explicitMaxThreads = Math.max(1, Math.min(os.availableParallelism(), 256));

const DEFAULT_RUNS = 5;
const DEFAULT_WARMUPS = 1;
const DEFAULT_CHECK_FILE_COUNT = 500;
const DEFAULT_VITE_FILE_COUNT = 1000;
const DEFAULT_NUXT_FILE_COUNT = 500;
const DEFAULT_LARGE_BLOCKS = 900;
export const DEFAULT_TASKS = ["compile", "large", "lint", "fmt", "check", "vite", "nuxt", "musea"];

function parseArgs(argv) {
  const args = {};
  for (let i = 0; i < argv.length; i++) {
    const arg = argv[i];
    if (!arg.startsWith("--")) {
      continue;
    }
    const key = arg.slice(2);
    const next = argv[i + 1];
    if (next == null || next.startsWith("--")) {
      args[key] = "true";
    } else {
      args[key] = next;
      i++;
    }
  }
  return args;
}

function requireArg(args, key) {
  const value = args[key];
  if (!value) {
    throw new Error(`Missing required argument: --${key}`);
  }
  return value;
}

function parsePositiveInt(value, fallback) {
  const parsed = Number.parseInt(value ?? "", 10);
  return Number.isFinite(parsed) && parsed > 0 ? parsed : fallback;
}

function parseNonNegativeInt(value, fallback) {
  const parsed = Number.parseInt(value ?? "", 10);
  return Number.isFinite(parsed) && parsed >= 0 ? parsed : fallback;
}

function selectedTasks(value) {
  const requested = new Set(
    (value ?? DEFAULT_TASKS.join(","))
      .split(",")
      .map((task) => task.trim())
      .filter(Boolean),
  );
  return DEFAULT_TASKS.filter((task) => requested.has(task));
}

function median(values) {
  const sorted = [...values].sort((a, b) => a - b);
  const mid = Math.floor(sorted.length / 2);
  if (sorted.length % 2 === 1) {
    return sorted[mid];
  }
  return (sorted[mid - 1] + sorted[mid]) / 2;
}

export function formatMs(ms) {
  if (!Number.isFinite(ms)) {
    return "n/a";
  }
  if (ms >= 1000) {
    return `${(ms / 1000).toLocaleString("en-US", {
      minimumFractionDigits: 2,
      maximumFractionDigits: 2,
    })}s`;
  }
  return `${ms.toLocaleString("en-US", {
    minimumFractionDigits: 1,
    maximumFractionDigits: 1,
  })}ms`;
}

function formatRunList(values) {
  return values.map(formatMs).join(", ");
}

function formatThroughput(files, ms) {
  if (!Number.isFinite(ms) || ms <= 0) {
    return "n/a";
  }
  const filesPerSecond = (files / ms) * 1000;
  if (filesPerSecond >= 1000) {
    return `${(filesPerSecond / 1000).toFixed(1)}k files/s`;
  }
  return `${filesPerSecond.toFixed(0)} files/s`;
}

function formatBytes(bytes) {
  if (bytes >= 1024 * 1024) {
    return `${(bytes / 1024 / 1024).toFixed(1)} MB`;
  }
  if (bytes >= 1024) {
    return `${(bytes / 1024).toFixed(1)} KB`;
  }
  return `${bytes} B`;
}

function pathWithNodeBins(cwd) {
  const dirs = [];
  let current = cwd;
  const root = parse(current).root;
  while (true) {
    const candidate = join(current, "node_modules", ".bin");
    if (existsSync(candidate)) {
      dirs.push(candidate);
    }
    if (current === root) {
      break;
    }
    current = dirname(current);
  }
  return [...dirs.reverse(), process.env.PATH ?? ""].join(delimiter);
}

function shellEnv(cwd, extraEnv = {}) {
  return {
    ...process.env,
    NO_COLOR: "1",
    VIZE_BENCH: "1",
    PATH: pathWithNodeBins(cwd),
    ...extraEnv,
  };
}

function runCommand(binary, commandArgs, options) {
  const start = performance.now();
  const result = spawnSync(binary, commandArgs, {
    cwd: options.cwd,
    env: shellEnv(options.cwd, options.env),
    encoding: "utf8",
    maxBuffer: 64 * 1024 * 1024,
  });
  const elapsedMs = performance.now() - start;

  if (result.error) {
    throw result.error;
  }
  if (result.status !== 0 && !options.allowNonZeroExit) {
    const output = `${result.stdout ?? ""}\n${result.stderr ?? ""}`.trim();
    throw new Error(
      `${basename(binary)} ${commandArgs.join(" ")} exited with ${result.status}\n${output}`,
    );
  }
  return elapsedMs;
}

function resolveWorkspaceBin(name) {
  const suffixes = process.platform === "win32" ? ["", ".cmd", ".ps1"] : [""];
  const candidates = [
    join(rootDir, "node_modules", ".bin", name),
    join(benchDir, "node_modules", ".bin", name),
    join(rootDir, "npm", "framework/nuxt", "node_modules", ".bin", name),
  ];
  for (const candidate of candidates) {
    for (const suffix of suffixes) {
      const bin = `${candidate}${suffix}`;
      if (existsSync(bin)) {
        return bin;
      }
    }
  }
  throw new Error(`Could not resolve ${name} from workspace node_modules/.bin`);
}

/** Same lookup as resolveWorkspaceBin, but null instead of throwing: a tool
 * that is not installed records a null version rather than failing the run. */
function optionalWorkspaceBin(name) {
  try {
    return resolveWorkspaceBin(name);
  } catch {
    return null;
  }
}

function collectVueFiles(inputDir, limit = Infinity) {
  return readdirSync(inputDir)
    .filter((file) => file.endsWith(".vue"))
    .sort()
    .slice(0, limit);
}

function totalFileBytes(inputDir, files) {
  return files.reduce((sum, file) => sum + statSync(join(inputDir, file)).size, 0);
}

function copySelectedFiles(inputDir, outputDir, files, extraFiles = []) {
  rmSync(outputDir, { recursive: true, force: true });
  mkdirSync(outputDir, { recursive: true });

  for (const file of files) {
    copyFileSync(join(inputDir, file), join(outputDir, file));
  }
  for (const file of extraFiles) {
    const source = join(inputDir, file);
    if (existsSync(source)) {
      copyFileSync(source, join(outputDir, file));
    }
  }
}

function prepareFormatDir(inputDir, files, label, invocation) {
  const outputDir = join(workRoot, "fmt", `${label}-${String(invocation).padStart(4, "0")}`);
  copySelectedFiles(inputDir, outputDir, files, ["vize.config.json"]);
  return outputDir;
}

function prepareViteDir(inputDir, files, label, invocation) {
  const outputDir = join(workRoot, "vite", `${label}-${String(invocation).padStart(4, "0")}`);
  copySelectedFiles(inputDir, outputDir, files, ["vize.config.json"]);

  const imports = [];
  const components = [];
  for (let i = 0; i < files.length; i++) {
    const name = `C${i}`;
    imports.push(`import ${name} from './${files[i]}'`);
    components.push(name);
  }
  const entryFile = join(outputDir, "__entry__.ts");
  writeFileSync(
    entryFile,
    `${imports.join("\n")}
import { createApp, h } from 'vue'

const app = createApp({
  render() {
    return h('div', [${components.map((component) => `h(${component})`).join(", ")}])
  }
})
app.mount('#app')
`,
  );
  writeFileSync(
    join(outputDir, "index.html"),
    `<!doctype html>
<html>
<body>
  <div id="app"></div>
  <script type="module" src="./__entry__.ts"></script>
</body>
</html>
`,
  );

  return { workDir: outputDir, entryFile };
}

function prepareLargeSfcDir(blockCount) {
  const outputDir = join(workRoot, "large-sfc");
  rmSync(outputDir, { recursive: true, force: true });
  mkdirSync(outputDir, { recursive: true });

  const filename = "LargeDashboard.vue";
  writeFileSync(join(outputDir, filename), createLargeSfcSource(blockCount));
  writeFileSync(
    join(outputDir, "tsconfig.json"),
    `${JSON.stringify(
      {
        compilerOptions: {
          target: "ESNext",
          module: "ESNext",
          moduleResolution: "bundler",
          strict: true,
          jsx: "preserve",
          noEmit: true,
          skipLibCheck: true,
          paths: {
            vue: [
              relative(outputDir, join(rootDir, "node_modules", "vue"))
                .split(sep)
                .join("/"),
            ],
          },
        },
        include: [filename],
      },
      null,
      2,
    )}\n`,
  );
  writeFileSync(
    join(outputDir, "vize.config.json"),
    `${JSON.stringify({ typeChecker: {} }, null, 2)}\n`,
  );

  return {
    dir: outputDir,
    files: [filename],
    bytes: totalFileBytes(outputDir, [filename]),
  };
}

function prepareNuxtDir(inputDir, files, label, invocation, useVize) {
  const outputDir = join(workRoot, "nuxt", `${label}-${String(invocation).padStart(4, "0")}`);
  rmSync(outputDir, { recursive: true, force: true });
  mkdirSync(join(outputDir, "components"), { recursive: true });

  for (const file of files) {
    copyFileSync(join(inputDir, file), join(outputDir, "components", file));
  }
  linkColdNodeModules(
    outputDir,
    [join(rootDir, "npm", "framework/nuxt", "node_modules"), join(benchDir, "node_modules")],
    label,
  );

  const imports = [];
  const componentNames = [];
  for (let i = 0; i < files.length; i++) {
    const name = `BenchComponent${i}`;
    imports.push(`import ${name} from './components/${files[i]}'`);
    componentNames.push(name);
  }

  writeFileSync(
    join(outputDir, "app.vue"),
    `<template>
  <main>
    <component
      v-for="(BenchComponent, index) in benchComponents"
      :key="index"
      :is="BenchComponent"
    />
  </main>
</template>

<script setup lang="ts">
${imports.join("\n")}

const benchComponents = [${componentNames.join(", ")}]
</script>
`,
  );

  const vizeModuleUrl = pathToFileURL(join(rootDir, "npm/framework/nuxt/dist/index.mjs")).href;
  const moduleImport = useVize ? `import vizeNuxt from '${vizeModuleUrl}'\n` : "";
  const modules = useVize ? "modules: [vizeNuxt]," : "modules: [],";
  const vizeOptions = useVize
    ? `  vize: {
    compiler: {
      scanPatterns: ['app.vue', 'components/*.vue'],
      precompileBatchSize: ${files.length + 1},
    },
  },`
    : "";

  writeFileSync(
    join(outputDir, "nuxt.config.mjs"),
    `import { defineNuxtConfig } from 'nuxt/config'
${moduleImport}export default defineNuxtConfig({
  devtools: { enabled: false },
  telemetry: false,
  ssr: false,
  typescript: {
    typeCheck: false,
  },
  ${modules}
${vizeOptions}
})
`,
  );
  writeFileSync(
    join(outputDir, "package.json"),
    `${JSON.stringify(
      {
        private: true,
        type: "module",
        dependencies: {
          nuxt: "4.4.6",
          vue: "3.5.34",
        },
      },
      null,
      2,
    )}\n`,
  );

  return outputDir;
}

async function measureVariants(variants, options) {
  for (let i = 0; i < options.warmups; i++) {
    for (const variant of variants) {
      await variant.measure({ phase: "warmup", iteration: i });
    }
  }

  const runsById = new Map(variants.map((variant) => [variant.id, []]));
  for (let i = 0; i < options.runs; i++) {
    const ordered = i % 2 === 0 ? variants : [...variants].reverse();
    for (const variant of ordered) {
      const ms = await variant.measure({ phase: "measure", iteration: i });
      runsById.get(variant.id).push(ms);
    }
  }

  return variants.map((variant) => {
    const runs = runsById.get(variant.id).map((ms) => Number(ms.toFixed(3)));
    const medianMs = Number(median(runs).toFixed(3));
    return {
      id: variant.id,
      label: variant.label,
      medianMs,
      runs,
      throughput: formatThroughput(variant.files, medianMs),
    };
  });
}

function vueCompileSfc(compiler, source, filename) {
  const { descriptor } = compiler.parse(source, { filename });
  let bindings = {};
  let scriptCode = "";

  if (descriptor.scriptSetup || descriptor.script) {
    const scriptResult = compiler.compileScript(descriptor, { id: filename });
    bindings = scriptResult.bindings || {};
    scriptCode = scriptResult.content;
  }

  let templateCode = "";
  if (descriptor.template) {
    const templateResult = compiler.compileTemplate({
      source: descriptor.template.content,
      filename,
      id: filename,
      compilerOptions: { bindingMetadata: bindings },
    });
    templateCode = templateResult.code;
  }

  return `${scriptCode}\n${templateCode}`;
}

async function runVueCompilerWorkers(files, compilerSfcPath) {
  const workerCount = Math.min(cpuCount, files.length);
  const chunkSize = Math.ceil(files.length / workerCount);
  const workerCode = `
    const { parentPort, workerData } = require("worker_threads");
    const compiler = require(workerData.compilerSfcPath);

    function compileSfc(source, filename) {
      const { descriptor } = compiler.parse(source, { filename });
      let bindings = {};
      if (descriptor.scriptSetup || descriptor.script) {
        const scriptResult = compiler.compileScript(descriptor, { id: filename });
        bindings = scriptResult.bindings || {};
      }
      if (descriptor.template) {
        compiler.compileTemplate({
          source: descriptor.template.content,
          filename,
          id: filename,
          compilerOptions: { bindingMetadata: bindings },
        });
      }
    }

    for (const file of workerData.files) {
      compileSfc(file.source, file.filename);
    }
    parentPort.postMessage("done");
  `;

  const workers = [];
  for (let i = 0; i < workerCount; i++) {
    const startIndex = i * chunkSize;
    const endIndex = Math.min(startIndex + chunkSize, files.length);
    const chunk = files.slice(startIndex, endIndex);
    if (chunk.length === 0) {
      continue;
    }

    const worker = new Worker(workerCode, {
      eval: true,
      workerData: { files: chunk, compilerSfcPath },
    });

    workers.push(
      new Promise((resolvePromise, reject) => {
        worker.on("message", resolvePromise);
        worker.on("error", reject);
        worker.on("exit", (code) => {
          if (code !== 0) {
            reject(new Error(`@vue/compiler-sfc worker exited with ${code}`));
          }
        });
      }),
    );
  }

  await Promise.all(workers);
}

async function runEslintWorkers(inputDir, files, eslintPath) {
  const workerCount = Math.min(cpuCount, files.length);
  const chunkSize = Math.ceil(files.length / workerCount);
  const workerCode = `
    const { parentPort, workerData } = require("worker_threads");
    const { ESLint } = require(workerData.eslintPath);

    (async () => {
      const eslint = new ESLint({
        overrideConfigFile: workerData.configFile,
        cwd: workerData.cwd,
      });
      await eslint.lintFiles(workerData.files);
      parentPort.postMessage("done");
    })().catch((error) => {
      parentPort.postMessage({ error: error && error.stack ? error.stack : String(error) });
    });
  `;

  const workers = [];
  for (let i = 0; i < workerCount; i++) {
    const startIndex = i * chunkSize;
    const endIndex = Math.min(startIndex + chunkSize, files.length);
    const chunk = files.slice(startIndex, endIndex).map((file) => join(inputDir, file));
    if (chunk.length === 0) {
      continue;
    }

    const worker = new Worker(workerCode, {
      eval: true,
      workerData: {
        cwd: inputDir,
        configFile: join(inputDir, "eslint.config.mjs"),
        files: chunk,
        eslintPath,
      },
    });

    workers.push(
      new Promise((resolvePromise, reject) => {
        worker.on("message", (message) => {
          if (message && typeof message === "object" && "error" in message) {
            reject(new Error(message.error));
            return;
          }
          resolvePromise(message);
        });
        worker.on("error", reject);
        worker.on("exit", (code) => {
          if (code !== 0) {
            reject(new Error(`ESLint worker exited with ${code}`));
          }
        });
      }),
    );
  }

  await Promise.all(workers);
}

function timedSync(fn) {
  const start = performance.now();
  fn();
  return performance.now() - start;
}

async function timedAsync(fn) {
  const start = performance.now();
  await fn();
  return performance.now() - start;
}

function loadNativeBindings() {
  const nativePath = join(rootDir, "npm", "native");
  try {
    return require(nativePath);
  } catch (error) {
    throw new Error(
      `Could not load @vizejs/native from ${nativePath}. Run vp run --workspace-root build:native first.\n${error instanceof Error ? error.message : String(error)}`,
    );
  }
}

function assertNativeCompileResult(result, filename) {
  if (!result || typeof result !== "object") {
    throw new Error(`Vize native compile returned an invalid result for ${filename}.`);
  }
  if (Array.isArray(result.errors) && result.errors.length > 0) {
    throw new Error(`Vize native compile failed for ${filename}: ${result.errors.join("; ")}`);
  }
}

function measureNativeBatchResultsCompile(native, sources, expectedFiles) {
  const start = performance.now();
  const result = native.compileSfcBatchWithResults(
    sources.map((file) => ({ path: file.filename, source: file.source })),
  );
  const ms = performance.now() - start;
  if (!result || typeof result !== "object") {
    throw new Error("Vize native batch-with-results compile returned an invalid result.");
  }
  if (result.failedCount !== 0) {
    throw new Error(
      `Vize native batch-with-results compile failed for ${result.failedCount} file(s).`,
    );
  }
  if (result.successCount !== expectedFiles) {
    throw new Error(
      `Vize native batch-with-results compiled ${result.successCount} files, expected ${expectedFiles}.`,
    );
  }
  return ms;
}

async function measureCompile(inputDir, files, options) {
  const compiler = await import("@vue/compiler-sfc");
  const compilerSfcPath = require.resolve("@vue/compiler-sfc");
  const native = loadNativeBindings();
  const sources = files.map((filename) => ({
    filename,
    source: readFileSync(join(inputDir, filename), "utf8"),
  }));
  const pattern = join(inputDir, "*.vue");

  const variants = [
    {
      id: "vue-compiler-sfc-1t",
      label: "@vue/compiler-sfc (1T)",
      files: files.length,
      measure: () =>
        timedSync(() => {
          for (const file of sources) {
            vueCompileSfc(compiler, file.source, file.filename);
          }
        }),
    },
    {
      id: "vue-compiler-sfc-workers",
      label: `@vue/compiler-sfc (${Math.min(cpuCount, files.length)} workers)`,
      files: files.length,
      measure: () => timedAsync(() => runVueCompilerWorkers(sources, compilerSfcPath)),
    },
    {
      id: "vize-native-1t",
      label: "Vize native loop (1T)",
      files: files.length,
      measure: () =>
        timedSync(() => {
          for (const file of sources) {
            assertNativeCompileResult(
              native.compileSfc(file.source, { filename: file.filename }),
              file.filename,
            );
          }
        }),
    },
    {
      id: "vize-native-max",
      label: "Vize native batch results (max)",
      files: files.length,
      measure: () => measureNativeBatchResultsCompile(native, sources, files.length),
    },
    {
      id: "vize-native-core-max",
      label: "Vize native batch stats-only (core max)",
      files: files.length,
      measure: () => measureNativeBatchCompile(native, pattern, files.length),
    },
    ...createNativeBatchSequenceVariants({
      native,
      pattern,
      expectedFiles: files.length,
      maxThreads: explicitMaxThreads,
    }),
  ];

  return createSurface({
    id: "compile",
    label: "SFC compile",
    files: files.length,
    bytes: totalFileBytes(inputDir, files),
    variants: await measureVariants(variants, options),
    baselineId: "vue-compiler-sfc-1t",
    vizeSingleId: "vize-native-1t",
    vizeMaxId: "vize-native-max",
  });
}

async function measureLargeSfc(largeSfc, options) {
  const compile = await measureCompile(largeSfc.dir, largeSfc.files, options);
  const check = await measureCheck(largeSfc.dir, largeSfc.files, options);

  return [
    {
      ...compile,
      id: "large-compile",
      label: "Large SFC compile",
    },
    {
      ...check,
      id: "large-check",
      label: "Large SFC type check",
    },
  ];
}

async function measureLint(inputDir, files, options) {
  const { ESLint } = await import("eslint");
  const eslintPath = require.resolve("eslint");
  const vizeBin = resolve(options.vizeBin);
  if (!existsSync(vizeBin)) {
    throw new Error(`Vize CLI not found: ${vizeBin}`);
  }

  const filePaths = files.map((file) => join(inputDir, file));
  const variants = [
    {
      id: "eslint-plugin-vue-1t",
      label: "eslint-plugin-vue (1T)",
      files: files.length,
      measure: async () => {
        const eslint = new ESLint({
          overrideConfigFile: join(inputDir, "eslint.config.mjs"),
          cwd: inputDir,
        });
        return timedAsync(() => eslint.lintFiles(filePaths));
      },
    },
    {
      id: "eslint-plugin-vue-workers",
      label: `eslint-plugin-vue (${Math.min(cpuCount, files.length)} workers)`,
      files: files.length,
      measure: () => timedAsync(() => runEslintWorkers(inputDir, files, eslintPath)),
    },
    {
      id: "vize-lint-1t",
      label: "Vize lint (1T)",
      files: files.length,
      measure: () =>
        runCommand(vizeBin, ["lint", ".", "--quiet"], {
          cwd: inputDir,
          allowNonZeroExit: true,
          env: { RAYON_NUM_THREADS: "1" },
        }),
    },
    {
      id: "vize-lint-max",
      label: "Vize lint (max)",
      files: files.length,
      measure: () =>
        runCommand(vizeBin, ["lint", ".", "--quiet"], {
          cwd: inputDir,
          allowNonZeroExit: true,
        }),
    },
  ];

  return createSurface({
    id: "lint",
    label: "Lint",
    files: files.length,
    bytes: totalFileBytes(inputDir, files),
    variants: await measureVariants(variants, options),
    baselineId: "eslint-plugin-vue-1t",
    vizeSingleId: "vize-lint-1t",
    vizeMaxId: "vize-lint-max",
  });
}

async function measureFormat(inputDir, files, options) {
  const prettierBin = resolveWorkspaceBin("prettier");
  const vizeBin = resolve(options.vizeBin);
  if (!existsSync(vizeBin)) {
    throw new Error(`Vize CLI not found: ${vizeBin}`);
  }

  let invocation = 0;
  const nextWorkDir = (label) => prepareFormatDir(inputDir, files, label, ++invocation);
  const variants = [
    {
      id: "prettier-cli",
      label: "Prettier CLI",
      files: files.length,
      measure: () =>
        runCommand(prettierBin, ["--write", "*.vue", "--log-level", "error"], {
          cwd: nextWorkDir("prettier"),
          allowNonZeroExit: false,
        }),
    },
    {
      id: "vize-fmt-1t",
      label: "Vize fmt (1T)",
      files: files.length,
      measure: () =>
        runCommand(vizeBin, ["fmt", "--write", "*.vue"], {
          cwd: nextWorkDir("vize-1t"),
          allowNonZeroExit: false,
          env: { RAYON_NUM_THREADS: "1" },
        }),
    },
    {
      id: "vize-fmt-max",
      label: "Vize fmt (max)",
      files: files.length,
      measure: () =>
        runCommand(vizeBin, ["fmt", "--write", "*.vue"], {
          cwd: nextWorkDir("vize-max"),
          allowNonZeroExit: false,
        }),
    },
  ];

  return createSurface({
    id: "fmt",
    label: "Format",
    files: files.length,
    bytes: totalFileBytes(inputDir, files),
    variants: await measureVariants(variants, options),
    baselineId: "prettier-cli",
    vizeSingleId: "vize-fmt-1t",
    vizeMaxId: "vize-fmt-max",
  });
}

async function measureCheck(inputDir, files, options) {
  const checkDir = prepareTypecheckDir({ inputDir, files, workRoot, copySelectedFiles });
  const vizeBin = resolve(options.vizeBin);
  if (!existsSync(vizeBin)) {
    throw new Error(`Vize CLI not found: ${vizeBin}`);
  }
  // Fail closed: without a resolvable native TypeScript engine `vize check`
  // is not measuring type checking, and a timing published from it would be
  // the heuristic-fallback result the upstream vue-benchmarks report hit.
  if (!options.backend.ready) {
    throw new Error(`Type-check surface requires a ready tsgo backend: ${options.backend.reason}`);
  }
  // Measure the backend that the artifact records, not whatever the ambient
  // resolution happens to find.
  const corsaArgs = ["--corsa-path", options.backend.corsaPath];
  const tsconfigPath = join(checkDir, "tsconfig.json");

  const variants = [
    ...createTypecheckToolVariants({
      fileCount: files.length,
      checkDir,
      tsconfigPath,
      corsaPath: options.backend.corsaPath,
      resolveWorkspaceBin,
      runCommand,
    }),
    {
      id: "vize-check-1t",
      label: "Vize check (1T)",
      files: files.length,
      measure: () =>
        runCommand(
          vizeBin,
          ["check", ".", "--quiet", "--servers", "1", "--tsconfig", tsconfigPath, ...corsaArgs],
          {
            cwd: checkDir,
            allowNonZeroExit: true,
            env: { RAYON_NUM_THREADS: "1" },
          },
        ),
    },
    {
      id: "vize-check-max",
      label: "Vize check (max)",
      files: files.length,
      measure: () =>
        runCommand(vizeBin, ["check", ".", "--quiet", "--tsconfig", tsconfigPath, ...corsaArgs], {
          cwd: checkDir,
          allowNonZeroExit: true,
        }),
    },
  ];

  return createSurface({
    id: "check",
    label: "Type check",
    files: files.length,
    bytes: totalFileBytes(inputDir, files),
    variants: await measureVariants(variants, options),
    baselineId: "vue-tsc",
    vizeSingleId: "vize-check-1t",
    vizeMaxId: "vize-check-max",
    // vue-tsc drives the JavaScript TypeScript compiler; vize check drives
    // native tsgo. Declaring the classes suppresses the cross-engine ratio.
    engineClasses: ENGINE_CLASSES_BY_SURFACE.check,
  });
}

async function measureVite(inputDir, files, options) {
  const { build } = await import("vite");
  const officialVuePlugin = (await import("@vitejs/plugin-vue")).default;
  const vizePluginPath = join(rootDir, "npm", "builder/vite", "dist", "index.mjs");
  if (!existsSync(vizePluginPath)) {
    throw new Error(
      `Vite plugin build not found: ${vizePluginPath}. Run vp run --workspace-root build:vite-plugin first.`,
    );
  }
  const vizePlugin = (await import(pathToFileURL(vizePluginPath).href)).default;

  let invocation = 0;
  const runBuild = async (label, plugins) => {
    const { workDir, entryFile } = prepareViteDir(inputDir, files, label, ++invocation);
    const outDir = join(workDir, "dist");
    return timedAsync(async () => {
      await build({
        root: workDir,
        plugins,
        build: {
          outDir,
          write: true,
          minify: false,
          rollupOptions: {
            input: entryFile,
            external: ["vue"],
          },
        },
        logLevel: "silent",
      });
    });
  };

  const variants = [
    {
      id: "vite-plugin-vue",
      label: "@vitejs/plugin-vue",
      files: files.length,
      measure: () => runBuild("official", [officialVuePlugin()]),
    },
    {
      id: "vize-vite-plugin",
      label: "@vizejs/vite-plugin",
      files: files.length,
      measure: () =>
        runBuild("vize", [
          vizePlugin({
            scanPatterns: ["*.vue"],
            precompileBatchSize: files.length,
          }),
        ]),
    },
  ];

  return createSurface({
    id: "vite",
    label: "Vite build (end-to-end)",
    files: files.length,
    bytes: totalFileBytes(inputDir, files),
    variants: await measureVariants(variants, options),
    baselineId: "vite-plugin-vue",
    vizeSingleId: null,
    vizeMaxId: "vize-vite-plugin",
  });
}

async function measureNuxt(inputDir, files, options) {
  const nuxtBin = resolveWorkspaceBin("nuxt");
  const vizeNuxtPath = join(rootDir, "npm", "framework/nuxt", "dist", "index.mjs");
  if (!existsSync(vizeNuxtPath)) {
    throw new Error(
      `Nuxt module build not found: ${vizeNuxtPath}. Run vp run --workspace-root build:nuxt-stack first.`,
    );
  }

  let invocation = 0;
  const runNuxtBuild = (label, useVize) => {
    const workDir = prepareNuxtDir(inputDir, files, label, ++invocation, useVize);
    return runCommand(nuxtBin, ["build"], {
      cwd: workDir,
      allowNonZeroExit: false,
      env: {
        CI: "1",
        NITRO_PRESET: "node-server",
        NUXT_TELEMETRY_DISABLED: "1",
      },
    });
  };

  const variants = [
    {
      id: "nuxt-default",
      label: "Nuxt default compiler",
      files: files.length,
      measure: () => runNuxtBuild("default", false),
    },
    {
      id: "vize-nuxt",
      label: "@vizejs/nuxt",
      files: files.length,
      measure: () => runNuxtBuild("vize", true),
    },
  ];

  return createSurface({
    id: "nuxt",
    label: "Nuxt SPA build (end-to-end)",
    files: files.length,
    bytes: totalFileBytes(inputDir, files),
    variants: await measureVariants(variants, options),
    baselineId: "nuxt-default",
    vizeSingleId: null,
    vizeMaxId: "vize-nuxt",
  });
}

export function renderMarkdown(data) {
  const lines = [];
  lines.push("## Tool Benchmark");
  lines.push("");

  const commit = data.commit.sha ? `\`${data.commit.sha.slice(0, 12)}\`` : "`unknown`";
  const run = data.commit.runUrl ? ` ([run](${data.commit.runUrl}))` : "";
  const runnerSpec = data.runner.blacksmithMaxSpec ? `, ${data.runner.blacksmithMaxSpec}` : "";
  lines.push(`Measured: ${data.generatedAt}`);
  lines.push(`Commit: ${commit}${run}`);
  lines.push(
    `Runner: \`${data.runner.label}\` (${data.runner.cpuCount} logical CPU, ${data.runner.cpuModel}${runnerSpec})`,
  );
  lines.push(
    `Input: ${data.input.fileCount.toLocaleString()} generated SFC files (${formatBytes(data.input.totalBytes)}). Median of ${data.settings.runs} measured run(s) after ${data.settings.warmups} warmup run(s).`,
  );
  lines.push(...renderProvenanceLines(data));
  if (data.input.largeSfcBytes > 0) {
    lines.push(
      `Large SFC: ${data.input.largeBlocks.toLocaleString()} repeated template blocks (${formatBytes(data.input.largeSfcBytes)}). Nuxt import set: ${data.input.nuxtFileCount.toLocaleString()} SFC files.`,
    );
  }
  lines.push("");
  lines.push(
    "| Surface | Files | Existing tool | Existing median | Vize 1T | Vize max | Speedup |",
  );
  lines.push("| --- | ---: | --- | ---: | ---: | ---: | ---: |");
  for (const surface of data.surfaces) {
    lines.push(renderSurfaceTable(surface, formatMs));
  }
  lines.push("");
  lines.push(...renderEngineClassSections(data.surfaces, formatMs));
  lines.push("Fairness notes:");
  for (const note of data.fairness) {
    lines.push(`- ${note}`);
  }
  lines.push("");
  lines.push("Commands:");
  lines.push("");
  lines.push("```sh");
  lines.push(data.commands.workflowDispatch);
  lines.push(data.commands.generate);
  lines.push(data.commands.benchmark);
  lines.push("```");
  lines.push("");
  lines.push("<details>");
  lines.push("<summary>Variant details and raw run times</summary>");
  lines.push("");
  for (const surface of data.surfaces) {
    lines.push(`### ${surface.label}`);
    lines.push("");
    lines.push("| Variant | Median | Throughput | Raw measured runs |");
    lines.push("| --- | ---: | ---: | --- |");
    for (const variant of surface.variants) {
      lines.push(
        `| ${variant.label} | ${formatMs(variant.medianMs)} | ${variant.throughput} | ${formatRunList(variant.runs)} |`,
      );
    }
    lines.push("");
  }
  lines.push("</details>");
  lines.push("");
  return `${lines.join("\n")}\n`;
}

export function renderDocument(data) {
  const lines = [];
  lines.push("---");
  lines.push("title: Blacksmith Benchmark Snapshot");
  lines.push("---");
  lines.push("");
  lines.push("# Blacksmith Benchmark Snapshot");
  lines.push("");
  lines.push(
    "<!-- Generated by .github/workflows/tool-benchmark.yml. Do not edit benchmark numbers by hand. -->",
  );
  lines.push("");
  lines.push(
    "This page is generated from the Tool Benchmark workflow so published performance numbers can cite one reproducible runner, input corpus, and commit.",
  );
  lines.push("");
  lines.push(
    renderMarkdown(data)
      .replace(/^## Tool Benchmark\n/, "## Latest Result\n")
      .trimEnd(),
  );
  lines.push("");
  return `${lines.join("\n")}\n`;
}

async function runBenchmarks(args) {
  const inputDir = resolve(requireArg(args, "input"));
  const runs = parsePositiveInt(args.runs, DEFAULT_RUNS);
  const warmups = parseNonNegativeInt(args.warmups, DEFAULT_WARMUPS);
  const checkFileCount = parsePositiveInt(args["check-file-count"], DEFAULT_CHECK_FILE_COUNT);
  const viteFileCount = parsePositiveInt(args["vite-file-count"], DEFAULT_VITE_FILE_COUNT);
  const nuxtFileCount = parsePositiveInt(args["nuxt-file-count"], DEFAULT_NUXT_FILE_COUNT);
  const museaFileCount = parsePositiveInt(args["musea-file-count"], DEFAULT_MUSEA_FILE_COUNT);
  const largeBlocks = parsePositiveInt(args["large-blocks"], DEFAULT_LARGE_BLOCKS);
  const taskList = selectedTasks(args.tasks);

  if (!existsSync(inputDir)) {
    throw new Error(`Input directory not found: ${inputDir}`);
  }
  if (taskList.length === 0) {
    throw new Error("No benchmark tasks selected.");
  }

  const allFiles = collectVueFiles(inputDir);
  if (allFiles.length === 0) {
    throw new Error(`No .vue files found in ${inputDir}`);
  }

  rmSync(workRoot, { recursive: true, force: true });
  mkdirSync(workRoot, { recursive: true });

  const options = {
    runs,
    warmups,
    vizeBin: args["vize-bin"] ?? join(rootDir, "target", "release", "vize"),
    checkFileCount: Math.min(checkFileCount, allFiles.length),
    viteFileCount: Math.min(viteFileCount, allFiles.length),
    nuxtFileCount: Math.min(nuxtFileCount, allFiles.length),
    // Not clamped to `allFiles`: the Musea surface generates its own `.art.vue`
    // corpus rather than reusing the plain-SFC input directory.
    museaFileCount,
    largeBlocks,
    backend: resolveBackend(),
  };
  const data = {
    ...buildMetadata({
      args,
      inputDir,
      files: allFiles,
      totalBytes: totalFileBytes(inputDir, allFiles),
      taskList,
      options,
      bins: {
        vizeBin: resolve(options.vizeBin),
        ...typecheckToolBins(optionalWorkspaceBin),
        eslintBin: optionalWorkspaceBin("eslint"),
        prettierBin: optionalWorkspaceBin("prettier"),
      },
    }),
    surfaces: [],
  };

  if (taskList.includes("compile")) {
    data.surfaces.push(await measureCompile(inputDir, allFiles, options));
  }
  if (taskList.includes("large")) {
    const largeSfc = prepareLargeSfcDir(options.largeBlocks);
    data.input.largeSfcBytes = largeSfc.bytes;
    data.surfaces.push(...(await measureLargeSfc(largeSfc, options)));
  }
  if (taskList.includes("lint")) {
    data.surfaces.push(await measureLint(inputDir, allFiles, options));
  }
  if (taskList.includes("fmt")) {
    data.surfaces.push(await measureFormat(inputDir, allFiles, options));
  }
  if (taskList.includes("check")) {
    data.surfaces.push(
      await measureCheck(inputDir, allFiles.slice(0, options.checkFileCount), options),
    );
  }
  if (taskList.includes("vite")) {
    data.surfaces.push(
      await measureVite(inputDir, allFiles.slice(0, options.viteFileCount), options),
    );
  }
  if (taskList.includes("nuxt")) {
    data.surfaces.push(
      await measureNuxt(inputDir, allFiles.slice(0, options.nuxtFileCount), options),
    );
  }
  if (taskList.includes("musea")) {
    data.surfaces.push(await measureMuseaSurface(rootDir, options));
  }

  return data;
}

export async function main(argv = process.argv.slice(2)) {
  const args = parseArgs(argv);
  const data = await runBenchmarks(args);
  const markdown = renderMarkdown(data);

  if (args.out) {
    writeFileSync(resolve(args.out), markdown);
  } else {
    process.stdout.write(markdown);
  }
  if (args.json) {
    writeFileSync(resolve(args.json), `${JSON.stringify(data, null, 2)}\n`);
  }
  if (args.doc) {
    writeFileSync(resolve(args.doc), renderDocument(data));
  }
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  try {
    await main();
  } catch (error) {
    console.error(error instanceof Error ? error.message : String(error));
    process.exit(1);
  }
}
