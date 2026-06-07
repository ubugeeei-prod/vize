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
  symlinkSync,
  writeFileSync,
} from "node:fs";
import { basename, delimiter, dirname, join, parse, relative, resolve, sep } from "node:path";
import { performance } from "node:perf_hooks";
import { fileURLToPath, pathToFileURL } from "node:url";
import { Worker } from "node:worker_threads";

const require = createRequire(import.meta.url);
const benchDir = dirname(fileURLToPath(import.meta.url));
const rootDir = dirname(benchDir);
const workRoot = join(rootDir, "target", "tool-benchmark");
const cpuCount = os.cpus().length;

const DEFAULT_RUNS = 5;
const DEFAULT_WARMUPS = 1;
const DEFAULT_CHECK_FILE_COUNT = 500;
const DEFAULT_VITE_FILE_COUNT = 1000;
const DEFAULT_NUXT_FILE_COUNT = 500;
const DEFAULT_LARGE_BLOCKS = 900;
const DEFAULT_TASKS = ["compile", "large", "lint", "fmt", "check", "vite", "nuxt"];
const BLACKSMITH_MAX_LABEL = "blacksmith-32vcpu-ubuntu-2404";
const BLACKSMITH_MAX_SPEC = "32 vCPU / 128 GB RAM / 1.5 TB storage";

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

function formatSpeedup(value) {
  if (!Number.isFinite(value)) {
    return "n/a";
  }
  return `${value.toFixed(1)}x`;
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
    join(rootDir, "npm", "nuxt", "node_modules", ".bin", name),
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

function prepareCheckDir(inputDir, files) {
  const outputDir = join(workRoot, `check-${files.length}`);
  copySelectedFiles(inputDir, outputDir, files, ["vize.config.json"]);
  writeFileSync(
    join(outputDir, "tsconfig.json"),
    `${JSON.stringify(
      {
        extends: relative(outputDir, join(inputDir, "tsconfig.json")).split(sep).join("/"),
        include: files,
      },
      null,
      2,
    )}\n`,
  );
  return outputDir;
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

function createLargeSfcSource(blockCount) {
  const blocks = [];
  for (let i = 0; i < blockCount; i++) {
    const metricIndex = i % 64;
    blocks.push(`    <article class="metric-card metric-card-${i}" :class="{ active: selectedId === ${metricIndex} }" :data-index="${i}">
      <header>
        <p>{{ labels[${metricIndex}] }}</p>
        <h2>{{ formatMetric(metrics[${metricIndex}], ${i}) }}</h2>
      </header>
      <dl>
        <div>
          <dt>Score</dt>
          <dd>{{ metrics[${metricIndex}].score }}</dd>
        </div>
        <div>
          <dt>Status</dt>
          <dd>{{ metrics[${metricIndex}].active ? "active" : "idle" }}</dd>
        </div>
      </dl>
      <ul>
        <li v-for="point in metrics[${metricIndex}].points" :key="'${i}-' + point.id">
          <span>{{ point.label }}</span>
          <strong>{{ point.value + ${i} }}</strong>
        </li>
      </ul>
      <button type="button" @click="selectMetric(${metricIndex})">Select {{ labels[${metricIndex}] }}</button>
    </article>`);
  }

  return `<template>
  <main class="large-dashboard">
    <section class="summary">
      <h1>{{ title }}</h1>
      <p>{{ activeCount }} active metrics across {{ metrics.length }} tracked rows.</p>
    </section>
${blocks.join("\n")}
  </main>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'

type Point = {
  id: string
  label: string
  value: number
}

type Metric = {
  id: number
  title: string
  score: number
  active: boolean
  points: Point[]
}

const title = ref('Large synthetic dashboard')
const selectedId = ref(0)
const metrics = ref<Metric[]>(Array.from({ length: 64 }, (_, index) => ({
  id: index,
  title: 'Metric ' + index,
  score: (index * 13) % 100,
  active: index % 3 === 0,
  points: Array.from({ length: 4 }, (__, pointIndex) => ({
    id: index + '-' + pointIndex,
    label: 'Point ' + pointIndex,
    value: index * pointIndex,
  })),
})))

const labels = computed(() => metrics.value.map((metric) => metric.title + ' / ' + metric.score))
const activeCount = computed(() => metrics.value.filter((metric) => metric.active).length)

function formatMetric(metric: Metric, offset: number): string {
  return metric.title + ' #' + offset + ' (' + metric.score + ')'
}

function selectMetric(index: number): void {
  selectedId.value = index
}
</script>

<style scoped>
.large-dashboard {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(240px, 1fr));
  gap: 12px;
}
.summary {
  grid-column: 1 / -1;
}
.metric-card {
  border: 1px solid #d4d4d8;
  padding: 12px;
}
.metric-card.active {
  border-color: #2563eb;
}
</style>
`;
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
  const nuxtNodeModules = join(rootDir, "npm", "nuxt", "node_modules");
  symlinkSync(
    existsSync(nuxtNodeModules) ? nuxtNodeModules : join(benchDir, "node_modules"),
    join(outputDir, "node_modules"),
    "dir",
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

  const vizeModuleUrl = pathToFileURL(join(rootDir, "npm", "nuxt", "dist", "index.mjs")).href;
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
    const runs = runsById.get(variant.id);
    const medianMs = median(runs);
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
  const nativePath = join(rootDir, "npm", "vize-native");
  try {
    return require(nativePath);
  } catch (error) {
    throw new Error(
      `Could not load @vizejs/native from ${nativePath}. Run vp run --workspace-root build:native first.\n${error instanceof Error ? error.message : String(error)}`,
    );
  }
}

function assertNativeBatchResult(result, expectedFiles) {
  if (!result || typeof result !== "object") {
    throw new Error("Vize native batch compile returned an invalid result.");
  }
  if (result.failed !== 0) {
    throw new Error(`Vize native batch compile failed for ${result.failed} file(s).`);
  }
  if (result.success !== expectedFiles) {
    throw new Error(
      `Vize native batch compiled ${result.success} files, expected ${expectedFiles}.`,
    );
  }
  if (!Number.isFinite(result.timeMs) || result.timeMs <= 0) {
    throw new Error(`Vize native batch returned an invalid time: ${result.timeMs}.`);
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

function measureNativeBatchCompile(native, pattern, expectedFiles) {
  const result = native.compileSfcBatch(pattern);
  assertNativeBatchResult(result, expectedFiles);
  return result.timeMs;
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
  const checkDir = prepareCheckDir(inputDir, files);
  const vueTscBin = resolveWorkspaceBin("vue-tsc");
  const vizeBin = resolve(options.vizeBin);
  if (!existsSync(vizeBin)) {
    throw new Error(`Vize CLI not found: ${vizeBin}`);
  }
  const tsconfigPath = join(checkDir, "tsconfig.json");

  const variants = [
    {
      id: "vue-tsc",
      label: "vue-tsc",
      files: files.length,
      measure: () =>
        runCommand(vueTscBin, ["--noEmit", "-p", tsconfigPath], {
          cwd: checkDir,
          allowNonZeroExit: true,
        }),
    },
    {
      id: "vize-check-1t",
      label: "Vize check (1T)",
      files: files.length,
      measure: () =>
        runCommand(
          vizeBin,
          ["check", ".", "--quiet", "--servers", "1", "--tsconfig", tsconfigPath],
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
        runCommand(vizeBin, ["check", ".", "--quiet", "--tsconfig", tsconfigPath], {
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
  });
}

async function measureVite(inputDir, files, options) {
  const { build } = await import("vite");
  const officialVuePlugin = (await import("@vitejs/plugin-vue")).default;
  const vizePluginPath = join(rootDir, "npm", "vite-plugin-vize", "dist", "index.mjs");
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
  const vizeNuxtPath = join(rootDir, "npm", "nuxt", "dist", "index.mjs");
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

function getVariant(surface, id) {
  if (!id) {
    return null;
  }
  return surface.variants.find((variant) => variant.id === id) ?? null;
}

export function createSurface(surface) {
  const baseline = surface.variants.find((variant) => variant.id === surface.baselineId);
  const vizeMax = surface.variants.find((variant) => variant.id === surface.vizeMaxId);
  const speedup =
    baseline && vizeMax && vizeMax.medianMs > 0 ? baseline.medianMs / vizeMax.medianMs : Number.NaN;

  return {
    ...surface,
    primarySpeedup: speedup,
  };
}

function githubRunUrl() {
  const server = process.env.GITHUB_SERVER_URL;
  const repo = process.env.GITHUB_REPOSITORY;
  const runId = process.env.GITHUB_RUN_ID;
  if (!server || !repo || !runId) {
    return "";
  }
  return `${server}/${repo}/actions/runs/${runId}`;
}

function buildCommands(inputFileCount, options) {
  const workflowFlags = [
    `-f file_count=${inputFileCount}`,
    `-f check_file_count=${options.checkFileCount}`,
    `-f vite_file_count=${options.viteFileCount}`,
    `-f nuxt_file_count=${options.nuxtFileCount}`,
    `-f large_blocks=${options.largeBlocks}`,
    `-f runs=${options.runs}`,
    `-f warmups=${options.warmups}`,
    "-f commit_results=true",
  ];
  const compareFlags = [
    "--input bench/__in__",
    "--vize-bin target/release/vize",
    `--runs ${options.runs}`,
    `--warmups ${options.warmups}`,
    `--check-file-count ${options.checkFileCount}`,
    `--vite-file-count ${options.viteFileCount}`,
    `--nuxt-file-count ${options.nuxtFileCount}`,
    `--large-blocks ${options.largeBlocks}`,
    `--runner-label "${BLACKSMITH_MAX_LABEL}"`,
    "--out tool-benchmark-summary.md",
    "--json tool-benchmark-results.json",
    "--doc performance-blacksmith.md",
  ];

  return {
    workflowDispatch: `gh workflow run tool-benchmark.yml --ref <branch> ${workflowFlags.join(" ")}`,
    generate: `node bench/generate.mjs ${inputFileCount}`,
    benchmark: `node bench/compare-tools.mjs ${compareFlags.join(" ")}`,
  };
}

function buildMetadata(args, inputDir, files, taskList, options) {
  const runnerLabel = args["runner-label"] ?? process.env.VIZE_BENCH_RUNNER ?? "local";
  const cpuModel = os.cpus()[0]?.model ?? "unknown";
  return {
    schemaVersion: 1,
    kind: "tool-comparison",
    generatedAt: new Date().toISOString(),
    commit: {
      sha: args.commit ?? process.env.GITHUB_SHA ?? "",
      ref: args.ref ?? process.env.GITHUB_REF_NAME ?? "",
      repository: args.repository ?? process.env.GITHUB_REPOSITORY ?? "",
      runUrl: args["run-url"] ?? githubRunUrl(),
    },
    runner: {
      label: runnerLabel,
      blacksmithMaxSpec: runnerLabel === BLACKSMITH_MAX_LABEL ? BLACKSMITH_MAX_SPEC : "",
      cpuCount,
      cpuModel,
      platform: process.platform,
      arch: process.arch,
      osRelease: os.release(),
      node: process.version,
    },
    input: {
      dir: inputDir,
      fileCount: files.length,
      totalBytes: totalFileBytes(inputDir, files),
      checkFileCount: options.checkFileCount,
      viteFileCount: options.viteFileCount,
      nuxtFileCount: options.nuxtFileCount,
      largeBlocks: options.largeBlocks,
      largeSfcBytes: 0,
    },
    settings: {
      runs: options.runs,
      warmups: options.warmups,
      tasks: taskList,
    },
    commands: buildCommands(files.length, options),
    fairness: [
      "All tools run on the same generated Vue SFC corpus from the same checkout and lockfile.",
      "The 15,000-SFC rows are the many-file workload; the large-SFC rows isolate one large component.",
      "Reported times are medians; measured runs alternate variant order after warmup runs.",
      "Destructive formatter runs receive a fresh copy of the same input before each invocation.",
      "SFC compile Vize max uses `compileSfcBatchWithResults` wall time so the primary number includes generated output crossing the JS/native boundary; the stats-only native `timeMs` is shown only in variant details.",
      "Vite build timings exclude fixture copy/setup; the Vize max lane sets `precompileBatchSize` to the benchmark file count so Blacksmith max runs one native precompile batch instead of the memory-safe default chunks.",
      "Nuxt SPA build timings exclude synthetic app generation and compare `nuxt build` with Nuxt's default compiler against the same app with `@vizejs/nuxt` installed.",
      "Single-thread lanes are shown where useful, and the primary speedup compares the incumbent default/single-thread lane with Vize's max runner lane.",
    ],
  };
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
    const baseline = getVariant(surface, surface.baselineId);
    const vizeSingle = getVariant(surface, surface.vizeSingleId);
    const vizeMax = getVariant(surface, surface.vizeMaxId);
    lines.push(
      `| ${surface.label} | ${surface.files.toLocaleString()} | ${baseline?.label ?? "n/a"} | ${formatMs(baseline?.medianMs)} | ${vizeSingle ? formatMs(vizeSingle.medianMs) : "n/a"} | ${formatMs(vizeMax?.medianMs)} | ${formatSpeedup(surface.primarySpeedup)} |`,
    );
  }
  lines.push("");
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
    largeBlocks,
  };
  const data = {
    ...buildMetadata(args, inputDir, allFiles, taskList, options),
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
