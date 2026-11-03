// P4-16 measurement: the same rule over the same corpus through every arm
// the spike compared. `node tests/tooling/fixtures/davinci-plugin-sdk/bench.mjs [rounds]`
// prints µs per file; `measure()` is what the node test runs on a small
// corpus to keep the arms agreeing.
import { readdirSync, readFileSync } from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { Worker } from "node:worker_threads";

import { runProxy } from "./sdk.mjs";

const here = path.dirname(fileURLToPath(import.meta.url));
const root = path.resolve(here, "../../../..");

/** Every `.vue` file under `dirs` (repo-relative), sorted. */
export function corpus(dirs = ["examples", "playground/src", "npm/builder/vite-musea"]) {
  const files = [];
  const walk = (dir) => {
    for (const entry of readdirSync(dir, { withFileTypes: true })) {
      if (entry.name === "node_modules" || entry.name.startsWith(".")) continue;
      const full = path.join(dir, entry.name);
      if (entry.isDirectory()) walk(full);
      else if (entry.name.endsWith(".vue")) files.push(full);
    }
  };
  for (const dir of dirs) walk(path.join(root, dir));
  return files
    .sort((a, b) => (a < b ? -1 : a > b ? 1 : 0))
    .map((file) => ({ file: path.relative(root, file), source: readFileSync(file, "utf8") }));
}

function native() {
  return createRequire(import.meta.url)(path.join(root, "npm/native/index.js"));
}

const elapsed = (start) => Number(process.hrtime.bigint() - start) / 1000;

const median = (values) => {
  const sorted = [...values].sort((left, right) => left - right);
  const middle = Math.floor(sorted.length / 2);
  return sorted.length % 2 === 1 ? sorted[middle] : (sorted[middle - 1] + sorted[middle]) / 2;
};

const timed = (run) => {
  const start = process.hrtime.bigint();
  run();
  return elapsed(start);
};

/**
 * Compare the ordinary compile/lint surfaces with their explicitly empty
 * plugin hosts. Each pair consumes the same immutable corpus, alternates order
 * per round, and retains complete output parity before measuring. Timings are
 * microseconds for the whole corpus; callers own the environment-specific
 * ratio/absolute budget.
 */
export function measureDisabledPluginOverhead(
  files,
  { warmups = 2, samples = 9, binding = native() } = {},
) {
  if (files.length === 0) throw new Error("disabled-plugin measurement requires a corpus");
  if (!Number.isInteger(warmups) || warmups < 1)
    throw new Error("disabled-plugin measurement requires at least one warmup");
  if (!Number.isInteger(samples) || samples < 3 || samples % 2 === 0)
    throw new Error("disabled-plugin measurement requires an odd sample count of at least three");

  const inputs = files.map(({ file, source }) => {
    const parsed = binding.parseSfc(source, { filename: file, sourceMap: false });
    const template = parsed.template;
    if (template === null)
      throw new Error(`${file}: disabled-plugin measurement requires a template block`);
    return { file, source, template: template.content };
  });
  const compileDefault = () =>
    inputs.map(({ file, template }) => binding.compile(template, { filename: file }));
  const compileDisabled = () => {
    const outputs = inputs.map(({ file, template }) =>
      binding.compileWithOutputPlugins(template, [], { filename: file }),
    );
    return {
      results: outputs.map(({ result }) => result),
      pluginCounts: outputs.map(({ plugins }) => plugins.length),
    };
  };
  const lintDefault = () =>
    inputs.map(({ file, source }) =>
      binding.lintPatinaSfc(source, { filename: file, locale: "en", helpLevel: "none" }),
    );
  const lintDisabled = () => {
    const results = lintDefault();
    const hosts = inputs.map(({ file, source }) =>
      binding.lintWithPlugins(source, [], { filename: file }),
    );
    return {
      results,
      hosts: hosts.map(({ diagnostics, plugins, fixes, factProviders }) => ({
        diagnostics,
        plugins,
        fixes,
        factProviders,
      })),
    };
  };

  const defaultCompileOutput = compileDefault();
  const disabledCompileOutput = compileDisabled();
  const defaultLintOutput = lintDefault();
  const disabledLintOutput = lintDisabled();
  const parity = {
    compile:
      JSON.stringify(disabledCompileOutput.results) === JSON.stringify(defaultCompileOutput) &&
      disabledCompileOutput.pluginCounts.every((count) => count === 0),
    lint:
      JSON.stringify(disabledLintOutput.results) === JSON.stringify(defaultLintOutput) &&
      disabledLintOutput.hosts.every(
        ({ diagnostics, plugins, fixes, factProviders }) =>
          diagnostics.length === 0 &&
          plugins.length === 0 &&
          fixes.length === 0 &&
          factProviders.length === 0,
      ),
  };

  const paths = {
    compile: { default: compileDefault, disabled: compileDisabled },
    lint: { default: lintDefault, disabled: lintDisabled },
  };
  /**
   * @type {Record<string, {
   *   defaultSamplesUs: number[],
   *   disabledSamplesUs: number[],
   *   defaultMedianUs: number,
   *   disabledMedianUs: number,
   *   disabledToDefaultRatio: number,
   *   overheadUsPerFile: number,
   * }>}
   */
  const observations = {};
  for (const [name, modes] of Object.entries(paths)) {
    const timings = { default: [], disabled: [] };
    for (let round = 0; round < warmups + samples; round += 1) {
      const order = round % 2 === 0 ? ["default", "disabled"] : ["disabled", "default"];
      for (const mode of order) {
        const duration = timed(modes[mode]);
        if (round >= warmups) timings[mode].push(duration);
      }
    }
    const defaultMedianUs = median(timings.default);
    const disabledMedianUs = median(timings.disabled);
    observations[name] = {
      defaultSamplesUs: timings.default,
      disabledSamplesUs: timings.disabled,
      defaultMedianUs,
      disabledMedianUs,
      disabledToDefaultRatio: disabledMedianUs / defaultMedianUs,
      overheadUsPerFile: (disabledMedianUs - defaultMedianUs) / inputs.length,
    };
  }

  return {
    files: inputs.length,
    warmups,
    samples,
    parity,
    observations,
  };
}

/** Run every arm of plugin `module` (a file beside this one) over `files`
 * `rounds` times; returns µs per file and the reports per arm. */
export async function measure(files, rounds = 1, module = "team-conventions.mjs") {
  const { default: team } = await import(`./${module}`);
  const binding = native();
  const batches = [];
  const recorder = { ...team, run: (json) => (batches.push(json), "[]") };
  for (const { file, source } of files)
    binding.lintWithPlugins(source, [recorder], { filename: file });

  const arms = { batch: [], proxy: [], sync: [], worker: [] };
  const time = { build: 0, batch: 0, proxy: 0, sync: 0, worker: 0 };
  for (let round = 0; round < rounds; round += 1) {
    const keep = round === 0;
    let start = process.hrtime.bigint();
    for (const { file, source } of files) binding.openPluginDocument(source, file);
    time.build += elapsed(start);
    start = process.hrtime.bigint();
    for (const { file, source } of files) {
      const out = binding.lintWithPlugins(source, [team], { filename: file });
      if (keep) arms.batch.push(out.diagnostics.length);
    }
    time.batch += elapsed(start);
    start = process.hrtime.bigint();
    for (const { file, source } of files) {
      const reports = runProxy(team, binding.openPluginDocument(source, file));
      if (keep) arms.proxy.push(reports.map((r) => r.node).join());
    }
    time.proxy += elapsed(start);
    start = process.hrtime.bigint();
    for (const json of batches) {
      const reports = JSON.parse(team.run(json));
      if (keep) arms.sync.push(reports.map((r) => r.node).join());
    }
    time.sync += elapsed(start);
  }
  const worker = new Worker(path.join(here, "worker.mjs"), { workerData: module });
  const ask = (json) =>
    new Promise((resolve) => (worker.once("message", resolve), worker.postMessage(json)));
  await ask(batches[0] ?? "{}");
  for (let round = 0; round < rounds; round += 1) {
    const start = process.hrtime.bigint();
    for (const json of batches) {
      const reports = JSON.parse(await ask(json));
      if (round === 0) arms.worker.push(reports.map((r) => r.node).join());
    }
    time.worker += elapsed(start);
  }
  await worker.terminate();
  const perFile = Object.fromEntries(
    Object.entries(time).map(([arm, us]) => [arm, us / (rounds * Math.max(files.length, 1))]),
  );
  const bytes = batches.reduce((sum, json) => sum + json.length, 0);
  return {
    files: files.length,
    perFile,
    arms,
    batchBytesPerFile: bytes / Math.max(files.length, 1),
  };
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  const rounds = Number(process.argv[2] ?? 20);
  const files = corpus();
  for (const module of ["team-conventions.mjs", "design-system.mjs", "i18n.mjs"]) {
    const { perFile, batchBytesPerFile } = await measure(files, rounds, module);
    const arms = Object.entries(perFile).map(([arm, us]) => `${arm}=${us.toFixed(1)}`);
    console.log(
      `${module} files=${files.length} rounds=${rounds} batch-bytes/file=${batchBytesPerFile.toFixed(0)} µs/file: ${arms.join(" ")}`,
    );
  }
}
