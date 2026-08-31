/**
 * Planted-bug gates for the `vize check` benchmark (tools/benchmarks/scripts/check-gate.mjs).
 *
 * Plant shapes are the four classes pinned by
 * tests/snapshots/check/vue-benchmarks-correctness-plants.ts, derived from the
 * MIT-licensed pikax/vue-benchmarks work gate: a timing may only be published
 * when every planted diagnostic is reported in the minimal one-file projects
 * AND in a copy of the full timed corpus under the timed tsconfig.
 */

import { cpSync, existsSync, mkdirSync, rmSync, symlinkSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";

/** Minimal plant projects: id, sources, and the exact expected diagnostic. */
export const MINIMAL_PLANTS = [
  {
    id: "script",
    label: "script assignment",
    files: {
      "App.vue": `<script setup lang="ts">
const n: number = "not-a-number";
</script>

<template>
  <div>{{ n }}</div>
</template>
`,
    },
    expected: {
      file: "App.vue",
      diagnostic: "error:2:7 [TS2322] Type 'string' is not assignable to type 'number'.",
    },
  },
  {
    id: "templateProp",
    label: "native boolean prop",
    files: {
      "App.vue": `<script setup lang="ts">
const disabledFlag: string = "yes";
</script>

<template>
  <button type="button" :disabled="disabledFlag">go</button>
</template>
`,
    },
    expected: {
      file: "App.vue",
      diagnostic:
        "error:6:26 [TS2322] Type 'string' is not assignable to type 'Booleanish | undefined'.",
    },
  },
  {
    id: "templateEvent",
    label: "native event handler",
    files: {
      "App.vue": `<script setup lang="ts">
const clickHandler = 123;
</script>

<template>
  <button type="button" @click="clickHandler">go</button>
</template>
`,
    },
    expected: {
      file: "App.vue",
      diagnostic:
        "error:6:33 [TS2345] Argument of type 'number' is not assignable to parameter of type '(_e: PointerEvent) => unknown'.",
    },
  },
  {
    id: "componentProp",
    label: "static literal-union component prop",
    files: {
      "App.vue": `<script setup lang="ts">
import Child from "./Child.vue";
</script>

<template>
  <Child variant="danger" />
</template>
`,
      "Child.vue": `<script setup lang="ts">
defineProps<{
  variant: "primary" | "secondary";
}>();
</script>

<template>
  <span>{{ variant }}</span>
</template>
`,
    },
    expected: {
      file: "App.vue",
      diagnostic:
        'error:6:10 [TS2322] Type \'"danger"\' is not assignable to type \'"primary" | "secondary"\'.',
    },
  },
];

/** Corpus plant: script and template-prop failure modes in one appended file. */
export const CORPUS_PLANT_FILE = "__CheckGatePlant.vue";
const CORPUS_PLANT_SOURCE = `<script setup lang="ts">
const n: number = "not-a-number";
const disabledFlag: string = "yes";
</script>

<template>
  <button type="button" :disabled="disabledFlag">{{ n }}</button>
</template>
`;
export const CORPUS_PLANT_DIAGNOSTICS = [
  "error:2:7 [TS2322] Type 'string' is not assignable to type 'number'.",
  "error:7:26 [TS2322] Type 'string' is not assignable to type 'Booleanish | undefined'.",
];

const PLANT_TSCONFIG = {
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
  include: ["**/*.vue", "**/*.ts"],
};

/**
 * A plant whose tsconfig resolves no `vue` types certifies nothing: a checker
 * that resolves no types reports zero diagnostics and the gate would fail
 * closed for the wrong reason. Callers must pass the resolved vue package dir
 * and this helper links it into the plant project.
 */
function linkVuePackage(dir, vuePackageDir) {
  if (!vuePackageDir || !existsSync(vuePackageDir)) {
    throw new Error(
      "check-gate: cannot locate the vue package; refusing to build plant projects whose tsconfig would resolve no types",
    );
  }
  const nodeModules = join(dir, "node_modules");
  mkdirSync(nodeModules, { recursive: true });
  symlinkSync(vuePackageDir, join(nodeModules, "vue"), "dir");
  const vueNamespace = join(dirname(vuePackageDir), "@vue");
  if (existsSync(vueNamespace)) {
    symlinkSync(vueNamespace, join(nodeModules, "@vue"), "dir");
  }
}

/** Write one minimal plant project and return its directory. */
function writePlantProject(root, plant, vuePackageDir) {
  const dir = join(root, plant.id);
  mkdirSync(dir, { recursive: true });
  for (const [file, source] of Object.entries(plant.files)) {
    writeFileSync(join(dir, file), source);
  }
  writeFileSync(join(dir, "tsconfig.json"), `${JSON.stringify(PLANT_TSCONFIG, null, 2)}\n`);
  writeFileSync(
    join(dir, "package.json"),
    `${JSON.stringify({ name: `check-gate-${plant.id}`, private: true, type: "module" }, null, 2)}\n`,
  );
  linkVuePackage(dir, vuePackageDir);
  return dir;
}

/** Prepare all minimal plant projects under workRoot. */
export function prepareMinimalPlants(workRoot, vuePackageDir) {
  const root = join(workRoot, "check-gate-plants");
  rmSync(root, { recursive: true, force: true });
  mkdirSync(root, { recursive: true });
  const dirs = {};
  for (const plant of MINIMAL_PLANTS) {
    dirs[plant.id] = writePlantProject(root, plant, vuePackageDir);
  }
  return { root, dirs, cleanup: () => rmSync(root, { recursive: true, force: true }) };
}

/**
 * Copy the timed corpus, append the corpus plant, and extend the timed
 * tsconfig with it, so scale-degradation cannot hide behind a passing
 * one-file gate.
 */
export function prepareCorpusPlant(checkDir, tsconfig) {
  const dir = `${checkDir}-gate-plant`;
  rmSync(dir, { recursive: true, force: true });
  // Copy everything, including node_modules symlinks: a plant copy that drops
  // the corpus's vue resolution stops checking templates and would fail the
  // gate for the wrong reason (verified: the Booleanish half of the plant
  // disappears while the script half survives via runtime stubs).
  cpSync(checkDir, dir, { recursive: true, verbatimSymlinks: true });
  writeFileSync(join(dir, CORPUS_PLANT_FILE), CORPUS_PLANT_SOURCE);
  const planted = {
    ...tsconfig,
    include: [...tsconfig.include, CORPUS_PLANT_FILE],
  };
  writeFileSync(join(dir, "tsconfig.json"), `${JSON.stringify(planted, null, 2)}\n`);
  return { dir, cleanup: () => rmSync(dir, { recursive: true, force: true }) };
}

/**
 * Judge one vize `--format json` report against a plant expectation. The gate
 * is exact: the planted file must carry exactly the expected diagnostics and
 * nothing else in the project may be dirty, so an unrelated project-level
 * failure cannot impersonate a caught plant.
 */
export function vizeReportCatchesPlant(report, file, diagnostics) {
  if (!report || !Array.isArray(report.files)) return false;
  const planted = report.files.find((entry) => entry.file === file);
  if (!planted) return false;
  if (JSON.stringify(planted.diagnostics) !== JSON.stringify(diagnostics)) return false;
  const strayCount = report.files
    .filter((entry) => entry.file !== file)
    .reduce((count, entry) => count + entry.diagnostics.length, 0);
  return strayCount === 0 && report.errorCount === diagnostics.length;
}

/**
 * Judge vue-tsc plain output for a plant: non-zero exit, the planted file is
 * named, and a tsc-shaped error line carries the expected TS code.
 */
export function vueTscOutputCatchesPlant(status, output, file, tsCode) {
  if (status === 0 || status == null) return false;
  if (!output.includes(file)) return false;
  const pattern = new RegExp(`^\\S[^\\n]*\\(\\d+,\\d+\\): error ${tsCode}:`, "m");
  return pattern.test(output);
}

/** Count tsc-plain diagnostic lines (`file(line,col): error/warning ...`). */
export function countVueTscDiagnostics(output) {
  let count = 0;
  for (const line of output.split("\n")) {
    if (/^\S[^\n]*\(\d+,\d+\):[ \t]*(?:error|warning)\b/i.test(line)) count += 1;
  }
  return count;
}

/**
 * The corpus is measured as generated, so its own diagnostics are a baseline,
 * not a failure. The planted run must reproduce every baseline file entry
 * byte-for-byte and add exactly the planted diagnostics on the planted file —
 * anything else (dropped baseline diagnostics, extra noise, a missed plant)
 * fails the gate.
 */
export function vizeCorpusPlantMatches(baselineReport, plantedReport) {
  if (!baselineReport || !plantedReport || !Array.isArray(plantedReport.files)) return false;
  const planted = plantedReport.files.find((entry) => entry.file === CORPUS_PLANT_FILE);
  if (!planted) return false;
  if (JSON.stringify(planted.diagnostics) !== JSON.stringify(CORPUS_PLANT_DIAGNOSTICS)) {
    return false;
  }
  const others = plantedReport.files.filter((entry) => entry.file !== CORPUS_PLANT_FILE);
  if (JSON.stringify(others) !== JSON.stringify(baselineReport.files)) return false;
  return plantedReport.errorCount === baselineReport.errorCount + CORPUS_PLANT_DIAGNOSTICS.length;
}

/**
 * Run every plant gate for vize via the caller's runner. Throws on the first
 * miss (fail closed): no timing may be published past a missed plant.
 */
export function gateVize(runVize, plantDirs, corpusPlantDir, corpusBaselineReport) {
  const readiness = {};
  for (const plant of MINIMAL_PLANTS) {
    const run = runVize(plantDirs[plant.id]);
    const caught =
      run.status === 1 &&
      vizeReportCatchesPlant(run.report, plant.expected.file, [plant.expected.diagnostic]);
    readiness[plant.id] = caught;
    if (!caught) {
      throw new Error(
        `check-gate: vize missed the ${plant.label} plant (status=${run.status}); refusing to publish a timing.\n${run.stdout}\n${run.stderr}`,
      );
    }
  }
  const corpusRun = runVize(corpusPlantDir);
  readiness.corpus =
    corpusRun.status === 1 && vizeCorpusPlantMatches(corpusBaselineReport, corpusRun.report);
  if (!readiness.corpus) {
    throw new Error(
      `check-gate: vize missed the corpus-scale plant (status=${corpusRun.status}); refusing to publish a timing.\n${corpusRun.stdout}\n${corpusRun.stderr}`,
    );
  }
  return readiness;
}

/** Run the plant gates for vue-tsc; a miss unranks the row instead of exiting. */
export function gateVueTsc(runVueTsc, plantDirs, corpusPlantDir, corpusBaselineCount) {
  const expectedCodes = {
    script: "TS2322",
    templateProp: "TS2322",
    templateEvent: "TS2345",
    componentProp: "TS2322",
  };
  const readiness = {};
  for (const plant of MINIMAL_PLANTS) {
    const run = runVueTsc(plantDirs[plant.id]);
    readiness[plant.id] = vueTscOutputCatchesPlant(
      run.status,
      run.stdout,
      plant.expected.file,
      expectedCodes[plant.id],
    );
  }
  const corpusRun = runVueTsc(corpusPlantDir);
  readiness.corpus =
    vueTscOutputCatchesPlant(corpusRun.status, corpusRun.stdout, CORPUS_PLANT_FILE, "TS2322") &&
    countVueTscDiagnostics(corpusRun.stdout) ===
      corpusBaselineCount + CORPUS_PLANT_DIAGNOSTICS.length;
  return { readiness, ok: Object.values(readiness).every(Boolean) };
}
