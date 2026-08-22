import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";

import { repoRoot } from "../../_helpers/realworld-patch.ts";
import {
  omitProgramEvidence,
  resolveTsgoBinary,
  resolveVueTscBinary,
  runVizeCheck,
  runVueTsc,
  symlinkVueTypes,
} from "../../_helpers/realworld-typecheck.ts";

type Case = { files: Record<string, string>; id: string };

const component = (
  prefix: string,
  inheritance: string,
  member: string,
): string => `<script lang="ts">
import Vue from "vue";
${prefix}
export default Vue.extend({
  ${inheritance},
  computed: { consumed() { return this.${member}(); } },
});
</script>
<template><span /></template>
`;

const cases: Case[] = [
  {
    id: "an imported typed mixin",
    files: {
      "greeter.ts": `import Vue from "vue";
export default Vue.extend({ methods: { greet(): string { return "hi"; } } });
`,
      "Uses.vue": component('import greeter from "./greeter";', "mixins: [greeter]", "greet"),
    },
  },
  {
    id: "a typed extends component",
    files: {
      "base.ts": `import Vue from "vue";
export default Vue.extend({ methods: { describe(): string { return "base"; } } });
`,
      "Uses.vue": component('import base from "./base";', "extends: base", "describe"),
    },
  },
  {
    id: "a same-file typed mixin",
    files: {
      "Uses.vue": component(
        `const inlineMixin = Vue.extend({
  methods: { ping(): number { return 1; } },
});`,
        "mixins: [inlineMixin]",
        "ping",
      ),
    },
  },
  {
    id: "an inline object mixin",
    files: {
      "Uses.vue": component("", "mixins: [{ methods: { pong() { return 1; } } }]", "pong"),
    },
  },
  {
    id: "an any-typed mixin",
    files: {
      "untyped.d.ts": "declare const untyped: any; export default untyped;\n",
      "Uses.vue": `<script lang="ts">
import Vue from "vue";
import untyped from "./untyped";
export default Vue.extend({
  mixins: [untyped],
  methods: { own() {} },
  computed: { consumed() { return 1; } },
});
</script>
<template><span /></template>
`,
    },
  },
];

test("legacy inherited members match the complete vue-tsc oracle", async (t) => {
  const corsaPath = resolveTsgoBinary();
  const vueTscPath = resolveVueTscBinary();
  const outputRoot = path.join(repoRoot, "target/vize-tests/options-api-inherited-members");
  fs.mkdirSync(outputRoot, { recursive: true });

  for (const testCase of cases) {
    await t.test(testCase.id, () => {
      const workspaceDir = fs.mkdtempSync(path.join(outputRoot, "case-"));
      try {
        writeProject(workspaceDir, testCase.files);
        symlinkVueTypes(workspaceDir);

        const first = runVizeCheck(workspaceDir, corsaPath, ["**/*.vue"]);
        assert.equal(first.status, 0, first.stderr || first.stdout);
        assert.equal(first.stderr, "");
        assert.deepEqual(omitProgramEvidence(first.report), {
          files: [{ file: "Uses.vue", diagnostics: [] }],
          errorCount: 0,
          warningCount: 0,
          fileCount: 1,
        });
        const second = runVizeCheck(workspaceDir, corsaPath, ["**/*.vue"]);
        assert.equal(second.stdout, first.stdout, "Vize output must be byte-stable");

        const oracle = runVueTsc(workspaceDir, vueTscPath);
        assert.deepEqual(oracle, { status: 0, stderr: "", stdout: "" });
      } finally {
        fs.rmSync(workspaceDir, { recursive: true, force: true });
      }
    });
  }
});

function writeProject(workspaceDir: string, files: Record<string, string>): void {
  for (const [file, source] of Object.entries(files)) {
    fs.writeFileSync(path.join(workspaceDir, file), source, "utf8");
  }
  fs.writeFileSync(path.join(workspaceDir, "vue2.ts"), vue2TypeOracle, "utf8");
  fs.writeFileSync(
    path.join(workspaceDir, "tsconfig.json"),
    `${JSON.stringify(
      {
        compilerOptions: {
          lib: ["ES2022", "DOM"],
          module: "ESNext",
          moduleResolution: "bundler",
          noEmit: true,
          paths: { vue: ["./vue2.ts"], "vue-original": ["./node_modules/vue"] },
          skipLibCheck: true,
          strict: true,
          target: "ES2022",
        },
        include: ["**/*.vue", "**/*.ts"],
      },
      null,
      2,
    )}\n`,
  );
  fs.writeFileSync(
    path.join(workspaceDir, "vize.config.json"),
    `${JSON.stringify({ typeChecker: { legacyVue2: true } }, null, 2)}\n`,
  );
}

const vue2TypeOracle = `export * from "vue-original";

type Constructor = abstract new (...args: any[]) => any;
type Instance<T> = T extends Constructor ? InstanceType<T> : {};
type UnionToIntersection<T> = (T extends unknown ? (value: T) => void : never) extends
  (value: infer I) => void ? I : never;
type ComputedValues<T> = { [K in keyof T]: T[K] extends (...args: any[]) => infer R ? R : never };
type MixinInstance<T> = T extends Constructor
  ? Instance<T>
  : T extends { methods?: infer M; computed?: infer C }
    ? M & ComputedValues<C>
    : {};
type MixinInstances<T extends readonly unknown[]> = UnionToIntersection<MixinInstance<T[number]>>;
type Shape<M extends readonly unknown[], E, D, Methods, Computed> =
  MixinInstances<M> & Instance<E> & D & Methods & ComputedValues<Computed>;

type Options<
  M extends readonly unknown[],
  E,
  D extends object,
  Methods extends object,
  Computed extends object,
> = {
  mixins?: M;
  extends?: E;
  data?: () => D;
  methods?: Methods & ThisType<Shape<M, E, D, Methods, Computed>>;
  computed?: Computed & ThisType<Shape<M, E, D, Methods, Computed>>;
};

interface VueConstructor {
  extend<
    const M extends readonly unknown[] = [],
    E = undefined,
    D extends object = {},
    Methods extends object = {},
    Computed extends object = {},
  >(options: Options<M, E, D, Methods, Computed>):
    new () => Shape<M, E, D, Methods, Computed>;
}

declare const Vue: VueConstructor;
export default Vue;
`;
