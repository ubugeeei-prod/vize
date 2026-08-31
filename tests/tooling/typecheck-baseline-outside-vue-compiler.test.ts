import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import { applyIsolatedAliasOverlay } from "../../legacy-tools/fixtures/typecheck-baseline-outside-aliases.mjs";
import { writeIsolatedTsconfigOverlay } from "../../legacy-tools/fixtures/typecheck-baseline-outside-paths.mjs";
import { rewriteOutsideVueCompilerOptions } from "../../legacy-tools/fixtures/typecheck-baseline-outside-vue-compiler.mjs";
import { materializeBaselineProject } from "../../legacy-tools/fixtures/typecheck-baseline-project.mjs";

/**
 * Unique isolation cannot retarget `vueCompilerOptions.globalTypesPath` or
 * `typesRoot`. An outside `@vue/language-core` path lets vue-tsc load Vize's
 * Vue beside the fixture (#4461). Overlay rewrite is the repair, and the
 * vue-tsc baseline must apply the same rewrite so both tools measure one
 * program. Overlay replaces `vueCompilerOptions`, so other keys stay.
 */

function scaffold() {
  const outer = fs.realpathSync(
    fs.mkdtempSync(path.join(os.tmpdir(), "vize-baseline-vue-compiler-")),
  );
  const fixtureRoot = path.join(outer, "fixture");
  fs.mkdirSync(path.join(fixtureRoot, "src"), { recursive: true });
  const outsideCore = path.join(outer, "node_modules", "@vue", "language-core");
  fs.mkdirSync(path.join(outsideCore, "types"), { recursive: true });
  fs.writeFileSync(path.join(outsideCore, "package.json"), `{"name":"@vue/language-core"}\n`);
  fs.writeFileSync(path.join(outsideCore, "vue-global-types.d.ts"), "export {};\n");
  fs.writeFileSync(path.join(outsideCore, "types", "index.d.ts"), "export {};\n");
  return { fixtureRoot, outer, outsideCore };
}

function writeLocalCore(fixtureRoot: string) {
  const local = path.join(fixtureRoot, "node_modules", "@vue", "language-core");
  fs.mkdirSync(path.join(local, "types"), { recursive: true });
  fs.writeFileSync(path.join(local, "package.json"), `{"name":"@vue/language-core"}\n`);
  fs.writeFileSync(path.join(local, "vue-global-types.d.ts"), "export {};\n");
  fs.writeFileSync(path.join(local, "types", "index.d.ts"), "export {};\n");
  return local;
}

test("an outside globalTypesPath is retargeted to the fixture copy and other keys are kept", () => {
  const { fixtureRoot, outer, outsideCore } = scaffold();
  try {
    writeLocalCore(fixtureRoot);
    const sourcePath = path.join(fixtureRoot, "tsconfig.json");
    fs.writeFileSync(
      sourcePath,
      `${JSON.stringify({
        vueCompilerOptions: {
          strictTemplates: true,
          globalTypesPath: path.relative(
            fixtureRoot,
            path.join(outsideCore, "vue-global-types.d.ts"),
          ),
        },
      })}\n`,
    );
    assert.deepEqual(rewriteOutsideVueCompilerOptions(fixtureRoot, sourcePath, fixtureRoot), {
      strictTemplates: true,
      globalTypesPath: "./node_modules/@vue/language-core/vue-global-types.d.ts",
    });
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});

test("a typesRoot package specifier is retargeted to the fixture copy", () => {
  const { fixtureRoot, outer } = scaffold();
  try {
    writeLocalCore(fixtureRoot);
    const sourcePath = path.join(fixtureRoot, "tsconfig.json");
    fs.writeFileSync(
      sourcePath,
      `${JSON.stringify({ vueCompilerOptions: { typesRoot: "@vue/language-core/types" } })}\n`,
    );
    assert.deepEqual(rewriteOutsideVueCompilerOptions(fixtureRoot, sourcePath, fixtureRoot), {
      typesRoot: "./node_modules/@vue/language-core/types",
    });
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});

test("vueCompilerOptions reached only through relative extends are still retargeted", () => {
  const { fixtureRoot, outer, outsideCore } = scaffold();
  try {
    writeLocalCore(fixtureRoot);
    fs.writeFileSync(
      path.join(fixtureRoot, "tsconfig.app.json"),
      `${JSON.stringify({
        vueCompilerOptions: {
          globalTypesPath: path.relative(
            fixtureRoot,
            path.join(outsideCore, "vue-global-types.d.ts"),
          ),
        },
      })}\n`,
    );
    const sourcePath = path.join(fixtureRoot, "tsconfig.check.json");
    fs.writeFileSync(sourcePath, `{ "extends": "./tsconfig.app.json" }\n`);
    const overlay = applyIsolatedAliasOverlay(
      fixtureRoot,
      sourcePath,
      writeIsolatedTsconfigOverlay(fixtureRoot, sourcePath),
    );
    assert.equal(overlay.path, path.join(fixtureRoot, ".vize-isolated-tsconfig.check.json"));
    assert.deepEqual(JSON.parse(fs.readFileSync(overlay.path, "utf8")), {
      extends: "./tsconfig.check.json",
      vueCompilerOptions: {
        globalTypesPath: "./node_modules/@vue/language-core/vue-global-types.d.ts",
      },
    });
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});

test("an inside globalTypesPath is left for inheritance", () => {
  const { fixtureRoot, outer } = scaffold();
  try {
    writeLocalCore(fixtureRoot);
    const sourcePath = path.join(fixtureRoot, "tsconfig.json");
    fs.writeFileSync(
      sourcePath,
      `${JSON.stringify({
        vueCompilerOptions: {
          globalTypesPath: "./node_modules/@vue/language-core/vue-global-types.d.ts",
        },
      })}\n`,
    );
    assert.equal(rewriteOutsideVueCompilerOptions(fixtureRoot, sourcePath, fixtureRoot), null);
    assert.equal(applyIsolatedAliasOverlay(fixtureRoot, sourcePath, null), null);
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});

test("an outside globalTypesPath is left alone when the fixture has no local package", () => {
  const { fixtureRoot, outer, outsideCore } = scaffold();
  try {
    const sourcePath = path.join(fixtureRoot, "tsconfig.json");
    fs.writeFileSync(
      sourcePath,
      `${JSON.stringify({
        vueCompilerOptions: {
          globalTypesPath: path.relative(
            fixtureRoot,
            path.join(outsideCore, "vue-global-types.d.ts"),
          ),
        },
      })}\n`,
    );
    assert.equal(rewriteOutsideVueCompilerOptions(fixtureRoot, sourcePath, fixtureRoot), null);
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});

test("alias overlay keeps vueCompilerOptions when it also rewrites #app", () => {
  const { fixtureRoot, outer, outsideCore } = scaffold();
  try {
    writeLocalCore(fixtureRoot);
    const outsideNuxt = path.join(outer, "node_modules", "nuxt");
    fs.mkdirSync(path.join(outsideNuxt, "dist", "app"), { recursive: true });
    fs.writeFileSync(path.join(outsideNuxt, "package.json"), `{"name":"nuxt"}\n`);
    const localNuxt = path.join(fixtureRoot, "node_modules", "nuxt");
    fs.mkdirSync(path.join(localNuxt, "dist", "app"), { recursive: true });
    fs.writeFileSync(path.join(localNuxt, "package.json"), `{"name":"nuxt"}\n`);
    const sourcePath = path.join(fixtureRoot, "tsconfig.json");
    fs.writeFileSync(
      sourcePath,
      `${JSON.stringify({
        compilerOptions: {
          paths: { "#app": [path.relative(fixtureRoot, path.join(outsideNuxt, "dist", "app"))] },
        },
        vueCompilerOptions: {
          globalTypesPath: path.relative(
            fixtureRoot,
            path.join(outsideCore, "vue-global-types.d.ts"),
          ),
        },
      })}\n`,
    );
    const overlay = applyIsolatedAliasOverlay(
      fixtureRoot,
      sourcePath,
      writeIsolatedTsconfigOverlay(fixtureRoot, sourcePath),
    );
    assert.deepEqual(JSON.parse(fs.readFileSync(overlay.path, "utf8")), {
      extends: "./tsconfig.json",
      compilerOptions: { paths: { "#app": ["./node_modules/nuxt/dist/app"] } },
      vueCompilerOptions: {
        globalTypesPath: "./node_modules/@vue/language-core/vue-global-types.d.ts",
      },
    });
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});

test("materialized baseline retargets outside globalTypesPath with the overlay", () => {
  const { outer, fixtureRoot, outsideCore } = scaffold();
  try {
    writeLocalCore(fixtureRoot);
    fs.writeFileSync(path.join(fixtureRoot, "src/App.vue"), "<template />\n");
    const sourcePath = path.join(fixtureRoot, "tsconfig.json");
    fs.writeFileSync(
      sourcePath,
      `${JSON.stringify({
        vueCompilerOptions: {
          globalTypesPath: path.relative(
            fixtureRoot,
            path.join(outsideCore, "vue-global-types.d.ts"),
          ),
        },
      })}\n`,
    );
    const reportDir = path.join(outer, "report");
    fs.mkdirSync(reportDir);
    const project = materializeBaselineProject(
      fixtureRoot,
      reportDir,
      { id: "fixture", tsconfig: "tsconfig.json" },
      { fileCount: 1, files: [{ file: "src/App.vue" }] },
    );
    const parsed = JSON.parse(project.source);
    assert.deepEqual(parsed.vueCompilerOptions, {
      globalTypesPath: "../node_modules/@vue/language-core/vue-global-types.d.ts",
    });
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});
