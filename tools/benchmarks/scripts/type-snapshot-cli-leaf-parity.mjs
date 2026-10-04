/** Real-runtime, complete-vector regression cases for shared-script sharding. */
import assert from "node:assert/strict";
import { copyFileSync, mkdirSync, readFileSync, symlinkSync, writeFileSync } from "node:fs";
import { join } from "node:path";

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
    split: true,
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
    for (let index = 0; index < 4; index++)
      sources[`Comp${index}.vue`] =
        `<script setup lang="ts">import { value } from '${fixture.file === "shared.js" ? "./shared.js" : "./shared"}'; ${index === 0 ? (fixture.vueImport ?? "") : ""} const n = value;</script><template><div>{{ n }}</div></template>`;
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
      include: ["*.vue", "*.ts", "*.js"],
    });
    for (const [file, content] of Object.entries(sources)) {
      writeFileSync(join(dir, file), content);
      copyFileSync(join(dir, file), join(input, file));
    }
    mkdirSync(join(dir, "node_modules"));
    symlinkSync(vuePackageDir, join(dir, "node_modules/vue"), "dir");
    const corpus = { id: `leaf-parity-${fixture.id}`, args: ["--no-config"] };
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
