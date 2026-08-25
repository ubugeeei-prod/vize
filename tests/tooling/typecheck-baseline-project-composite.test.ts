import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { test } from "node:test";

import { materializeBaselineProject } from "../../tools/fixtures/typecheck-baseline-project.mjs";
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
  "materialized baseline includes imported authored scripts for composite projects",
  vueTscOptions,
  () => {
    const temp = fs.mkdtempSync(path.join(os.tmpdir(), "vize-composite-script-baseline-"));
    const fixtureRoot = path.join(temp, "fixture");
    const reportDir = path.join(temp, "report");
    fs.mkdirSync(path.join(fixtureRoot, "src"), { recursive: true });
    fs.mkdirSync(path.join(fixtureRoot, "docs/examples"), { recursive: true });
    fs.mkdirSync(path.join(fixtureRoot, "docs/.vitepress/vitepress/components/common"), {
      recursive: true,
    });
    fs.mkdirSync(path.join(fixtureRoot, "docs/.vitepress/vitepress/components/globals"), {
      recursive: true,
    });
    fs.mkdirSync(path.join(fixtureRoot, "docs/.vitepress/vitepress/utils"), {
      recursive: true,
    });
    fs.mkdirSync(reportDir);
    fs.writeFileSync(
      path.join(fixtureRoot, "tsconfig.json"),
      `${JSON.stringify({
        compilerOptions: {
          allowJs: true,
          composite: true,
          noEmit: true,
          resolveJsonModule: true,
          strict: true,
        },
      })}\n`,
    );
    fs.writeFileSync(path.join(fixtureRoot, "src/value.ts"), "export const value = 'ok';\n");
    fs.writeFileSync(
      path.join(fixtureRoot, "src/App.vue"),
      '<script setup lang="ts">import { value } from "./value"; const label: string = value</script>\n',
    );
    fs.writeFileSync(
      path.join(fixtureRoot, "docs/examples/basic.vue"),
      '<script setup lang="ts">import VpLink from "../.vitepress/vitepress/components/common/vp-link.vue"; const link = VpLink</script>\n',
    );
    fs.writeFileSync(
      path.join(fixtureRoot, "docs/.vitepress/vitepress/utils/index.ts"),
      "export const linkLabel = 'ok';\n",
    );
    fs.writeFileSync(
      path.join(fixtureRoot, "docs/.vitepress/vitepress/utils/label.js"),
      "export const jsLabel = 'ok';\n",
    );
    fs.writeFileSync(
      path.join(fixtureRoot, "docs/.vitepress/vitepress/components/globals/icons-categories.json"),
      '["ok"]\n',
    );
    fs.writeFileSync(
      path.join(fixtureRoot, "docs/.vitepress/vitepress/components/common/vp-link.vue"),
      '<script setup lang="ts">import { linkLabel } from "../../utils"; import { jsLabel } from "../../utils/label"; import categories from "../globals/icons-categories.json"; const labels: string[] = [linkLabel, jsLabel, ...categories]</script>\n',
    );

    try {
      const project = materializeBaselineProject(
        fixtureRoot,
        reportDir,
        {
          id: "fixture",
          tsconfig: "tsconfig.json",
          vueGlobs: ["src/**/*.vue", "docs/**/*.vue"],
        },
        {
          fileCount: 2,
          files: [{ file: "docs/examples/basic.vue" }, { file: "src/App.vue" }],
        },
      );
      const config = JSON.parse(project.source);
      assert.equal(config.include.includes("../src/**/*.ts"), true);
      assert.equal(config.include.includes("../docs/**/*.ts"), true);
      assert.equal(config.include.includes("../docs/.vitepress/**/*.ts"), true);
      assert.equal(config.include.includes("../docs/.vitepress/**/*.js"), true);
      assert.equal(config.include.includes("../docs/.vitepress/**/*.json"), true);
      assert.equal(config.include.includes("../docs/.vitepress/**/*.vue"), true);
      const result = runVueTsc(project.path, fixtureRoot);
      const diagnostics = result.stdout.split("\n").filter((line) => /: error TS\d+: /u.test(line));
      assert.deepEqual(diagnostics, []);
      assert.equal(result.status, 0, result.stderr);
      const program = result.stdout.split("\n").map((line) => line.trimEnd());
      assert.equal(program.includes(path.join(fixtureRoot, "src/value.ts")), true);
      assert.equal(
        program.includes(path.join(fixtureRoot, "docs/.vitepress/vitepress/utils/index.ts")),
        true,
      );
      assert.equal(
        program.includes(path.join(fixtureRoot, "docs/.vitepress/vitepress/utils/label.js")),
        true,
      );
      assert.equal(
        program.includes(
          path.join(fixtureRoot, "docs/.vitepress/vitepress/components/common/vp-link.vue"),
        ),
        true,
      );
    } finally {
      fs.rmSync(temp, { recursive: true, force: true });
    }
  },
);

test("materialized baseline include roots follow corpusGlobs instead of sibling vueGlobs", () => {
  const temp = fs.mkdtempSync(path.join(os.tmpdir(), "vize-corpus-globs-baseline-"));
  const fixtureRoot = path.join(temp, "fixture");
  const reportDir = path.join(temp, "report");
  fs.mkdirSync(path.join(fixtureRoot, "packages/lib"), { recursive: true });
  fs.mkdirSync(path.join(fixtureRoot, "apps/volt"), { recursive: true });
  fs.mkdirSync(reportDir);
  fs.writeFileSync(path.join(fixtureRoot, "tsconfig.json"), "{}\n");
  fs.writeFileSync(path.join(fixtureRoot, "packages/lib/Button.vue"), "<template />\n");
  fs.writeFileSync(path.join(fixtureRoot, "apps/volt/Page.vue"), "<template />\n");
  try {
    const project = materializeBaselineProject(
      fixtureRoot,
      reportDir,
      {
        id: "fixture",
        tsconfig: "tsconfig.json",
        vueGlobs: ["packages/lib/**/*.vue", "apps/volt/**/*.vue"],
        typecheckPerformance: { corpusGlobs: ["packages/lib/**/*.vue"] },
      },
      { fileCount: 1, files: [{ file: "packages/lib/Button.vue" }] },
    );
    const config = JSON.parse(project.source);
    assert.equal(config.include.includes("../packages/lib/**/*.ts"), true);
    assert.equal(config.include.includes("../apps/volt/**/*.ts"), false);
  } finally {
    fs.rmSync(temp, { recursive: true, force: true });
  }
});

test(
  "materialized baseline keeps sibling app declarations out of package-local typecheck",
  vueTscOptions,
  () => {
    const temp = fs.mkdtempSync(path.join(os.tmpdir(), "vize-package-dts-baseline-"));
    const fixtureRoot = path.join(temp, "fixture");
    const reportDir = path.join(temp, "report");
    fs.mkdirSync(path.join(fixtureRoot, "packages/common/src"), { recursive: true });
    fs.mkdirSync(path.join(fixtureRoot, "packages/admin/src/components"), { recursive: true });
    fs.mkdirSync(path.join(fixtureRoot, "types"), { recursive: true });
    fs.mkdirSync(reportDir);
    fs.writeFileSync(
      path.join(fixtureRoot, "packages/common/tsconfig.shared.json"),
      '{\n  // Keep shared root declarations explicit without importing sibling apps.\n  "compilerOptions": { "strict": true, "noEmit": true },\n  "include": ["src/**/*.vue", "../../types/**/*.d.ts"]\n}\n',
    );
    fs.writeFileSync(
      path.join(fixtureRoot, "packages/common/tsconfig.json"),
      `${JSON.stringify({ extends: "./tsconfig.shared.json" })}\n`,
    );
    fs.writeFileSync(
      path.join(fixtureRoot, "packages/common/src/App.vue"),
      '<script setup lang="ts">const value: ROOT_GLOBAL = "ok"</script>\n',
    );
    fs.writeFileSync(path.join(fixtureRoot, "types/globals.d.ts"), "type ROOT_GLOBAL = string;\n");
    fs.writeFileSync(
      path.join(fixtureRoot, "packages/admin/src/components/global-components.d.ts"),
      'import Header from "./Header.vue";\nexport {}\n',
    );
    fs.writeFileSync(
      path.join(fixtureRoot, "packages/admin/src/components/Header.vue"),
      "<template />\n",
    );
    try {
      const project = materializeBaselineProject(
        fixtureRoot,
        reportDir,
        {
          id: "fixture",
          tsconfig: "packages/common/tsconfig.json",
          vueGlobs: ["packages/common/src/**/*.vue", "packages/admin/src/**/*.vue"],
          typecheckPerformance: { corpusGlobs: ["packages/common/src/**/*.vue"] },
        },
        { fileCount: 1, files: [{ file: "packages/common/src/App.vue" }] },
      );
      const config = JSON.parse(project.source);
      assert.deepEqual(config.files, ["../src/App.vue"]);
      assert.equal(config.include.includes("../../../**/*.d.ts"), false);
      assert.equal(config.include.includes("../**/*.d.ts"), true);
      assert.equal(config.include.includes("../src/**/*.d.ts"), true);
      assert.equal(config.include.includes("../../../types/**/*.d.ts"), true);
      assert.equal(config.include.includes("../../admin/src/**/*.d.ts"), false);
      const result = runVueTsc(project.path, fixtureRoot);
      const diagnostics = result.stdout.split("\n").filter((line) => /: error TS\d+: /u.test(line));
      assert.deepEqual(diagnostics, []);
      assert.equal(result.status, 0, result.stderr);
      const program = result.stdout.split("\n").map((line) => line.trimEnd());
      assert.equal(program.includes(path.join(fixtureRoot, "packages/common/src/App.vue")), true);
      assert.equal(program.includes(path.join(fixtureRoot, "types/globals.d.ts")), true);
      assert.equal(
        program.includes(path.join(fixtureRoot, "packages/admin/src/components/Header.vue")),
        false,
      );
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
