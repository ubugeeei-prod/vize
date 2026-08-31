/**
 * Musea build-time benchmark lane (#3464).
 *
 * `@vizejs/vite-plugin-musea` and `@vizejs/musea-nuxt` run on every build of
 * every gallery that installs them, and until this lane existed nothing in the
 * repo measured either one.
 *
 * This is a **published** lane, not an enforcing one — #3464 asked whoever
 * added it to pick one and say so. It reports numbers and, like the existing
 * Vite and Nuxt rows, does not fail a build on a regression.
 *
 * It therefore declares no baseline of its own. Enforcement in this repo has
 * exactly one mechanism, `.github/workflows/benchmark.yml`, which since #3586
 * compares a lane against a pinned historical commit on a weekly schedule as
 * well as against the PR base. A second lane carrying its own committed
 * baseline file and its own threshold would be a parallel drift gate with
 * nothing keeping the two honest, which is the failure #3586 exists to close.
 * So this lane publishes absolute per-stage cost and nothing derived from a
 * stored history; if Musea is ever to be gated, it belongs behind that
 * workflow's fixed-baseline schedule, with the JS package stack built at both
 * commits, not behind a mechanism invented here.
 *
 *   node tools/benchmarks/scripts/musea.mjs
 *   node tools/benchmarks/scripts/musea.mjs --files 480 --runs 7 --warmups 2
 *
 * Prerequisites, because the lane measures the published entry points rather
 * than the sources behind them:
 *
 *   vp run --workspace-root build:native:test
 *   vp run --workspace-root build:vite-plugin
 *   vp run --workspace-root build:nuxt-stack
 *
 * Reproducibility: the corpus is generated from `tools/benchmarks/scripts/musea-corpus.mjs` at a
 * pinned size into a fixed path under `target/`, never a `mkdtemp` root — the
 * plugin derives virtual module ids from absolute file names, so a moving root
 * changes the generated modules and defeats the output check below. Every stage
 * returns a digest of what it produced, and a run whose digests differ between
 * passes fails instead of reporting a median over two different workloads.
 */

import { spawn } from "node:child_process";
import { mkdirSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

import {
  assertMuseaArtifactsUnchanged,
  assertMuseaNativeLoaded,
  assertMuseaNativeSelection,
  resolveMuseaArtifacts,
} from "./musea-artifacts.mjs";
import {
  MUSEA_CORPUS_FILE_COUNT,
  museaCorpusVariantCount,
  writeMuseaCorpus,
} from "./musea-corpus.mjs";
import { withMuseaWorkspaceLock } from "./musea-lock.mjs";
import { createMuseaStages } from "./musea-stages.mjs";

const benchDir = dirname(fileURLToPath(import.meta.url));
const rootDir = resolve(benchDir, "..", "..", "..");
const workerPath = join(benchDir, "musea-worker.mjs");

export const DEFAULT_MUSEA_RUNS = 5;
export const DEFAULT_MUSEA_WARMUPS = 1;

/** Fixed, not `os.tmpdir()`: see the module comment. */
export function museaWorkDir(root = rootDir) {
  return join(root, "target", "musea-benchmark", "corpus");
}

function parseArgs(argv) {
  const options = {
    files: MUSEA_CORPUS_FILE_COUNT,
    runs: DEFAULT_MUSEA_RUNS,
    warmups: DEFAULT_MUSEA_WARMUPS,
  };
  for (let i = 0; i < argv.length; i += 1) {
    if (argv[i] === "--files") options.files = Number(argv[++i]);
    else if (argv[i] === "--runs") options.runs = Number(argv[++i]);
    else if (argv[i] === "--warmups") options.warmups = Number(argv[++i]);
    else throw new Error(`tools/benchmarks/scripts/musea.mjs: unknown argument ${argv[i]}`);
  }
  return options;
}

function median(values) {
  const sorted = [...values].sort((a, b) => a - b);
  const mid = Math.floor(sorted.length / 2);
  return sorted.length % 2 === 1 ? sorted[mid] : (sorted[mid - 1] + sorted[mid]) / 2;
}

async function timed(fn) {
  const started = process.hrtime.bigint();
  const output = await fn();
  return { elapsedMs: Number(process.hrtime.bigint() - started) / 1e6, output };
}

async function observableDigest(stage, output) {
  const digest = stage.observe ? await stage.observe(output) : output;
  if (typeof digest !== "string" || digest.length === 0) {
    throw new Error(
      `tools/benchmarks/scripts/musea.mjs: ${stage.id} produced no deterministic digest; every measured stage must prove equivalent output`,
    );
  }
  return digest;
}

/**
 * Measure every stage: warmups first, then `runs` measured passes whose order
 * alternates, matching `measureVariants` in tools/benchmarks/scripts/compare-tools.mjs so a stage
 * is not systematically favoured by always running while the heap is cold.
 *
 * A stage's `prepare` is re-run before every pass and is never timed.
 */
export async function runMuseaStages(stages, { runs, warmups }) {
  const digests = new Map();
  const verifyDigest = async (stage, output) => {
    const digest = await observableDigest(stage, output);
    const seen = digests.get(stage.id);
    if (seen === undefined) {
      digests.set(stage.id, digest);
    } else if (seen !== digest) {
      throw new Error(
        `tools/benchmarks/scripts/musea.mjs: ${stage.id} produced different output between passes (${seen} -> ${digest}); refusing to report a median over two workloads`,
      );
    }
  };

  for (let pass = 0; pass < warmups; pass += 1) {
    for (const stage of stages) {
      await stage.prepare();
      const output = await stage.run();
      await verifyDigest(stage, output);
    }
  }

  const samples = new Map(stages.map((stage) => [stage.id, []]));

  for (let pass = 0; pass < runs; pass += 1) {
    const ordered = pass % 2 === 0 ? stages : [...stages].reverse();
    for (const stage of ordered) {
      await stage.prepare();
      const { elapsedMs, output } = await timed(() => stage.run());
      samples.get(stage.id).push(Number(elapsedMs.toFixed(3)));
      await verifyDigest(stage, output);
    }
  }

  return stages.map((stage) => {
    const measured = samples.get(stage.id);
    const medianMs = Number(median(measured).toFixed(3));
    return {
      id: stage.id,
      label: stage.label,
      units: stage.units,
      unitLabel: stage.unitLabel,
      medianMs,
      msPerUnit: Number((medianMs / stage.units).toFixed(6)),
      runs: measured,
      digest: digests.get(stage.id) ?? null,
    };
  });
}

/**
 * Generate the corpus, measure every stage, and re-check that the artifacts
 * measured are byte-identical to the ones the run started with.
 */
export async function measureMuseaInProcess({ files, runs, warmups, root = rootDir } = {}) {
  return withMuseaWorkspaceLock(root, async () => {
    const fileCount = files ?? MUSEA_CORPUS_FILE_COUNT;
    const workDir = museaWorkDir(root);
    mkdirSync(dirname(workDir), { recursive: true });
    const artifacts = resolveMuseaArtifacts(root);
    assertMuseaNativeSelection(artifacts);
    const corpus = writeMuseaCorpus(workDir, fileCount);
    const previousNativePath = process.env.NAPI_RS_NATIVE_LIBRARY_PATH;
    const previousForceWasi = process.env.NAPI_RS_FORCE_WASI;
    Reflect.deleteProperty(process.env, "NAPI_RS_NATIVE_LIBRARY_PATH");
    Reflect.deleteProperty(process.env, "NAPI_RS_FORCE_WASI");
    try {
      const stages = createMuseaStages({ artifacts, workDir, files: corpus.files });
      const results = await runMuseaStages(stages, {
        runs: runs ?? DEFAULT_MUSEA_RUNS,
        warmups: warmups ?? DEFAULT_MUSEA_WARMUPS,
      });

      assertMuseaNativeLoaded(artifacts);
      assertMuseaArtifactsUnchanged(artifacts);
      return {
        fileCount,
        variantCount: museaCorpusVariantCount(fileCount),
        bytes: corpus.bytes,
        workDir,
        artifacts,
        stages: results,
      };
    } finally {
      if (previousNativePath === undefined) {
        Reflect.deleteProperty(process.env, "NAPI_RS_NATIVE_LIBRARY_PATH");
      } else {
        process.env.NAPI_RS_NATIVE_LIBRARY_PATH = previousNativePath;
      }
      if (previousForceWasi === undefined) {
        Reflect.deleteProperty(process.env, "NAPI_RS_FORCE_WASI");
      } else {
        process.env.NAPI_RS_FORCE_WASI = previousForceWasi;
      }
    }
  });
}

/** Run in a fresh process so no earlier benchmark can supply a cached native binding. */
export async function measureMusea(options = {}) {
  const workerOptions = { ...options, root: options.root ?? rootDir };
  return new Promise((resolveResult, reject) => {
    const workerEnv = { ...process.env };
    Reflect.deleteProperty(workerEnv, "NAPI_RS_NATIVE_LIBRARY_PATH");
    Reflect.deleteProperty(workerEnv, "NAPI_RS_FORCE_WASI");
    const subprocess = spawn(process.execPath, [workerPath, JSON.stringify(workerOptions)], {
      cwd: workerOptions.root,
      env: workerEnv,
      stdio: ["ignore", "pipe", "pipe"],
    });
    let stdout = "";
    let stderr = "";
    subprocess.stdout.on("data", (chunk) => (stdout += chunk));
    subprocess.stderr.on("data", (chunk) => (stderr += chunk));
    subprocess.once("error", reject);
    subprocess.once("exit", (code, signal) => {
      if (code !== 0) {
        reject(
          new Error(
            stderr.trim() ||
              `tools/benchmarks/scripts/musea.mjs: isolated worker exited with ${signal ?? `status ${code}`}`,
          ),
        );
        return;
      }
      try {
        resolveResult(JSON.parse(stdout));
      } catch (error) {
        reject(
          new Error(
            `tools/benchmarks/scripts/musea.mjs: isolated worker returned invalid JSON: ${error instanceof Error ? error.message : String(error)}`,
          ),
        );
      }
    });
  });
}

function formatMs(ms) {
  return ms >= 1000 ? `${(ms / 1000).toFixed(2)}s` : `${ms.toFixed(1)}ms`;
}

export function renderMuseaReport(data, { runs, warmups }) {
  const lines = [];
  lines.push("## Musea build-time benchmark");
  lines.push("");
  lines.push(
    `Corpus: ${data.fileCount.toLocaleString()} pinned \`.art.vue\` files carrying ${data.variantCount.toLocaleString()} variants (${(data.bytes / 1024).toFixed(1)} KB).`,
  );
  lines.push(`Median of ${runs} measured pass(es) after ${warmups} warmup pass(es).`);
  lines.push(
    `Work directory: \`${data.workDir}\` (fixed path, see tools/benchmarks/scripts/musea.mjs).`,
  );
  lines.push("");
  lines.push("| Stage | Units | Median | Per unit | Measured passes |");
  lines.push("| --- | ---: | ---: | ---: | --- |");
  for (const stage of data.stages) {
    lines.push(
      `| ${stage.label} | ${stage.units.toLocaleString()} ${stage.unitLabel} | ${formatMs(stage.medianMs)} | ${stage.msPerUnit.toFixed(4)}ms | ${stage.runs.map(formatMs).join(", ")} |`,
    );
  }
  lines.push("");
  lines.push("Measured artifacts:");
  for (const [label, artifact] of Object.entries(data.artifacts)) {
    lines.push(`- ${label}: \`${artifact.sha256.slice(0, 16)}\``);
  }
  lines.push("");
  lines.push(
    "This lane is published, not enforcing: it reports the plugin's own build-time cost and does not fail on a regression.",
  );
  return `${lines.join("\n")}\n`;
}

export async function main(argv = process.argv.slice(2)) {
  const options = parseArgs(argv);
  const data = await measureMusea(options);
  process.stdout.write(renderMuseaReport(data, options));
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  try {
    await main();
  } catch (error) {
    console.error(error instanceof Error ? error.message : String(error));
    process.exit(1);
  }
}
