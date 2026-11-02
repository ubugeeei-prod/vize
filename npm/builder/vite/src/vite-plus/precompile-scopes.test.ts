import assert from "node:assert/strict";
import path from "node:path";
import { test } from "node:test";
import type { InlineConfig } from "vite";
import { defineConfig } from "../vite-plus.ts";
import type { VueOptions } from "./types.ts";
import { vize } from "../plugin/index.ts";
import { mergeCompilerOptions } from "../plugin/compiler-config.ts";
import {
  observeConsole,
  reportedProject,
  retainObservation,
  startupHost,
} from "../test/startup-scan-project.ts";

const original: VueOptions = { files: ["src/**/*.vue"], ignores: ["src/Ignored.vue"] };
const env = { command: "build", mode: "production" } as const;
const app = ["src/App.vue"];

void test("reported withVue files and ignores select only App during actual startup", async () => {
  const cases: { name: string; vue: VueOptions; expected: string[] }[] = [
    { name: "original", vue: original, expected: app },
    {
      name: "scan override",
      vue: { ...original, vite: { scanPatterns: ["other/**/*.vue"] } },
      expected: ["other/Outside.vue"],
    },
    {
      name: "ignore override",
      vue: { ...original, vite: { ignorePatterns: [] } },
      expected: ["src/App.vue", "src/Ignored.vue"],
    },
    {
      name: "both overrides",
      vue: { ...original, vite: { scanPatterns: ["**/*.vue"], ignorePatterns: [] } },
      expected: ["other/Outside.vue", "src/App.vue", "src/Ignored.vue"],
    },
    { name: "explicit empty scan", vue: { ...original, vite: { scanPatterns: [] } }, expected: [] },
    { name: "canonical empty files", vue: { files: [], ignores: original.ignores }, expected: [] },
    {
      name: "canonical empty ignores",
      vue: { files: original.files, ignores: [] },
      expected: ["src/App.vue", "src/Ignored.vue"],
    },
    {
      name: "undefined admits canonical",
      vue: { ...original, vite: { scanPatterns: undefined, ignorePatterns: undefined } },
      expected: app,
    },
  ];
  for (const row of cases) {
    const project = reportedProject(11376);
    const config = await defineConfig({}).withVue(row.vue)(env);
    const host = await startupHost(project.root, config as unknown as InlineConfig, "build", true);
    try {
      const observed = await observeConsole(() => host.start());
      const actual = host.candidates
        .map(({ file, source }) => ({
          file: path.relative(project.root, file).replaceAll("\\", "/"),
          source,
        }))
        .sort((a, b) => a.file.localeCompare(b.file));
      retainObservation(`withVue-${row.name}`, {
        fixture: project.fixture,
        vue: row.vue,
        actual,
        observed,
      });
      assert.equal(observed.error, null);
      assert.deepEqual(
        actual,
        row.expected.map((file) => ({
          file,
          source: project.fixture.files.find((input) => input.path === file)!.source,
        })),
      );
      if (row.name === "original") assert.deepEqual(observed.calls.warn, []);
    } finally {
      host.close();
      project.cleanup();
    }
  }
});

void test("direct plugin precedence stays per-field and explicit empty arrays survive", async () => {
  const project = reportedProject(11376);
  try {
    const scoped = {
      ...original,
      vite: { scanPatterns: ["other/**/*.vue"], ignorePatterns: ["other/Outside.vue"] },
    };
    const options = { scanPatterns: ["src/**/*.vue"], ignorePatterns: [] };
    const resolved = mergeCompilerOptions(options, scoped);
    assert.deepEqual([resolved.scanPatterns, resolved.ignorePatterns], [options.scanPatterns, []]);
    const host = await startupHost(
      project.root,
      { plugins: vize({ ...options, config: scoped, configMode: false }) },
      "build",
      true,
    );
    const observed = await observeConsole(() => host.start());
    host.close();
    retainObservation("plugin-overrides", {
      scoped,
      options,
      observed,
      candidates: host.candidates,
    });
    assert.equal(observed.error, null);
    assert.deepEqual(
      host.candidates
        .map(({ file }) => path.relative(project.root, file).replaceAll("\\", "/"))
        .sort(),
      ["src/App.vue", "src/Ignored.vue"],
    );
    const empty = mergeCompilerOptions({ scanPatterns: [], ignorePatterns: [] }, scoped);
    assert.deepEqual([empty.scanPatterns, empty.ignorePatterns], [[], []]);
    const absent = mergeCompilerOptions({}, null);
    assert.deepEqual([absent.scanPatterns, absent.ignorePatterns], [undefined, undefined]);
  } finally {
    project.cleanup();
  }
});

void test("withVue composition retains its existing undefined overlay and empty reset laws", async () => {
  const project = reportedProject(11376);
  try {
    for (const override of [undefined, []] as const) {
      const factory = defineConfig({})
        .withVue(original)
        .withVue({ vite: { scanPatterns: ["other/**/*.vue"] } })
        .withVue({ vite: { scanPatterns: override === undefined ? undefined : [...override] } });
      const host = await startupHost(
        project.root,
        (await factory(env)) as unknown as InlineConfig,
        "build",
        true,
      );
      const observed = await observeConsole(() => host.start());
      host.close();
      retainObservation("composition", { override, candidates: host.candidates, observed });
      assert.equal(observed.error, null);
      // Undefined does not erase an earlier explicit value; [] intentionally disables scanning.
      assert.deepEqual(
        host.candidates.map(({ file }) => path.relative(project.root, file).replaceAll("\\", "/")),
        override === undefined ? ["other/Outside.vue"] : [],
      );
    }
  } finally {
    project.cleanup();
  }
});
