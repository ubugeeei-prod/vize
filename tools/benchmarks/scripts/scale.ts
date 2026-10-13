#!/usr/bin/env node
/**
 * Large-scale Vue build benchmark: `@vizejs/vite-plugin` vs `@vitejs/plugin-vue`.
 *
 * Methodology is taken from `rolldown/benchmarks` (commit-independent; read
 * `bench.mjs`, `scripts/output-size.mjs`, and `apps/1000/package.json` there).
 * What was adopted, and what was not:
 *
 * ADOPTED
 * - Four scales, not one. The reference runs `apps/{1000,3000,5000,10000}`
 *   (2,413 / 5,714 / 9,014 / 19,014 modules). A single scale cannot show
 *   non-linear cost; four can, which is the whole point of this harness.
 * - Module count reported next to the time, and the per-module cost derived
 *   from the pair. A cost per module that climbs with N is the signal.
 * - Identical minimal production config for every tool: minification on,
 *   sourcemaps on, gzip size reporting off (`reportCompressedSize: false`).
 *   See `tools/benchmarks/scripts/scale/configs.mjs` — the two configs differ only in the plugin.
 * - Output size reported as JS / CSS / sourcemap buckets, walked out of the
 *   dist directory, so a tool that is fast because it emitted less is caught in
 *   the same table as the timing.
 * - Warmup then repeated runs, reported with spread. `hyperfine` is not in the
 *   nix devshell, so `tools/benchmarks/scripts/scale/measure.mjs` implements warmup + N runs +
 *   median/min/max itself, each run in a fresh process.
 *
 * NOT ADOPTED
 * - React JSX components: replaced by Vue SFCs (`<script setup lang="ts">`,
 *   scoped styles, CSS Modules, real directives). Vize compiles SFCs.
 * - Real npm dependencies for the third-party half of the graph: replaced by a
 *   generated local `node_modules/@tools/ui` package. Reasoning in
 *   `tools/benchmarks/scripts/scale/corpus.mjs`.
 * - The seven-bundler matrix: the comparison that matters here is Vize's plugin
 *   against the official Vue plugin on the same Vite, since that isolates the
 *   plugin. Bundler-vs-bundler is the reference's own question.
 * - `rome` and `three10x`: no Vue surface.
 *
 * WHY OUTPUT EQUIVALENCE IS CHECKED, NOT JUST TIME
 * A faster build that emits fewer modules, less CSS, or no sourcemaps is a
 * correctness regression wearing a performance win's clothes. `--strict` turns
 * a divergence into a non-zero exit.
 *
 * Usage:
 *   node tools/benchmarks/scripts/scale.mjs                       # 1000,3000,5000,10000, 3 runs
 *   node tools/benchmarks/scripts/scale.mjs --scales 1000 --runs 5
 *   node tools/benchmarks/scripts/scale.mjs --scales 10000 --cold
 *   node tools/benchmarks/scripts/scale.mjs --generate-only --scales 10000
 *   node tools/benchmarks/scripts/scale.mjs --json tools/benchmarks/results/scale.json
 */

import { existsSync, mkdirSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { parseArgs } from "node:util";

import { generateApp } from "./scale/corpus.mjs";
import { TOOLS, configPath, writeConfigs } from "./scale/configs.mjs";
import { collectOutput, measureTool } from "./scale/measure.mjs";
import {
  formatBytes,
  formatMs,
  printScaleTable,
  printVerification,
  reportDivergence,
} from "./scale/report.mjs";
import { sampleTokens, verifyScopedStyles, verifySourcemaps } from "./scale/verify.mjs";

const __dirname = dirname(fileURLToPath(import.meta.url));
const repoRoot = resolve(__dirname, "..", "..", "..");
const APPS_ROOT = join(__dirname, "__scale__");
const VIZE_PLUGIN_ENTRY = join(repoRoot, "npm", "builder", "vite", "dist", "index.mjs");

const { values } = parseArgs({
  options: {
    scales: { type: "string", default: "1000,3000,5000,10000" },
    runs: { type: "string", default: "3" },
    tools: { type: "string", default: TOOLS.join(",") },
    cold: { type: "boolean", default: false },
    "generate-only": { type: "boolean", default: false },
    strict: { type: "boolean", default: false },
    json: { type: "string" },
    help: { type: "boolean", short: "h", default: false },
  },
});

if (values.help) {
  console.log(
    [
      "node tools/benchmarks/scripts/scale.mjs [options]",
      "",
      "  --scales 1000,3000,5000,10000  component counts to generate and build",
      "  --runs 3                       timed runs per tool (after one warmup)",
      "  --tools vize,vue               subset of tools to measure",
      "  --cold                         drop persistent caches before every run",
      "  --generate-only                generate the apps, do not build",
      "  --strict                       exit non-zero on an output divergence",
      "  --json FILE                    write the full result set as JSON",
    ].join("\n"),
  );
  process.exit(0);
}

const scales = values.scales
  .split(",")
  .map((value) => Number.parseInt(value.trim(), 10))
  .filter((value) => Number.isFinite(value) && value > 0);
const runs = Math.max(1, Number.parseInt(values.runs, 10) || 3);
const tools = values.tools
  .split(",")
  .map((tool) => tool.trim())
  .filter(Boolean);

for (const tool of tools) {
  if (!TOOLS.includes(tool)) {
    console.error(`unknown tool: ${tool} (known: ${TOOLS.join(", ")})`);
    process.exit(2);
  }
}

if (!values["generate-only"] && tools.includes("vize") && !existsSync(VIZE_PLUGIN_ENTRY)) {
  console.error(
    `@vizejs/vite-plugin is not built at ${VIZE_PLUGIN_ENTRY}\nRun: vp run --workspace-root build:vite-plugin`,
  );
  process.exit(2);
}

mkdirSync(APPS_ROOT, { recursive: true });

const results = [];

for (const componentCount of scales) {
  const appDir = join(APPS_ROOT, String(componentCount));
  process.stderr.write(`generating ${componentCount} components in ${appDir} ...\n`);
  const app = generateApp({ appDir, componentCount });
  writeConfigs(appDir, VIZE_PLUGIN_ENTRY);
  process.stderr.write(`  ${app.componentCount} SFCs + ${app.vendorModuleCount} vendor modules\n`);

  if (values["generate-only"]) {
    continue;
  }

  const perTool = {};
  for (const tool of tools) {
    process.stderr.write(
      `  building with ${tool} (${runs} runs${values.cold ? ", cold" : ""}) ...\n`,
    );
    const timing = await measureTool({
      appDir,
      configPath: configPath(appDir, tool),
      runs,
      cold: values.cold,
      cacheDirs: [join(appDir, "node_modules", ".vize"), join(appDir, "node_modules", ".vite")],
    });
    const output = collectOutput(appDir, tool);
    const sourcemaps = verifySourcemaps(appDir, tool, sampleTokens(componentCount));
    const scopedStyles = verifyScopedStyles(appDir, tool);
    perTool[tool] = { timing, output, sourcemaps, scopedStyles };
    process.stderr.write(
      `    ${formatMs(timing.wallMedianMs)} median, ${output.moduleCount} modules, ` +
        `js ${formatBytes(output.jsBytes)}, css ${formatBytes(output.cssBytes)}, ` +
        `maps ${formatBytes(output.mapBytes)}\n`,
    );
  }

  results.push({ componentCount, vendorModuleCount: app.vendorModuleCount, tools: perTool });
}

if (values["generate-only"]) {
  process.exit(0);
}

printScaleTable(results, tools);
const verificationFailures = printVerification(results, tools);
const divergences = [...reportDivergence(results), ...verificationFailures];

if (values.json) {
  mkdirSync(dirname(values.json), { recursive: true });
  writeFileSync(
    values.json,
    `${JSON.stringify(
      {
        generatedAt: new Date().toISOString(),
        runs,
        cold: values.cold,
        results: results.map((result) => ({
          componentCount: result.componentCount,
          vendorModuleCount: result.vendorModuleCount,
          tools: Object.fromEntries(
            Object.entries(result.tools).map(([tool, data]) => [
              tool,
              {
                wallMedianMs: data.timing.wallMedianMs,
                wallMinMs: data.timing.wallMinMs,
                wallMaxMs: data.timing.wallMaxMs,
                samplesMs: data.timing.samplesMs,
                moduleCount: data.output.moduleCount,
                jsBytes: data.output.jsBytes,
                cssBytes: data.output.cssBytes,
                mapBytes: data.output.mapBytes,
                fileCount: data.output.fileCount,
                sourcemaps: data.sourcemaps,
                scopedStyles: {
                  jsScopeIdCount: data.scopedStyles.jsScopeIdCount,
                  cssScopeIdCount: data.scopedStyles.cssScopeIdCount,
                  jsOnly: data.scopedStyles.jsOnly,
                  cssOnly: data.scopedStyles.cssOnly,
                },
              },
            ]),
          ),
        })),
        divergences,
      },
      null,
      2,
    )}\n`,
  );
}

if (values.strict && divergences.length > 0) {
  process.exit(1);
}
