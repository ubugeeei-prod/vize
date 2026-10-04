/** Real-runtime, complete-vector regression cases for shared-script sharding. */
import assert from "node:assert/strict";
import { copyFileSync, mkdirSync, readFileSync, symlinkSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";

const PACKAGE_GLOBAL_SOURCES = {
  "node_modules/leaf-global/package.json":
    '{"name":"leaf-global","version":"1.0.0","types":"index.d.ts"}',
  "node_modules/leaf-global/index.d.ts": "export {}; declare global { type LeafGlobal = number; }",
};
const TEMPLATE_PACKAGE_SOURCES = {
  "node_modules/leaf-types/package.json":
    '{"name":"leaf-types","version":"1.0.0","types":"index.d.ts"}',
  "node_modules/leaf-types/index.d.ts":
    PACKAGE_GLOBAL_SOURCES["node_modules/leaf-global/index.d.ts"],
};
const NESTED_VUE_SOURCES = {
  "sub/node_modules/vue/package.json": '{"name":"vue","version":"0.0.0","types":"index.d.ts"}',
  "sub/node_modules/vue/index.d.ts": "export {}; declare global { type LeafGlobal = number; }",
};

const CASES = [
  {
    id: "leaf-error",
    leaf: 'export const value: number = "bad";',
    expectedError: true,
    split: true,
  },
  {
    id: "global-augmentation",
    leaf: 'export const value: number = "bad";',
    extra: { "augment.ts": "export {}; declare global { interface Window { leaf: number } }" },
    expectedError: true,
    split: true,
  },
  {
    id: "global-namespace",
    leaf: "export const value: Common.Value = 1;",
    extra: { "globals.ts": "namespace Common { export type Value = number; }" },
    split: false,
  },
  {
    id: "forced-module-namespace",
    leaf: "export const value: Common.Value = 1;",
    extra: { "globals.ts": "namespace Common { export type Value = number; }" },
    options: { moduleDetection: "force" },
    expectedError: true,
    split: false,
  },
  {
    id: "exported-namespace",
    leaf: "export namespace Local { export type Value = number; } export const value: Local.Value = 1;",
    split: false,
  },
  {
    id: "check-js-enabled",
    file: "shared.js",
    leaf: '/** @type {number} */ export const value = "bad";',
    options: { allowJs: true, checkJs: true },
    expectedError: true,
    split: true,
  },
  {
    id: "check-js-disabled",
    file: "shared.js",
    leaf: '/** @type {number} */ export const value = "bad";',
    options: { allowJs: true, checkJs: false },
    split: true,
  },
  {
    id: "package-global-augmentation",
    leaf: "export const value: LeafGlobal = 1;",
    extra: PACKAGE_GLOBAL_SOURCES,
    vueImport: "import /* trivia */ 'leaf-global';",
    split: false,
  },
  {
    id: "template-package-global-augmentation",
    leaf: "export const value: LeafGlobal = 1;",
    extra: TEMPLATE_PACKAGE_SOURCES,
    vueImport: "void import(/* trivia */ `leaf-types`);",
    split: false,
  },
  {
    id: "entity-template-package-global-augmentation",
    leaf: "export const value: LeafGlobal = 1;",
    extra: TEMPLATE_PACKAGE_SOURCES,
    template: "<div>{{ n }} {{ &#105;mport('leaf-types') }}</div>",
    split: false,
  },
  {
    id: "nested-vue-global-augmentation",
    leaf: "export const value: LeafGlobal = 1;",
    extra: NESTED_VUE_SOURCES,
    vueImport: "import 'vue';",
    nestedVue: true,
    split: false,
  },
  {
    id: "nested-vue-paths-override",
    leaf: "export const value: LeafGlobal = 1;",
    extra: NESTED_VUE_SOURCES,
    vueImport: "import 'vue';",
    nestedVue: true,
    options: { paths: { vue: ["./sub/node_modules/vue/index.d.ts"] } },
    split: false,
  },
  {
    id: "declaration-trivia-global-augmentation",
    leaf: "export const value: LeafGlobal = 1;",
    vueImport: "declare /* gap */\tglobal { type LeafGlobal = number; }",
    split: false,
  },
  {
    id: "optional-require-js-global-augmentation",
    leaf: "export const value: LeafGlobal = 1;",
    extra: {
      ...TEMPLATE_PACKAGE_SOURCES,
      "loader.js":
        "import { value } from './shared'; export const loaded = require?.('leaf-types'); void value;",
      "globals.d.ts": "declare function require(name: string): any;",
    },
    options: { allowJs: true, checkJs: true },
    split: false,
  },
  ...[
    ["malformed-import-caret", 'import ^ "leaf-types";'],
    ["malformed-import-missing-from", 'import x "leaf-types";'],
    ["malformed-import-optional", 'void import?.("leaf-types");'],
    ["malformed-export-caret", 'export ^ "leaf-types";'],
    ["malformed-export-missing-from", 'export * "leaf-types";'],
    ["malformed-export-from-caret", 'export * from ^ "leaf-types";'],
  ].map(([id, vueImport]) => ({
    id,
    leaf: "export const value: LeafGlobal = 1;",
    extra: TEMPLATE_PACKAGE_SOURCES,
    vueImport,
    expectedError: true,
    split: false,
  })),
  {
    id: "relative-ambient-augmentation",
    leaf: "export const value: LeafGlobal = 1;",
    extra: { "types/custom.d.ts": "export {}; declare global { type LeafGlobal = number; }" },
    vueImport: "import './types/custom';",
    split: false,
  },
  {
    id: "transitive-vue",
    leaf: "export { value } from './other';",
    extra: { "other.ts": "export const value = 1;" },
    vueImport: "import Child from './Comp1.vue'; void Child;",
    split: false,
  },
];

export function checkLeafParity(directory, vuePackageDir, run) {
  const rows = [];
  for (const fixture of CASES) {
    const dir = join(directory, "work", `leaf-parity-${fixture.id}`);
    const input = join(directory, "inputs", `leaf-parity-${fixture.id}`);
    mkdirSync(dir);
    mkdirSync(input);
    const sources = { [fixture.file ?? "shared.ts"]: fixture.leaf, ...fixture.extra };
    for (let index = 0; index < 4; index++) {
      const nested = fixture.nestedVue && index === 0;
      const template =
        index === 0 ? (fixture.template ?? "<div>{{ n }}</div>") : "<div>{{ n }}</div>";
      sources[`${nested ? "sub/" : ""}Comp${index}.vue`] =
        `<script setup lang="ts">import { value } from '${nested ? "../shared" : fixture.file === "shared.js" ? "./shared.js" : "./shared"}'; ${index === 0 ? (fixture.vueImport ?? "") : ""} const n = value;</script><template>${template}</template>`;
    }
    sources["tsconfig.json"] = JSON.stringify({
      compilerOptions: {
        strict: true,
        module: "ESNext",
        moduleResolution: "Bundler",
        target: "ESNext",
        lib: ["ESNext", "DOM"],
        skipLibCheck: true,
        ...fixture.options,
      },
      include: [fixture.nestedVue ? "**/*.vue" : "*.vue", "*.ts", "*.js"],
    });
    for (const [file, content] of Object.entries(sources)) {
      mkdirSync(dirname(join(dir, file)), { recursive: true });
      mkdirSync(dirname(join(input, file)), { recursive: true });
      writeFileSync(join(dir, file), content);
      copyFileSync(join(dir, file), join(input, file));
    }
    mkdirSync(join(dir, "node_modules"), { recursive: true });
    symlinkSync(vuePackageDir, join(dir, "node_modules/vue"), "dir");
    const corpus = {
      id: `leaf-parity-${fixture.id}`,
      args: ["--no-config"],
      expectedVuePaths: Object.keys(sources).filter((file) => file.endsWith(".vue")),
    };
    let reference;
    for (const servers of [1, 2]) {
      const mode = { id: `${servers}-server`, args: ["--servers", String(servers)], rayon: 1 };
      for (const side of ["base", "head"]) {
        const checked = run(side, mode, corpus, dir, "parity");
        reference ??= checked.report;
        assert.deepEqual(
          checked.report,
          reference,
          `${fixture.id}/${side}/${servers}: complete ordered report changed`,
        );
        assert.equal(
          checked.report.errorCount > 0,
          fixture.expectedError ?? false,
          `${fixture.id}: diagnostic plant/clean status changed`,
        );
        rows.push({
          case: fixture.id,
          side,
          servers,
          sampleId: checked.id,
          fingerprint: checked.fingerprint,
        });
      }
    }
    const profiled = run(
      "head",
      { id: "2-server", args: ["--servers", "2"], rayon: 1 },
      corpus,
      dir,
      "parity-profile",
      true,
    );
    assert.deepEqual(profiled.report, reference);
    const profile = JSON.parse(readFileSync(join(directory, profiled.profileFile), "utf8"));
    const shards =
      profile.counters.find((counter) => counter.key === "canon.corsa.cli.shards")?.total ?? 1;
    assert.equal(
      shards > 1,
      fixture.split,
      `${fixture.id}: intended partition path was not exercised`,
    );
  }
  return rows;
}
