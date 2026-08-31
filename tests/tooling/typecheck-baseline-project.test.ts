import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { test } from "node:test";

import { materializeBaselineProject } from "../../legacy-tools/fixtures/typecheck-baseline-project.mjs";
import { typecheckDependencySkip } from "./support/typecheck-dependency.ts";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const dependencyRoot =
  process.env.VIZE_TEST_WORKSPACE_NODE_MODULES ?? path.join(root, "tests/node_modules");
const vueTsc = path.join(dependencyRoot, ".bin/vue-tsc");
const vueTscOptions = {
  skip: typecheckDependencySkip(
    fs.existsSync(vueTsc) ? vueTsc : undefined,
    "vue-tsc for the baseline-project gates",
    "vue-tsc binary unavailable",
  ),
};

test(
  "materialized baseline checks Vue files omitted by a solution-style config",
  vueTscOptions,
  () => {
    const temp = fs.mkdtempSync(path.join(os.tmpdir(), "vize-baseline-project-"));
    const fixtureRoot = path.join(temp, "fixture");
    const reportDir = path.join(temp, "report");
    const app = path.join(fixtureRoot, "src/App.vue");
    fs.mkdirSync(path.dirname(app), { recursive: true });
    fs.mkdirSync(reportDir);
    fs.writeFileSync(
      path.join(fixtureRoot, "tsconfig.json"),
      `${JSON.stringify({ files: [], references: [{ path: "./tsconfig.app.json" }] })}\n`,
    );
    fs.writeFileSync(
      path.join(fixtureRoot, "tsconfig.app.json"),
      `${JSON.stringify({ compilerOptions: { composite: true }, include: ["src/**/*.vue"] })}\n`,
    );
    fs.writeFileSync(app, '<script setup lang="ts">const value: string = 1</script>\n');

    try {
      const direct = runVueTsc(path.join(fixtureRoot, "tsconfig.json"), fixtureRoot);
      assert.doesNotMatch(direct.stdout, /App\.vue/u);

      const project = materializeBaselineProject(
        fixtureRoot,
        reportDir,
        { id: "fixture", tsconfig: "tsconfig.json" },
        { fileCount: 1, files: [{ file: "src/App.vue" }] },
      );
      const materialized = runVueTsc(project.path, fixtureRoot);
      assert.match(materialized.stdout, new RegExp(escapeRegExp(app)));
      assert.equal(materialized.status, 2, materialized.stderr);
    } finally {
      fs.rmSync(temp, { recursive: true, force: true });
    }
  },
);

test("materialized baseline extends an explicit generated project", () => {
  const temp = fs.mkdtempSync(path.join(os.tmpdir(), "vize-generated-baseline-project-"));
  const fixtureRoot = path.join(temp, "fixture");
  const reportDir = path.join(temp, "report");
  fs.mkdirSync(path.join(fixtureRoot, ".generated"), { recursive: true });
  fs.mkdirSync(path.join(fixtureRoot, "src"));
  fs.mkdirSync(reportDir);
  fs.writeFileSync(path.join(fixtureRoot, ".generated/tsconfig.json"), "{}\n");
  fs.writeFileSync(path.join(fixtureRoot, "src/App.vue"), "<template />\n");
  try {
    const project = materializeBaselineProject(
      fixtureRoot,
      reportDir,
      {
        id: "fixture",
        tsconfig: "tsconfig.json",
        typecheckPerformance: { baseline: { tsconfig: ".generated/tsconfig.json" } },
      },
      { fileCount: 1, files: [{ file: "src/App.vue" }] },
    );
    assert.equal(project.sourceProject, ".generated/tsconfig.json");
    assert.match(
      project.path,
      /[/\\]fixture[/\\]\.generated[/\\]\.vize-baseline[/\\]fixture-vue-tsc\.tsconfig\.json$/u,
    );
    assert.equal(JSON.parse(project.source).extends, "../tsconfig.json");
    assert.equal(
      fs.readFileSync(path.join(reportDir, "fixture-vue-tsc.tsconfig.json"), "utf8"),
      project.source,
    );
  } finally {
    fs.rmSync(temp, { recursive: true, force: true });
  }
});

test(
  "materialized baseline resolves inherited workspace type references from the fixture root",
  vueTscOptions,
  () => {
    const temp = fs.mkdtempSync(path.join(os.tmpdir(), "vize-workspace-types-baseline-"));
    const fixtureRoot = path.join(temp, "fixture");
    const reportDir = path.join(temp, "report");
    fs.mkdirSync(path.join(fixtureRoot, "src"), { recursive: true });
    fs.mkdirSync(path.join(fixtureRoot, "node_modules/@vben/types"), { recursive: true });
    fs.mkdirSync(reportDir);
    fs.writeFileSync(
      path.join(fixtureRoot, "tsconfig.json"),
      `${JSON.stringify({
        compilerOptions: {
          strict: true,
          noEmit: true,
          types: ["@vben/types/global"],
        },
      })}\n`,
    );
    fs.writeFileSync(
      path.join(fixtureRoot, "node_modules/@vben/types/global.d.ts"),
      "declare const VbenFixtureGlobal: string;\n",
    );
    fs.writeFileSync(
      path.join(fixtureRoot, "src/App.vue"),
      '<script setup lang="ts">const value: string = VbenFixtureGlobal</script>\n',
    );

    try {
      const project = materializeBaselineProject(
        fixtureRoot,
        reportDir,
        { id: "fixture", tsconfig: "tsconfig.json" },
        { fileCount: 1, files: [{ file: "src/App.vue" }] },
      );
      assert.match(
        project.path,
        /[/\\]fixture[/\\]\.vize-baseline[/\\]fixture-vue-tsc\.tsconfig\.json$/u,
      );
      const result = runVueTsc(project.path, fixtureRoot);
      const diagnostics = result.stdout.split("\n").filter((line) => /: error TS\d+: /u.test(line));
      assert.deepEqual(diagnostics, []);
      assert.equal(result.status, 0, result.stderr);
    } finally {
      fs.rmSync(temp, { recursive: true, force: true });
    }
  },
);

test(
  "materialized baseline resolves package-local workspace type references",
  vueTscOptions,
  () => {
    const temp = fs.mkdtempSync(path.join(os.tmpdir(), "vize-package-types-baseline-"));
    const fixtureRoot = path.join(temp, "fixture");
    const reportDir = path.join(temp, "report");
    fs.mkdirSync(path.join(fixtureRoot, "apps/web/src"), { recursive: true });
    fs.mkdirSync(path.join(fixtureRoot, "playground/node_modules/@vben/types"), {
      recursive: true,
    });
    fs.mkdirSync(reportDir);
    fs.writeFileSync(
      path.join(fixtureRoot, "playground/tsconfig.json"),
      `${JSON.stringify({
        compilerOptions: {
          strict: true,
          noEmit: true,
          types: ["@vben/types/global"],
        },
      })}\n`,
    );
    fs.writeFileSync(
      path.join(fixtureRoot, "playground/node_modules/@vben/types/global.d.ts"),
      "declare const VbenFixtureGlobal: string;\n",
    );
    fs.writeFileSync(
      path.join(fixtureRoot, "apps/web/src/App.vue"),
      '<script setup lang="ts">const value: string = VbenFixtureGlobal</script>\n',
    );

    try {
      const project = materializeBaselineProject(
        fixtureRoot,
        reportDir,
        { id: "fixture", tsconfig: "playground/tsconfig.json" },
        { fileCount: 1, files: [{ file: "apps/web/src/App.vue" }] },
      );
      assert.match(
        project.path,
        /[/\\]fixture[/\\]playground[/\\]\.vize-baseline[/\\]fixture-vue-tsc\.tsconfig\.json$/u,
      );
      const result = runVueTsc(project.path, fixtureRoot);
      const diagnostics = result.stdout.split("\n").filter((line) => /: error TS\d+: /u.test(line));
      assert.deepEqual(diagnostics, []);
      assert.equal(result.status, 0, result.stderr);
    } finally {
      fs.rmSync(temp, { recursive: true, force: true });
    }
  },
);

test(
  "materialized baseline keeps the fixture's ambient declarations in the program",
  vueTscOptions,
  () => {
    // #3738. A `files` list seeds the program with those roots and nothing else,
    // so an ambient declaration — never imported, by definition — leaves the
    // program and the baseline reports the fixture's own globals as undeclared.
    // Both declarations here are the two shapes that failed on run 30738583070:
    // one under a plain directory (lx-music-desktop's `src/**/*.d.ts`), one under
    // a dot-directory that a TypeScript wildcard segment never descends into
    // (elk's `.nuxt/imports.d.ts`).
    const temp = fs.mkdtempSync(path.join(os.tmpdir(), "vize-ambient-baseline-project-"));
    const fixtureRoot = path.join(temp, "fixture");
    const reportDir = path.join(temp, "report");
    fs.mkdirSync(path.join(fixtureRoot, "src"), { recursive: true });
    fs.mkdirSync(path.join(fixtureRoot, ".generated"));
    fs.mkdirSync(reportDir);
    fs.writeFileSync(
      path.join(fixtureRoot, ".generated/tsconfig.json"),
      `${JSON.stringify({ compilerOptions: { strict: true, noEmit: true } })}\n`,
    );
    fs.writeFileSync(
      path.join(fixtureRoot, "src/globals.d.ts"),
      "declare namespace Authored { type Id = string }\n",
    );
    fs.writeFileSync(
      path.join(fixtureRoot, ".generated/imports.d.ts"),
      "declare function generatedHelper(): number\n",
    );
    fs.writeFileSync(
      path.join(fixtureRoot, "src/App.vue"),
      '<script setup lang="ts">const id: Authored.Id = String(generatedHelper())</script>\n',
    );

    try {
      const project = materializeBaselineProject(
        fixtureRoot,
        reportDir,
        {
          id: "fixture",
          tsconfig: "tsconfig.json",
          typecheckPerformance: { baseline: { tsconfig: ".generated/tsconfig.json" } },
        },
        { fileCount: 1, files: [{ file: "src/App.vue" }] },
      );
      const result = runVueTsc(project.path, fixtureRoot);
      const diagnostics = result.stdout.split("\n").filter((line) => /: error TS\d+: /u.test(line));
      assert.deepEqual(diagnostics, []);
      assert.equal(result.status, 0, result.stderr);
      // Both ambient roots reached the program, not just the compared SFC.
      const program = result.stdout.split("\n").map((line) => line.trimEnd());
      assert.equal(program.includes(path.join(fixtureRoot, "src/globals.d.ts")), true);
      assert.equal(program.includes(path.join(fixtureRoot, ".generated/imports.d.ts")), true);
    } finally {
      fs.rmSync(temp, { recursive: true, force: true });
    }
  },
);

test(
  "materialized baseline silences inherited TypeScript 6 deprecation errors",
  vueTscOptions,
  (t) => {
    const version = spawnSync(vueTsc, ["--version"], { encoding: "utf8" }).stdout;
    if (!/\b6\./u.test(version)) {
      t.skip("vue-tsc is not using TypeScript 6");
      return;
    }

    const temp = fs.mkdtempSync(path.join(os.tmpdir(), "vize-deprecation-baseline-project-"));
    const fixtureRoot = path.join(temp, "fixture");
    const reportDir = path.join(temp, "report");
    fs.mkdirSync(path.join(fixtureRoot, "src"), { recursive: true });
    fs.mkdirSync(reportDir);
    fs.writeFileSync(
      path.join(fixtureRoot, "tsconfig.json"),
      `${JSON.stringify({
        compilerOptions: {
          baseUrl: ".",
          moduleResolution: "node10",
          target: "ES5",
        },
      })}\n`,
    );
    fs.writeFileSync(path.join(fixtureRoot, "src/App.vue"), "<template />\n");

    try {
      const project = materializeBaselineProject(
        fixtureRoot,
        reportDir,
        { id: "fixture", tsconfig: "tsconfig.json" },
        { fileCount: 1, files: [{ file: "src/App.vue" }] },
      );
      const result = runVueTsc(project.path, fixtureRoot);
      assert.doesNotMatch(result.stdout, /TS510[17]/u);
      assert.equal(result.status, 0, result.stderr);
      assert.equal(JSON.parse(project.source).compilerOptions.ignoreDeprecations, "6.0");
      assert.equal(JSON.parse(project.source).compilerOptions.rootDir, "..");
    } finally {
      fs.rmSync(temp, { recursive: true, force: true });
    }
  },
);

function runVueTsc(project: string, cwd: string) {
  return spawnSync(vueTsc, ["--noEmit", "--pretty", "false", "--listFiles", "-p", project], {
    cwd,
    encoding: "utf8",
  });
}

function escapeRegExp(value: string) {
  return value.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}
