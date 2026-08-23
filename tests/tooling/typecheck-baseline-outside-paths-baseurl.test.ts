import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import {
  applyIsolatedAliasOverlay,
  rewriteOutsideAliasPaths,
} from "../../tools/fixtures/typecheck-baseline-outside-aliases.mjs";
import {
  rewriteOutsidePackagePaths,
  writeIsolatedTsconfigOverlay,
} from "../../tools/fixtures/typecheck-baseline-outside-paths.mjs";
import { materializeBaselineProject } from "../../tools/fixtures/typecheck-baseline-project.mjs";

/**
 * Nuxt writes `baseUrl: ".."` on `.nuxt/tsconfig.json` and `paths` relative to
 * that directory. Resolving mappings from the tsconfig file itself treats a
 * fixture-local unique-link as interior and never retargets, while vue-tsc still
 * walks above the fixture (#4461).
 */

function scaffold() {
  const outer = fs.realpathSync(
    fs.mkdtempSync(
      path.join(os.tmpdir(), "vize-baseline-outside-paths-baseurl-"),
    ),
  );
  const fixtureRoot = path.join(outer, "fixture");
  fs.mkdirSync(path.join(fixtureRoot, "src"), { recursive: true });
  const outsideVue = path.join(outer, "node_modules", "vue");
  fs.mkdirSync(outsideVue, { recursive: true });
  fs.writeFileSync(path.join(outsideVue, "package.json"), `{"name":"vue"}\n`);
  const outsideNuxt = path.join(outer, "node_modules", "nuxt");
  fs.mkdirSync(path.join(outsideNuxt, "dist", "app"), { recursive: true });
  fs.writeFileSync(path.join(outsideNuxt, "package.json"), `{"name":"nuxt"}\n`);
  fs.writeFileSync(
    path.join(outsideNuxt, "dist", "app", "index.d.ts"),
    "export {}\n",
  );
  return { fixtureRoot, outer, outsideNuxt, outsideVue };
}

function writeLocalVue(fixtureRoot: string) {
  const local = path.join(fixtureRoot, "node_modules", "vue");
  fs.mkdirSync(local, { recursive: true });
  fs.writeFileSync(path.join(local, "package.json"), `{"name":"vue"}\n`);
  return local;
}

function writeLocalNuxt(fixtureRoot: string) {
  const local = path.join(fixtureRoot, "node_modules", "nuxt");
  fs.mkdirSync(path.join(local, "dist", "app"), { recursive: true });
  fs.writeFileSync(path.join(local, "package.json"), `{"name":"nuxt"}\n`);
  fs.writeFileSync(
    path.join(local, "dist", "app", "index.d.ts"),
    "export {}\n",
  );
  return local;
}

function writeNuxtTsconfig(
  fixtureRoot: string,
  paths: Record<string, string[]>,
) {
  const nuxtDir = path.join(fixtureRoot, ".nuxt");
  fs.mkdirSync(nuxtDir, { recursive: true });
  const sourcePath = path.join(nuxtDir, "tsconfig.json");
  fs.writeFileSync(
    sourcePath,
    `${JSON.stringify({ compilerOptions: { baseUrl: "..", paths } })}\n`,
  );
  return sourcePath;
}

test("a Nuxt baseUrl resolves outside package mappings from the fixture root", () => {
  const { outer, fixtureRoot, outsideVue } = scaffold();
  try {
    writeLocalVue(fixtureRoot);
    const sourcePath = writeNuxtTsconfig(fixtureRoot, {
      "#imports": ["./src/imports.d.ts"],
      vue: [path.relative(fixtureRoot, outsideVue)],
    });
    assert.deepEqual(
      rewriteOutsidePackagePaths(
        fixtureRoot,
        sourcePath,
        path.join(fixtureRoot, ".vize-baseline"),
      ),
      {
        "#imports": ["../src/imports.d.ts"],
        vue: ["../node_modules/vue"],
      },
    );
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});

test("an isolated overlay pins baseUrl so rewritten paths stay beside the source", () => {
  const { outer, fixtureRoot, outsideVue } = scaffold();
  try {
    writeLocalVue(fixtureRoot);
    const sourcePath = writeNuxtTsconfig(fixtureRoot, {
      "#imports": ["./src/imports.d.ts"],
      vue: [path.relative(fixtureRoot, outsideVue)],
    });
    const overlay = writeIsolatedTsconfigOverlay(fixtureRoot, sourcePath);
    assert.equal(
      overlay.path,
      path.join(fixtureRoot, ".nuxt", ".vize-isolated-tsconfig.json"),
    );
    assert.deepEqual(JSON.parse(fs.readFileSync(overlay.path, "utf8")), {
      extends: "./tsconfig.json",
      compilerOptions: {
        baseUrl: ".",
        paths: {
          "#imports": ["../src/imports.d.ts"],
          vue: ["../node_modules/vue"],
        },
      },
    });
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});

test("a Nuxt baseUrl resolves outside hash aliases from the fixture root", () => {
  const { outer, fixtureRoot, outsideNuxt } = scaffold();
  try {
    writeLocalNuxt(fixtureRoot);
    const sourcePath = writeNuxtTsconfig(fixtureRoot, {
      "#app": [
        path.relative(fixtureRoot, path.join(outsideNuxt, "dist", "app")),
      ],
    });
    assert.deepEqual(
      rewriteOutsideAliasPaths(
        fixtureRoot,
        sourcePath,
        path.join(fixtureRoot, ".vize-baseline"),
      ),
      { "#app": ["../node_modules/nuxt/dist/app"] },
    );
    const overlay = applyIsolatedAliasOverlay(fixtureRoot, sourcePath, null);
    assert.deepEqual(JSON.parse(fs.readFileSync(overlay.path, "utf8")), {
      extends: "./tsconfig.json",
      compilerOptions: {
        baseUrl: ".",
        paths: { "#app": ["../node_modules/nuxt/dist/app"] },
      },
    });
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});

test("materialized baseline pins baseUrl when outside paths were retargeted", () => {
  const { outer, fixtureRoot, outsideVue } = scaffold();
  try {
    writeLocalVue(fixtureRoot);
    fs.writeFileSync(path.join(fixtureRoot, "src/App.vue"), "<template />\n");
    writeNuxtTsconfig(fixtureRoot, {
      vue: [path.relative(fixtureRoot, outsideVue)],
    });
    const reportDir = path.join(outer, "report");
    fs.mkdirSync(reportDir);
    const project = materializeBaselineProject(
      fixtureRoot,
      reportDir,
      { id: "fixture", tsconfig: ".nuxt/tsconfig.json" },
      { fileCount: 1, files: [{ file: "src/App.vue" }] },
    );
    const options = JSON.parse(project.source).compilerOptions;
    assert.equal(options.baseUrl, ".");
    assert.deepEqual(options.paths, { vue: ["../../node_modules/vue"] });
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});

test("array extends applies later package paths with an earlier baseUrl", () => {
  const { outer, fixtureRoot, outsideVue } = scaffold();
  try {
    writeLocalVue(fixtureRoot);
    const sourceRoot = path.join(fixtureRoot, "src");
    fs.writeFileSync(
      path.join(fixtureRoot, "base-url.json"),
      `${JSON.stringify({ compilerOptions: { baseUrl: "./src" } })}\n`,
    );
    fs.writeFileSync(
      path.join(fixtureRoot, "paths.json"),
      `${JSON.stringify({
        compilerOptions: {
          paths: { vue: [path.relative(sourceRoot, outsideVue)] },
        },
      })}\n`,
    );
    const sourcePath = path.join(fixtureRoot, "tsconfig.json");
    fs.writeFileSync(
      sourcePath,
      `{ "extends": ["./base-url.json", "./paths.json"] }\n`,
    );
    assert.deepEqual(
      rewriteOutsidePackagePaths(
        fixtureRoot,
        sourcePath,
        path.join(fixtureRoot, ".vize-baseline"),
      ),
      { vue: ["../node_modules/vue"] },
    );
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});

test("array extends applies later hash aliases with an earlier baseUrl", () => {
  const { outer, fixtureRoot, outsideNuxt } = scaffold();
  try {
    writeLocalNuxt(fixtureRoot);
    const sourceRoot = path.join(fixtureRoot, "src");
    fs.writeFileSync(
      path.join(fixtureRoot, "base-url.json"),
      `${JSON.stringify({ compilerOptions: { baseUrl: "./src" } })}\n`,
    );
    fs.writeFileSync(
      path.join(fixtureRoot, "paths.json"),
      `${JSON.stringify({
        compilerOptions: {
          paths: {
            "#app": [
              path.relative(sourceRoot, path.join(outsideNuxt, "dist", "app")),
            ],
          },
        },
      })}\n`,
    );
    const sourcePath = path.join(fixtureRoot, "tsconfig.json");
    fs.writeFileSync(
      sourcePath,
      `{ "extends": ["./base-url.json", "./paths.json"] }\n`,
    );
    assert.deepEqual(
      rewriteOutsideAliasPaths(
        fixtureRoot,
        sourcePath,
        path.join(fixtureRoot, ".vize-baseline"),
      ),
      { "#app": ["../node_modules/nuxt/dist/app"] },
    );
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});
