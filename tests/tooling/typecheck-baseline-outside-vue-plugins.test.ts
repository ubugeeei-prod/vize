import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import { applyIsolatedAliasOverlay } from "../../tools/fixtures/typecheck-baseline-outside-aliases.mjs";
import { writeIsolatedTsconfigOverlay } from "../../tools/fixtures/typecheck-baseline-outside-paths.mjs";
import { rewriteOutsideVueCompilerOptions } from "../../tools/fixtures/typecheck-baseline-outside-vue-compiler.mjs";
import { materializeBaselineProject } from "../../tools/fixtures/typecheck-baseline-project.mjs";

/**
 * Unique isolation cannot retarget `require("../../node_modules/<pkg>")`
 * from `vueCompilerOptions.plugins` (#4461). Package-name plugins stay with
 * unique-link. Overlay rewrite is the repair for path specifiers, and the
 * vue-tsc baseline must apply the same rewrite so both tools measure one
 * program.
 */

function scaffold() {
  const outer = fs.realpathSync(
    fs.mkdtempSync(path.join(os.tmpdir(), "vize-baseline-vue-plugins-")),
  );
  const fixtureRoot = path.join(outer, "fixture");
  fs.mkdirSync(path.join(fixtureRoot, "src"), { recursive: true });
  const outsidePug = path.join(outer, "node_modules", "@vue", "language-plugin-pug");
  fs.mkdirSync(outsidePug, { recursive: true });
  fs.writeFileSync(path.join(outsidePug, "package.json"), `{"name":"@vue/language-plugin-pug"}\n`);
  const outsideCore = path.join(outer, "node_modules", "@vue", "language-core");
  fs.mkdirSync(path.join(outsideCore, "types"), { recursive: true });
  fs.writeFileSync(path.join(outsideCore, "package.json"), `{"name":"@vue/language-core"}\n`);
  fs.writeFileSync(path.join(outsideCore, "vue-global-types.d.ts"), "export {};\n");
  return { fixtureRoot, outer, outsideCore, outsidePug };
}

function writeLocalPug(fixtureRoot: string) {
  const local = path.join(fixtureRoot, "node_modules", "@vue", "language-plugin-pug");
  fs.mkdirSync(local, { recursive: true });
  fs.writeFileSync(path.join(local, "package.json"), `{"name":"@vue/language-plugin-pug"}\n`);
  return local;
}

function writeLocalCore(fixtureRoot: string) {
  const local = path.join(fixtureRoot, "node_modules", "@vue", "language-core");
  fs.mkdirSync(local, { recursive: true });
  fs.writeFileSync(path.join(local, "package.json"), `{"name":"@vue/language-core"}\n`);
  fs.writeFileSync(path.join(local, "vue-global-types.d.ts"), "export {};\n");
  return local;
}

test("an outside relative plugin path is retargeted to the fixture copy", () => {
  const { fixtureRoot, outer, outsidePug } = scaffold();
  try {
    writeLocalPug(fixtureRoot);
    const sourcePath = path.join(fixtureRoot, "tsconfig.json");
    fs.writeFileSync(
      sourcePath,
      `${JSON.stringify({
        vueCompilerOptions: {
          strictTemplates: true,
          plugins: [path.relative(fixtureRoot, outsidePug)],
        },
      })}\n`,
    );
    assert.deepEqual(rewriteOutsideVueCompilerOptions(fixtureRoot, sourcePath, fixtureRoot), {
      strictTemplates: true,
      plugins: ["./node_modules/@vue/language-plugin-pug"],
    });
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});

test("an outside Windows relative plugin path is retargeted to the fixture copy", () => {
  const { fixtureRoot, outer, outsidePug } = scaffold();
  try {
    writeLocalPug(fixtureRoot);
    const sourcePath = path.join(fixtureRoot, "tsconfig.json");
    fs.writeFileSync(
      sourcePath,
      `${JSON.stringify({
        vueCompilerOptions: {
          plugins: [path.relative(fixtureRoot, outsidePug).replaceAll("/", "\\")],
        },
      })}\n`,
    );
    assert.deepEqual(rewriteOutsideVueCompilerOptions(fixtureRoot, sourcePath, fixtureRoot), {
      plugins: ["./node_modules/@vue/language-plugin-pug"],
    });
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});

test("an outside object plugin name is retargeted", () => {
  const { fixtureRoot, outer, outsidePug } = scaffold();
  try {
    writeLocalPug(fixtureRoot);
    const sourcePath = path.join(fixtureRoot, "tsconfig.json");
    fs.writeFileSync(
      sourcePath,
      `${JSON.stringify({
        vueCompilerOptions: {
          plugins: [{ pretty: true, name: path.relative(fixtureRoot, outsidePug) }],
        },
      })}\n`,
    );
    assert.deepEqual(rewriteOutsideVueCompilerOptions(fixtureRoot, sourcePath, fixtureRoot), {
      plugins: [{ pretty: true, name: "./node_modules/@vue/language-plugin-pug" }],
    });
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});

test("an outside tuple plugin name is retargeted", () => {
  const { fixtureRoot, outer, outsidePug } = scaffold();
  try {
    writeLocalPug(fixtureRoot);
    const sourcePath = path.join(fixtureRoot, "tsconfig.json");
    fs.writeFileSync(
      sourcePath,
      `${JSON.stringify({
        vueCompilerOptions: {
          plugins: [[path.relative(fixtureRoot, outsidePug), { pretty: true }]],
        },
      })}\n`,
    );
    assert.deepEqual(rewriteOutsideVueCompilerOptions(fixtureRoot, sourcePath, fixtureRoot), {
      plugins: [["./node_modules/@vue/language-plugin-pug", { pretty: true }]],
    });
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});

test("a package-name plugin is left for unique isolation", () => {
  const { fixtureRoot, outer } = scaffold();
  try {
    writeLocalPug(fixtureRoot);
    const sourcePath = path.join(fixtureRoot, "tsconfig.json");
    fs.writeFileSync(
      sourcePath,
      `${JSON.stringify({
        vueCompilerOptions: { plugins: ["@vue/language-plugin-pug"] },
      })}\n`,
    );
    assert.equal(rewriteOutsideVueCompilerOptions(fixtureRoot, sourcePath, fixtureRoot), null);
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});

test("an inside plugin path is left for inheritance", () => {
  const { fixtureRoot, outer } = scaffold();
  try {
    writeLocalPug(fixtureRoot);
    const sourcePath = path.join(fixtureRoot, "tsconfig.json");
    fs.writeFileSync(
      sourcePath,
      `${JSON.stringify({
        vueCompilerOptions: { plugins: ["./node_modules/@vue/language-plugin-pug"] },
      })}\n`,
    );
    assert.equal(rewriteOutsideVueCompilerOptions(fixtureRoot, sourcePath, fixtureRoot), null);
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});

test("an outside plugin path is left alone when the fixture has no local package", () => {
  const { fixtureRoot, outer, outsidePug } = scaffold();
  try {
    const sourcePath = path.join(fixtureRoot, "tsconfig.json");
    fs.writeFileSync(
      sourcePath,
      `${JSON.stringify({
        vueCompilerOptions: { plugins: [path.relative(fixtureRoot, outsidePug)] },
      })}\n`,
    );
    assert.equal(rewriteOutsideVueCompilerOptions(fixtureRoot, sourcePath, fixtureRoot), null);
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});

test("relative extends concatenates parent plugin paths onto the overlay", () => {
  const { fixtureRoot, outer, outsidePug } = scaffold();
  try {
    writeLocalPug(fixtureRoot);
    fs.writeFileSync(
      path.join(fixtureRoot, "tsconfig.app.json"),
      `${JSON.stringify({
        vueCompilerOptions: { plugins: [path.relative(fixtureRoot, outsidePug)] },
      })}\n`,
    );
    const sourcePath = path.join(fixtureRoot, "tsconfig.check.json");
    fs.writeFileSync(
      sourcePath,
      `${JSON.stringify({
        extends: "./tsconfig.app.json",
        vueCompilerOptions: { strictTemplates: true },
      })}\n`,
    );
    const overlay = applyIsolatedAliasOverlay(
      fixtureRoot,
      sourcePath,
      writeIsolatedTsconfigOverlay(fixtureRoot, sourcePath),
    );
    assert.equal(overlay.path, path.join(fixtureRoot, ".vize-isolated-tsconfig.check.json"));
    assert.deepEqual(JSON.parse(fs.readFileSync(overlay.path, "utf8")), {
      extends: "./tsconfig.check.json",
      vueCompilerOptions: {
        strictTemplates: true,
        plugins: ["./node_modules/@vue/language-plugin-pug"],
      },
    });
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});

test("alias overlay keeps globalTypesPath when it also rewrites plugins", () => {
  const { fixtureRoot, outer, outsideCore, outsidePug } = scaffold();
  try {
    writeLocalPug(fixtureRoot);
    writeLocalCore(fixtureRoot);
    const sourcePath = path.join(fixtureRoot, "tsconfig.json");
    fs.writeFileSync(
      sourcePath,
      `${JSON.stringify({
        vueCompilerOptions: {
          globalTypesPath: path.relative(
            fixtureRoot,
            path.join(outsideCore, "vue-global-types.d.ts"),
          ),
          plugins: [path.relative(fixtureRoot, outsidePug)],
        },
      })}\n`,
    );
    const overlay = applyIsolatedAliasOverlay(fixtureRoot, sourcePath, null);
    assert.deepEqual(JSON.parse(fs.readFileSync(overlay.path, "utf8")), {
      extends: "./tsconfig.json",
      vueCompilerOptions: {
        globalTypesPath: "./node_modules/@vue/language-core/vue-global-types.d.ts",
        plugins: ["./node_modules/@vue/language-plugin-pug"],
      },
    });
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});

test("materialized baseline retargets outside plugin paths with the overlay", () => {
  const { outer, fixtureRoot, outsidePug } = scaffold();
  try {
    writeLocalPug(fixtureRoot);
    fs.writeFileSync(path.join(fixtureRoot, "src/App.vue"), "<template />\n");
    const sourcePath = path.join(fixtureRoot, "tsconfig.json");
    fs.writeFileSync(
      sourcePath,
      `${JSON.stringify({
        vueCompilerOptions: { plugins: [path.relative(fixtureRoot, outsidePug)] },
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
      plugins: ["../node_modules/@vue/language-plugin-pug"],
    });
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});
