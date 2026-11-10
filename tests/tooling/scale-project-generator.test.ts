import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import {
  existsSync,
  mkdirSync,
  mkdtempSync,
  readdirSync,
  realpathSync,
  rmSync,
  symlinkSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { performance } from "node:perf_hooks";
import { type TestContext, test } from "node:test";

import {
  type Plant,
  type ScaleKind,
  expectedPlants,
  generateScaleProject,
  minSfcs,
  monorepoMapperConfigs,
  plantCounts,
  projectShape,
} from "../../tools/benchmarks/scripts/scale-project-generator.ts";
import { autoImportName, nuxtPlace } from "../../tools/benchmarks/scripts/scale-project-nuxt.ts";
import { PLANT_KINDS } from "../../tools/benchmarks/scripts/scale-project-sfc.ts";
import { gate, parseRun } from "../../tools/benchmarks/scripts/typecheck-scaling-gate.ts";
import { root } from "./support/github-workflows.ts";

const plantLine = (files: Map<string, string>, plant: Plant) =>
  (files.get(plant.file) ?? "").split("\n")[plant.line - 1] ?? "";

const PLANT_TEXT: Record<string, RegExp> = {
  script: /^const planted\d+: number = "deep-\d+";$/u,
  "emit-script": /emit\("apply-filter", 42\)/u,
  generic: /<KitSelect :options="rows" :label-of="\(row\) => row\.score\d+" \/>/u,
  slot: /\{\{ row\.missing\d+ \}\}/u,
  "emit-template": /@blur-commit="onPlantCommit"/u,
  "v-model": /<KitField v-model="climate"/u,
  global: /<GBadge :count="'many'" \/>/u,
  "listener-arrow":
    /<KitAction action-label="Plant" @save-item="\(value: number\) => undefined" \/>/u,
  "listener-prop": /<KitAction :action-label="1" @save-item="\(value: number\) => undefined" \/>/u,
  "listener-statement": /@save-item="takesLabel\(count\)"/u,
};
const vueFiles = (files: Map<string, string>) =>
  [...files.keys()].filter((path) => path.endsWith(".vue")).length;

for (const kind of ["vue", "nuxt", "monorepo"] as ScaleKind[]) {
  void test(`a ${kind} project has exactly the asked SFCs and every plant on its own line`, () => {
    const sfcs = minSfcs(kind) * 2 + 3;
    const project = generateScaleProject({ sfcs, kind, featurePackages: 2 });
    assert.equal(vueFiles(project.files), sfcs);
    assert.equal(project.features, 2);
    assert.deepEqual(
      plantCounts(project.plants),
      Object.fromEntries(PLANT_KINDS.map((plant) => [plant, 2])),
    );
    for (const plant of project.plants) {
      assert.match(plantLine(project.files, plant), PLANT_TEXT[plant.kind] as RegExp);
      assert.ok(plant.codes.length > 0);
    }
    assert.deepEqual(generateScaleProject({ sfcs, kind, featurePackages: 2 }).files, project.files);
    assert.throws(() => generateScaleProject({ sfcs: minSfcs(kind) - 1, kind }), /at least/u);
    assert.equal(
      generateScaleProject({ sfcs, kind, featurePackages: 2, plants: false }).plants.length,
      0,
    );
  });
}

void test("the seed moves the plants and the rows, and nothing else", () => {
  const a = generateScaleProject({ sfcs: 1000, kind: "vue", seed: 0 });
  const b = generateScaleProject({ sfcs: 1000, kind: "vue", seed: 3 });
  assert.deepEqual([...a.files.keys()], [...b.files.keys()]);
  assert.notDeepEqual(a.plants, b.plants);
  assert.ok(b.plants.some((plant) => plant.file.startsWith("src/features/f0007/")));
  assert.deepEqual(generateScaleProject({ sfcs: 1000, kind: "vue", seed: 3 }).files, b.files);
  assert.throws(() => generateScaleProject({ sfcs: 1000, kind: "vue", seed: -1 }), /natural/u);
});

void test("plants scale with the project and always reach the last feature", () => {
  assert.deepEqual(projectShape(1000), { features: 49, extraWidgets: 6 });
  const project = generateScaleProject({ sfcs: 1000, kind: "vue" });
  assert.equal(project.plants.length, 6 * PLANT_KINDS.length);
  assert.ok(project.plants.some((plant) => plant.file.startsWith("src/features/f0048/")));
  assert.ok(project.files.has("src/features/f0048/Widget15.vue"));
  assert.match(
    project.files.get("src/features/f0000/Page.vue") ?? "",
    /import Page2 from "\.\.\/f0002\/Page\.vue";/u,
  );
  assert.match(
    project.files.get("src/components.d.ts") ?? "",
    /GBadge: \(typeof import\("\.\/global\/GBadge\.vue"\)\)\["default"\];/u,
  );
});

void test("the giant profile keeps all work in one oversized SFC", () => {
  const project = generateScaleProject({ sfcs: 1, kind: "giant", giantBlocks: 100 });
  assert.equal(vueFiles(project.files), 1);
  assert.equal(project.features, 100);
  assert.ok((project.files.get("src/App.vue")?.length ?? 0) > 70_000);
  assert.equal(project.plants.length, 1);
  assert.match(plantLine(project.files, project.plants[0]!), PLANT_TEXT.script);
  assert.throws(
    () => generateScaleProject({ sfcs: 2, kind: "giant", giantBlocks: 100 }),
    /exactly 1 SFC/u,
  );
});

void test("fixed model and listener plants stay required", () => {
  const { files, plants } = generateScaleProject({ sfcs: minSfcs("vue"), kind: "vue" });
  assert.match(files.get("src/features/f0000/Page.vue")!, /const climate = ref\(42\);/u);
  const vModel = (lane: string) =>
    expectedPlants(plants, lane).find((plant) => plant.kind === "v-model")?.count;
  assert.equal(vModel("vue-tsc"), 1);
  assert.equal(vModel("tsc-vize-mapper"), 1);
  assert.equal(vModel("tsrs-fast-vize-mapper"), 1);
  const listener = (lane: string) =>
    expectedPlants(plants, lane).find((plant) => plant.kind === "listener-prop")?.count;
  assert.deepEqual(
    ["vize-check", "tsc-vize-mapper", "tsrs-fast-vize-mapper", "vue-tsc"].map(listener),
    [2, 2, 2, 1],
  );
});

void test("the performance corpus omits known cross-mapper misses", () => {
  const project = generateScaleProject({
    sfcs: minSfcs("vue"),
    kind: "vue",
    parityPlants: true,
  });
  assert.ok(project.plants.length > 0);
  assert.ok(
    project.plants.every((plant) => !["listener-arrow", "listener-prop"].includes(plant.kind)),
  );
  assert.doesNotMatch(
    project.files.get("src/features/f0000/Page.vue") ?? "",
    /@save-item="\(value: number\) => undefined"/u,
  );
  const counts = ["tsc-vize-mapper", "tsc-vue-mapper", "tsrs-fast-parallel-vize-mapper"].map(
    (lane) => expectedPlants(project.plants, lane).reduce((sum, plant) => sum + plant.count, 0),
  );
  assert.equal(new Set(counts).size, 1);
});

void test("the giant profile has no reference-only attribute diagnostics", () => {
  const project = generateScaleProject({ sfcs: 1, kind: "giant", giantBlocks: 10 });
  const source = project.files.get("src/App.vue") ?? "";
  assert.doesNotMatch(source, /:data-index=/u);
  assert.match(source, /id="metric-9"/u);
});

void test("a Nuxt project is a Nuxt 4 app tree with auto-imported, path-prefixed components", () => {
  const project = generateScaleProject({ sfcs: minSfcs("nuxt") * 3, kind: "nuxt" });
  const { files } = project;
  assert.equal(project.tsconfig, ".nuxt/tsconfig.app.json");
  for (const path of [
    "app/app.vue",
    "app/layouts/default.vue",
    "app/layouts/wide.vue",
    "app/pages/index.vue",
    "app/pages/features/f0002.vue",
    "app/components/kit/Table.vue",
    "app/components/g/Badge.vue",
    "app/components/feature/f0001/Panel.vue",
    "app/composables/useFeature1.ts",
    "app/utils/f0001.ts",
    "nuxt.config.ts",
  ]) {
    assert.ok(files.has(path), path);
  }
  assert.equal(autoImportName(nuxtPlace("features/f0001/Layer00.vue")), "FeatureF0001Layer00");
  assert.equal(autoImportName(nuxtPlace("kit/KitTable.vue")), "KitTable");
  const panel = files.get("app/components/feature/f0000/Panel.vue") ?? "";
  assert.doesNotMatch(panel, /\.vue";$/mu);
  assert.match(panel, /const \{ rows, query \} = useFeature0\(\);/u);
  assert.match(panel, /<FeatureF0000Layer00 :rows="rows"/u);
  assert.match(panel, /<FeatureF0001Panel \/>/u);
  assert.match(
    files.get("app/pages/features/f0001.vue") ?? "",
    /definePageMeta\(\{ layout: "wide" \}\);/u,
  );
  assert.ok(![...files.keys()].some((path) => path.endsWith("components.d.ts")));
});

void test("the monorepo lists packages after their references and keeps mapper configs parallel", () => {
  const project = generateScaleProject({ sfcs: minSfcs("monorepo") + 5 * 20, kind: "monorepo" });
  const order = project.packages;
  assert.deepEqual(order.slice(0, 2), ["packages/kit", "packages/feat-5"]);
  assert.equal(order.at(-1), "packages/app");
  for (const [index, name] of order.entries()) {
    const config = JSON.parse(project.files.get(`${name}/tsconfig.json`) ?? "{}") as {
      references: { path: string }[];
      compilerOptions: { rootDir: string };
    };
    assert.equal(config.compilerOptions.rootDir, "src");
    for (const { path } of config.references) {
      assert.ok(order.indexOf(join(name, path)) < index, `${name} -> ${path}`);
    }
  }
  assert.deepEqual(JSON.parse(project.files.get("tsconfig.json") ?? "{}").files, []);
  const planted = new Set(project.plants.map((plant) => plant.file.split("/src/")[0]));
  assert.deepEqual([...planted].sort(), ["packages/feat-0", "packages/feat-5"]);
  const mapper = monorepoMapperConfigs(
    project,
    { id: "vize", package: "vize" },
    "tsconfig.vize-mapper.json",
  );
  const feat = JSON.parse(mapper.get("packages/feat-0/tsconfig.vize-mapper.json") ?? "{}") as {
    references: { path: string }[];
    compilerOptions: { outDir: string };
  };
  assert.equal(feat.compilerOptions.outDir, ".out-vize-mapper");
  assert.ok(feat.references.every(({ path }) => path.endsWith("/tsconfig.vize-mapper.json")));
});

void test("10k SFCs generate and write in seconds", (t: TestContext) => {
  const dir = realpathSync(mkdtempSync(join(tmpdir(), "scale-project-10k-")));
  t.after(() => rmSync(dir, { recursive: true, force: true }));
  const start = performance.now();
  const project = generateScaleProject({ sfcs: 10_000, kind: "nuxt", dir });
  const ms = performance.now() - start;
  assert.equal(vueFiles(project.files), 10_000);
  assert.ok(readdirSync(join(dir, "app", "components", "feature")).length === project.features);
  assert.ok(ms < 20_000, `10k SFCs took ${ms.toFixed(0)} ms`);
});

void test(
  "vue-tsc reports exactly the planted errors of a small project",
  {
    skip:
      !existsSync(join(root, "tools/benchmarks/scripts/node_modules/vue-tsc")) &&
      "vue-tsc is not installed",
  },
  (t: TestContext) => {
    const dir = realpathSync(mkdtempSync(join(tmpdir(), "scale-project-vue-tsc-")));
    t.after(() => rmSync(dir, { recursive: true, force: true }));
    const project = generateScaleProject({ sfcs: minSfcs("vue"), kind: "vue", dir, seed: 4 });
    const modules = join(root, "tools/benchmarks/scripts/node_modules");
    mkdirSync(join(dir, "node_modules"));
    symlinkSync(realpathSync(join(modules, "vue")), join(dir, "node_modules", "vue"));
    const vueTsc = join(realpathSync(join(modules, "vue-tsc")), "bin", "vue-tsc.js");
    const result = spawnSync(
      process.execPath,
      [vueTsc, "--noEmit", "--pretty", "false", "-p", join(dir, "tsconfig.json")],
      { cwd: dir, encoding: "utf8" },
    );
    const parsed = parseRun(
      { status: result.status, stdout: result.stdout, stderr: result.stderr },
      dir,
      "text",
    );
    const verdict = gate("vue-tsc", parsed.diagnostics, project.plants);
    assert.equal(verdict.ok, true, [...verdict.missing, ...verdict.unexpected].join("\n"));
  },
);
