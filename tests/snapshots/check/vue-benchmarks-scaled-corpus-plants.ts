import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";

import { repoRoot } from "../../_helpers/realworld-patch.ts";
import {
  type CommandResult,
  omitProgramEvidence,
  resolveTsgoBinary,
  resolveVueTscBinary,
  runVizeCheck,
  runVueTsc,
  symlinkVueTypes,
  type VizeCheckResult,
} from "../../_helpers/realworld-typecheck.ts";

// Scaled companion to vue-benchmarks-correctness-plants.ts (#3283): the
// minimal suite proves each plant class in a 1-2 file project, this suite
// proves the same four classes are still caught inside a full unique-content
// corpus checked under one timed-style tsconfig, so `vize check` cannot
// degrade, sample, or bail at scale without failing an exact assertion.
// Gate classes and confirmation methodology from the MIT-licensed
// pikax/vue-benchmarks; see vue-benchmarks-correctness-plants.ts.
const upstream = {
  commit: "02c0dac76075924bfb8e9b6985e51a9795ff6909",
  license: "MIT",
  repository: "pikax/vue-benchmarks",
  url: "https://github.com/pikax/vue-benchmarks",
};

// One plant per upstream gate class, spread across the corpus (first file,
// one-third, two-thirds, last file) so a checker that truncates or samples the
// file list drops at least one planted diagnostic.
const corpusFileCount = 120;
const plantIndexes = [0, 45, 90, 119];

const tsconfig = {
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

type ScaledPlant = {
  broken: string;
  clean: string;
  diagnostics: string[];
  file: string;
  id: string;
  vueTscOutput: string;
};

// Anchors, diagnostics, and vue-tsc outputs are pinned from probe runs against
// the generated corpus below; positions differ from the minimal suite because
// the corpus shapes carry more script lines.
const plants: ScaledPlant[] = [
  {
    id: "script assignment",
    file: componentName(plantIndexes[0]),
    clean: 'const label0000: string = "counter 0000";',
    broken: "const label0000: string = 12345;",
    diagnostics: ["error:4:7 [TS2322] Type 'number' is not assignable to type 'string'."],
    vueTscOutput:
      "Component0000.vue(4,7): error TS2322: Type 'number' is not assignable to type 'string'.\n",
  },
  {
    id: "native boolean prop",
    file: componentName(plantIndexes[1]),
    clean: "const disabled0045: boolean = false;",
    broken: 'const disabled0045: string = "yes";',
    diagnostics: [
      "error:7:26 [TS2322] Type 'string' is not assignable to type 'Booleanish | undefined'.",
    ],
    vueTscOutput:
      "Component0045.vue(7,26): error TS2322: Type 'string' is not assignable to type 'Booleanish | undefined'.\n",
  },
  {
    id: "native event handler",
    file: componentName(plantIndexes[2]),
    clean: "const handler0090 = () => {",
    broken: "const handler0090 = 123; const spare0090 = () => {",
    diagnostics: [
      "error:8:33 [TS2345] Argument of type 'number' is not assignable to parameter of type '(_e: PointerEvent) => unknown'.",
    ],
    vueTscOutput:
      "Component0090.vue(8,4): error TS2345: Argument of type '{ type: \"button\"; onClick: 123; }' is not assignable to parameter of type 'ButtonHTMLAttributes & ReservedProps'.\n  Type '{ type: \"button\"; onClick: 123; }' is not assignable to type 'ButtonHTMLAttributes'.\n    Types of property 'onClick' are incompatible.\n      Type 'number' is not assignable to type '(payload: PointerEvent) => void'.\n",
  },
  {
    id: "static literal-union component prop",
    file: componentName(plantIndexes[3]),
    clean: '  <Child variant="primary" />',
    broken: '  <Child variant="danger" />',
    diagnostics: [
      'error:7:10 [TS2322] Type \'"danger"\' is not assignable to type \'"primary" | "secondary"\'.',
    ],
    vueTscOutput:
      'Component0119.vue(7,10): error TS2322: Type \'"danger"\' is not assignable to type \'"primary" | "secondary"\'.\n',
  },
];

function componentName(index: number): string {
  return `Component${String(index).padStart(4, "0")}.vue`;
}

// Every generated body embeds its index in identifiers and string literals, so
// repeated bodies cannot be served from content-addressed caches and every
// file forces a real check — the same uniqueness rule the upstream benchmark
// applies to its timed corpus.
function componentSource(index: number): string {
  const id = String(index).padStart(4, "0");
  const shape = index % 4;
  if (shape === 0) {
    return `<script setup lang="ts">
import { ref } from "vue";
const count${id} = ref(0);
const label${id}: string = "counter ${id}";
const bump${id} = (): void => {
  count${id}.value += 1;
};
</script>

<template>
  <button type="button" @click="bump${id}">{{ label${id} }} {{ count${id} }}</button>
</template>
`;
  }
  if (shape === 1) {
    return `<script setup lang="ts">
const disabled${id}: boolean = ${index % 2 === 0};
const title${id}: string = "row ${id}";
</script>

<template>
  <button type="button" :disabled="disabled${id}">{{ title${id} }}</button>
</template>
`;
  }
  if (shape === 2) {
    return `<script setup lang="ts">
const handler${id} = () => {
  console.log("clicked ${id}");
};
</script>

<template>
  <button type="button" @click="handler${id}">go ${id}</button>
</template>
`;
  }
  return `<script setup lang="ts">
import Child from "./Child.vue";
const caption${id}: string = "card ${id}";
</script>

<template>
  <Child variant="primary" />
  <span>{{ caption${id} }}</span>
</template>
`;
}

const childSource = `<script setup lang="ts">
defineProps<{
  variant: "primary" | "secondary";
}>();
</script>

<template>
  <span>{{ variant }}</span>
</template>
`;

test("vue-benchmarks plants stay caught across the scaled unique corpus", async (t) => {
  assert.equal(
    process.env.VIZE_TEST_REQUIRE_TSGO,
    "1",
    "the scaled vue-benchmarks gate must run with VIZE_TEST_REQUIRE_TSGO=1",
  );
  const corsaPath = resolveTsgoBinary();
  const vueTscPath = resolveVueTscBinary();
  const outputRoot = path.join(repoRoot, "target/vize-tests/vue-benchmarks-scaled-corpus");
  fs.mkdirSync(outputRoot, { recursive: true });
  const workspaceDir = fs.mkdtempSync(path.join(outputRoot, "corpus-"));
  const corpusFiles = writeCorpus(workspaceDir);

  try {
    await t.test("corpus contents are unique per file", () => {
      const bodies = new Set(
        corpusFiles.map((file) => fs.readFileSync(path.join(workspaceDir, file), "utf8")),
      );
      assert.equal(bodies.size, corpusFiles.length, "corpus bodies must be unique");
      for (const plant of plants) {
        assert.ok(corpusFiles.includes(plant.file), `corpus must contain ${plant.file}`);
      }
    });

    const clean = runVizeCheck(workspaceDir, corsaPath, []);
    await t.test("clean corpus reports zero diagnostics over every file", () => {
      assertClean(clean, runVueTsc(workspaceDir, vueTscPath), corpusFiles);
    });

    const cleanSources = new Map<string, string>();
    for (const plant of plants) {
      const sourcePath = path.join(workspaceDir, plant.file);
      const source = fs.readFileSync(sourcePath, "utf8");
      cleanSources.set(plant.file, source);
      fs.writeFileSync(sourcePath, replaceExact(source, plant.clean, plant.broken), "utf8");
    }

    await t.test("all planted diagnostics are found in the full corpus", () => {
      const brokenFirst = runVizeCheck(workspaceDir, corsaPath, []);
      const brokenSecond = runVizeCheck(workspaceDir, corsaPath, []);
      const vueTsc = runVueTsc(workspaceDir, vueTscPath);

      assert.equal(brokenFirst.status, 1, brokenFirst.stderr || brokenFirst.stdout);
      assert.deepEqual(
        omitProgramEvidence(brokenFirst.report),
        expectedReport(corpusFiles, plants),
        JSON.stringify(brokenFirst.report),
      );
      assert.equal(brokenSecond.stdout, brokenFirst.stdout, "diagnostics must be byte-stable");
      assert.equal(vueTsc.status, 2, vueTsc.stderr || vueTsc.stdout);
      assert.equal(vueTsc.stdout, plants.map((plant) => plant.vueTscOutput).join(""));
    });

    await t.test("repairing every plant restores the clean corpus byte-for-byte", () => {
      for (const plant of plants) {
        const sourcePath = path.join(workspaceDir, plant.file);
        const broken = fs.readFileSync(sourcePath, "utf8");
        fs.writeFileSync(sourcePath, replaceExact(broken, plant.broken, plant.clean), "utf8");
        assert.equal(fs.readFileSync(sourcePath, "utf8"), cleanSources.get(plant.file));
      }
      const repaired = runVizeCheck(workspaceDir, corsaPath, []);
      assertClean(repaired, runVueTsc(workspaceDir, vueTscPath), corpusFiles);
      assert.equal(repaired.stdout, clean.stdout, "repair must restore byte-stable clean output");
    });

    await t.test("upstream provenance stays pinned", () => {
      assert.deepEqual(upstream, {
        commit: "02c0dac76075924bfb8e9b6985e51a9795ff6909",
        license: "MIT",
        repository: "pikax/vue-benchmarks",
        url: "https://github.com/pikax/vue-benchmarks",
      });
    });
  } finally {
    fs.rmSync(workspaceDir, { recursive: true, force: true });
  }
});

function writeCorpus(workspaceDir: string): string[] {
  const files: string[] = [];
  for (let index = 0; index < corpusFileCount; index++) {
    const name = componentName(index);
    fs.writeFileSync(path.join(workspaceDir, name), componentSource(index), "utf8");
    files.push(name);
  }
  fs.writeFileSync(path.join(workspaceDir, "Child.vue"), childSource, "utf8");
  files.push("Child.vue");
  fs.writeFileSync(path.join(workspaceDir, "tsconfig.json"), JSON.stringify(tsconfig, null, 2));
  fs.writeFileSync(
    path.join(workspaceDir, "package.json"),
    JSON.stringify({ name: "vize-vue-benchmarks-scaled-corpus", private: true, type: "module" }),
  );
  symlinkVueTypes(workspaceDir);
  return files;
}

function replaceExact(source: string, expected: string, replacement: string): string {
  const first = source.indexOf(expected);
  assert.notEqual(first, -1, `missing patch anchor: ${expected}`);
  assert.equal(
    source.indexOf(expected, first + expected.length),
    -1,
    "patch anchor must be unique",
  );
  return `${source.slice(0, first)}${replacement}${source.slice(first + expected.length)}`;
}

function expectedReport(corpusFiles: string[], activePlants: ScaledPlant[]): unknown {
  const diagnosticsByFile = new Map(activePlants.map((plant) => [plant.file, plant.diagnostics]));
  const entries = [...corpusFiles]
    .sort()
    .map((file) => ({ file, diagnostics: diagnosticsByFile.get(file) ?? [] }));
  return {
    files: entries,
    errorCount: entries.reduce((count, entry) => count + entry.diagnostics.length, 0),
    warningCount: 0,
    fileCount: entries.length,
  };
}

function assertClean(vize: VizeCheckResult, vueTsc: CommandResult, corpusFiles: string[]): void {
  assert.equal(vize.status, 0, vize.stderr || vize.stdout);
  assert.deepEqual(
    omitProgramEvidence(vize.report),
    expectedReport(corpusFiles, []),
    JSON.stringify(vize.report),
  );
  assert.equal(vueTsc.status, 0, vueTsc.stderr || vueTsc.stdout);
  assert.equal(vueTsc.stdout, "");
}
