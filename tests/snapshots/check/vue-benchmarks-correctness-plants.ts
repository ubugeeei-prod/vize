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

// #3283 asks for "minimal MIT-licensed typecheck plants". The plant sources
// below are written here rather than copied, but the gate classes and the
// confirmation methodology come from pikax/vue-benchmarks, which is
// MIT-licensed (https://github.com/pikax/vue-benchmarks/blob/main/LICENSE), so
// the attribution is recorded as data and asserted rather than left in a
// comment that can rot.
const upstream = {
  commit: "02c0dac76075924bfb8e9b6985e51a9795ff6909",
  license: "MIT",
  repository: "pikax/vue-benchmarks",
  url: "https://github.com/pikax/vue-benchmarks",
};

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

type Plant = {
  broken: string;
  clean: string;
  diagnostics: Record<string, string[]>;
  files: Record<string, string>;
  id: string;
  sourceFile: string;
  vueTscOutput: string;
};

const plants: Plant[] = [
  {
    id: "script assignment",
    sourceFile: "App.vue",
    clean: "const n: number = 1;",
    broken: 'const n: number = "not-a-number";',
    files: {
      "App.vue": `<script setup lang="ts">
const n: number = 1;
</script>

<template>
  <div>{{ n }}</div>
</template>
`,
    },
    diagnostics: {
      "App.vue": ["error:2:7 [TS2322] Type 'string' is not assignable to type 'number'."],
    },
    vueTscOutput: "App.vue(2,7): error TS2322: Type 'string' is not assignable to type 'number'.\n",
  },
  {
    id: "native boolean prop",
    sourceFile: "App.vue",
    clean: "const disabledFlag: boolean = true;",
    broken: 'const disabledFlag: string = "yes";',
    files: {
      "App.vue": `<script setup lang="ts">
const disabledFlag: boolean = true;
</script>

<template>
  <button type="button" :disabled="disabledFlag">go</button>
</template>
`,
    },
    diagnostics: {
      "App.vue": [
        "error:6:26 [TS2322] Type 'string' is not assignable to type 'Booleanish | undefined'.",
      ],
    },
    vueTscOutput:
      "App.vue(6,26): error TS2322: Type 'string' is not assignable to type 'Booleanish | undefined'.\n",
  },
  {
    id: "native event handler",
    sourceFile: "App.vue",
    clean: "const clickHandler = () => {};",
    broken: "const clickHandler = 123;",
    files: {
      "App.vue": `<script setup lang="ts">
const clickHandler = () => {};
</script>

<template>
  <button type="button" @click="clickHandler">go</button>
</template>
`,
    },
    diagnostics: {
      "App.vue": [
        "error:6:33 [TS2345] Argument of type 'number' is not assignable to parameter of type '(_e: PointerEvent) => unknown'.",
      ],
    },
    vueTscOutput:
      "App.vue(6,4): error TS2345: Argument of type '{ type: \"button\"; onClick: 123; }' is not assignable to parameter of type 'ButtonHTMLAttributes & ReservedProps'.\n  Type '{ type: \"button\"; onClick: 123; }' is not assignable to type 'ButtonHTMLAttributes'.\n    Types of property 'onClick' are incompatible.\n      Type 'number' is not assignable to type '(payload: PointerEvent) => void'.\n",
  },
  {
    id: "static literal-union component prop",
    sourceFile: "App.vue",
    clean: '  <Child variant="primary" />',
    broken: '  <Child variant="danger" />',
    files: {
      "App.vue": `<script setup lang="ts">
import Child from "./Child.vue";
</script>

<template>
  <Child variant="primary" />
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
    diagnostics: {
      "App.vue": [
        'error:6:10 [TS2322] Type \'"danger"\' is not assignable to type \'"primary" | "secondary"\'.',
      ],
      "Child.vue": [],
    },
    vueTscOutput:
      'App.vue(6,10): error TS2322: Type \'"danger"\' is not assignable to type \'"primary" | "secondary"\'.\n',
  },
  {
    id: "strict component attrs",
    sourceFile: "App.vue",
    clean: "  <!-- strict attrs stay absent -->",
    broken: `  <PlainChild known="ok" not-declared="bad" />
  <PlainChild known="ok" data-test="bad" />
  <DefaultInheritChild ghost="bad" />
  <NoInheritChild ghost="bad" />
  <GenericNoInheritChild value="ok" ghost="bad" />`,
    files: {
      "App.vue": `<script setup lang="ts">
import PlainChild from "./PlainChild.vue";
import DefaultInheritChild from "./DefaultInheritChild.vue";
import NoInheritChild from "./NoInheritChild.vue";
import GenericNoInheritChild from "./GenericNoInheritChild.vue";
</script>

<template>
  <!-- strict attrs stay absent -->
</template>
`,
      "PlainChild.vue": `<script setup lang="ts">
defineProps<{
  known: string;
}>();
</script>

<template>
  <template />
</template>
`,
      "DefaultInheritChild.vue": `<template>
  <template />
</template>
`,
      "NoInheritChild.vue": `<script setup lang="ts">
defineOptions({ inheritAttrs: false });
</script>

<template>
  <template />
</template>
`,
      "GenericNoInheritChild.vue": `<script setup lang="ts" generic="T extends string">
defineOptions({ inheritAttrs: false });
defineProps<{
  value: T;
}>();
</script>

<template>
  <template />
</template>
`,
    },
    diagnostics: {
      "App.vue": [
        `error:9:26 [TS2353] Object literal may only specify known properties, and '"notDeclared"' does not exist in type '{ readonly known: string; } & __VizePublicComponentAttrs'.`,
        `error:10:26 [TS2353] Object literal may only specify known properties, and '"dataTest"' does not exist in type '{ readonly known: string; } & __VizePublicComponentAttrs'.`,
        `error:11:24 [TS2353] Object literal may only specify known properties, and '"ghost"' does not exist in type '{} & __VizePublicComponentAttrs'.`,
        `error:12:19 [TS2353] Object literal may only specify known properties, and '"ghost"' does not exist in type '{} & __VizePublicComponentAttrs'.`,
        `error:13:37 [TS2353] Object literal may only specify known properties, and '"ghost"' does not exist in type 'Partial<Props<"ok">> & { class?: unknown; style?: unknown; }'.`,
      ],
      "DefaultInheritChild.vue": [],
      "GenericNoInheritChild.vue": [],
      "NoInheritChild.vue": [],
      "PlainChild.vue": [],
    },
    vueTscOutput:
      "App.vue(9,26): error TS2353: Object literal may only specify known properties, and 'notDeclared' does not exist in type '{ readonly known: string; } & VNodeProps & AllowedComponentProps & ComponentCustomProps'.\n" +
      "App.vue(10,26): error TS2353: Object literal may only specify known properties, and 'dataTest' does not exist in type '{ readonly known: string; } & VNodeProps & AllowedComponentProps & ComponentCustomProps'.\n" +
      "App.vue(11,24): error TS2353: Object literal may only specify known properties, and 'ghost' does not exist in type 'Partial<{}> & Omit<{} & VNodeProps & AllowedComponentProps & ComponentCustomProps, never>'.\n" +
      "App.vue(12,19): error TS2353: Object literal may only specify known properties, and 'ghost' does not exist in type 'Partial<{}> & Omit<{} & VNodeProps & AllowedComponentProps & ComponentCustomProps, never>'.\n" +
      `App.vue(13,37): error TS2353: Object literal may only specify known properties, and 'ghost' does not exist in type 'VNodeProps & AllowedComponentProps & ComponentCustomProps & { value: "ok"; }'.\n`,
  },
];

test("vue-benchmarks correctness plants fail closed on current main", async (t) => {
  assert.equal(
    process.env.VIZE_TEST_REQUIRE_TSGO,
    "1",
    "the vue-benchmarks gate must run with VIZE_TEST_REQUIRE_TSGO=1",
  );
  const corsaPath = resolveTsgoBinary();
  const vueTscPath = resolveVueTscBinary();
  const outputRoot = path.join(repoRoot, "target/vize-tests/vue-benchmarks-plants");
  fs.mkdirSync(outputRoot, { recursive: true });

  for (const plant of plants) {
    await t.test(plant.id, () => {
      const workspaceDir = fs.mkdtempSync(path.join(outputRoot, "plant-"));
      try {
        writeProject(workspaceDir, plant.files);
        symlinkVueTypes(workspaceDir);

        const clean = runVizeCheck(workspaceDir, corsaPath, []);
        assertClean(clean, runVueTsc(workspaceDir, vueTscPath), plant.files);

        const sourcePath = path.join(workspaceDir, plant.sourceFile);
        const cleanSource = fs.readFileSync(sourcePath, "utf8");
        const brokenSource = replaceExact(cleanSource, plant.clean, plant.broken);
        fs.writeFileSync(sourcePath, brokenSource, "utf8");

        const brokenFirst = runVizeCheck(workspaceDir, corsaPath, []);
        const brokenSecond = runVizeCheck(workspaceDir, corsaPath, []);
        const vueTsc = runVueTsc(workspaceDir, vueTscPath);
        assert.equal(brokenFirst.status, 1, brokenFirst.stderr || brokenFirst.stdout);
        assert.deepEqual(
          omitProgramEvidence(brokenFirst.report),
          expectedReport(plant.files, plant.diagnostics),
          JSON.stringify(brokenFirst.report),
        );
        assert.equal(brokenSecond.stdout, brokenFirst.stdout, "diagnostics must be byte-stable");
        assert.equal(vueTsc.status, 2, vueTsc.stderr || vueTsc.stdout);
        assert.equal(vueTsc.stdout, plant.vueTscOutput);

        fs.writeFileSync(sourcePath, replaceExact(brokenSource, plant.broken, plant.clean), "utf8");
        const repaired = runVizeCheck(workspaceDir, corsaPath, []);
        assertClean(repaired, runVueTsc(workspaceDir, vueTscPath), plant.files);
        assert.equal(repaired.stdout, clean.stdout, "repair must restore byte-stable clean output");
        assert.equal(fs.readFileSync(sourcePath, "utf8"), cleanSource);
      } finally {
        fs.rmSync(workspaceDir, { recursive: true, force: true });
      }
    });
  }

  await t.test("upstream provenance stays pinned", () => {
    assert.deepEqual(upstream, {
      commit: "02c0dac76075924bfb8e9b6985e51a9795ff6909",
      license: "MIT",
      repository: "pikax/vue-benchmarks",
      url: "https://github.com/pikax/vue-benchmarks",
    });
  });
});

function writeProject(workspaceDir: string, files: Record<string, string>): void {
  for (const [relativePath, source] of Object.entries(files)) {
    fs.writeFileSync(path.join(workspaceDir, relativePath), source, "utf8");
  }
  fs.writeFileSync(path.join(workspaceDir, "tsconfig.json"), JSON.stringify(tsconfig, null, 2));
  fs.writeFileSync(
    path.join(workspaceDir, "package.json"),
    JSON.stringify({ name: "vize-vue-benchmarks-plants", private: true, type: "module" }),
  );
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

function expectedReport(
  files: Record<string, string>,
  diagnostics: Record<string, string[]>,
): unknown {
  const entries = Object.keys(files)
    .sort()
    .map((file) => ({ file, diagnostics: diagnostics[file] ?? [] }));
  return {
    files: entries,
    errorCount: entries.reduce((count, entry) => count + entry.diagnostics.length, 0),
    warningCount: 0,
    fileCount: entries.length,
  };
}

function assertClean(
  vize: VizeCheckResult,
  vueTsc: CommandResult,
  files: Record<string, string>,
): void {
  assert.equal(vize.status, 0, vize.stderr || vize.stdout);
  assert.deepEqual(
    omitProgramEvidence(vize.report),
    expectedReport(files, {}),
    JSON.stringify(vize.report),
  );
  assert.equal(vueTsc.status, 0, vueTsc.stderr || vueTsc.stdout);
  assert.equal(vueTsc.stdout, "");
}
