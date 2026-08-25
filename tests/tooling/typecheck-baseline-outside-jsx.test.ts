import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import {
  applyIsolatedJsxBaseline,
  applyIsolatedJsxOverlay,
  rewriteOutsideJsxImportSource,
} from "../../tools/fixtures/typecheck-baseline-outside-jsx.mjs";
import { applyIsolatedTypecheckOverlays } from "../../tools/fixtures/typecheck-baseline-outside-overlays.mjs";

/**
 * Unique isolation cannot retarget `jsxImportSource: "../../node_modules/vue"`
 * (#4461). Package-name sources stay with unique-link. Overlay rewrite is the
 * repair for path specifiers, and the vue-tsc baseline must apply the same
 * rewrite so both tools measure one program.
 */

function scaffold() {
  const outer = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), "vize-baseline-jsx-")));
  const fixtureRoot = path.join(outer, "fixture");
  fs.mkdirSync(path.join(fixtureRoot, "src"), { recursive: true });
  const outsideVue = path.join(outer, "node_modules", "vue");
  fs.mkdirSync(outsideVue, { recursive: true });
  fs.writeFileSync(path.join(outsideVue, "package.json"), `{"name":"vue"}\n`);
  return { fixtureRoot, outer, outsideVue };
}

function writeLocalVue(fixtureRoot: string) {
  const local = path.join(fixtureRoot, "node_modules", "vue");
  fs.mkdirSync(local, { recursive: true });
  fs.writeFileSync(path.join(local, "package.json"), `{"name":"vue"}\n`);
  return local;
}

test("an outside jsxImportSource path is retargeted to the fixture copy", () => {
  const { fixtureRoot, outer, outsideVue } = scaffold();
  try {
    writeLocalVue(fixtureRoot);
    const sourcePath = path.join(fixtureRoot, "tsconfig.json");
    fs.writeFileSync(
      sourcePath,
      `${JSON.stringify({
        compilerOptions: { jsxImportSource: path.relative(fixtureRoot, outsideVue) },
      })}\n`,
    );
    assert.equal(
      rewriteOutsideJsxImportSource(fixtureRoot, sourcePath, fixtureRoot),
      "./node_modules/vue",
    );
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});

test("a package-name jsxImportSource is left for unique isolation", () => {
  const { fixtureRoot, outer } = scaffold();
  try {
    writeLocalVue(fixtureRoot);
    const sourcePath = path.join(fixtureRoot, "tsconfig.json");
    fs.writeFileSync(
      sourcePath,
      `${JSON.stringify({ compilerOptions: { jsxImportSource: "vue" } })}\n`,
    );
    assert.equal(rewriteOutsideJsxImportSource(fixtureRoot, sourcePath, fixtureRoot), null);
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});

test("an inside jsxImportSource path is left for inheritance", () => {
  const { fixtureRoot, outer } = scaffold();
  try {
    writeLocalVue(fixtureRoot);
    const sourcePath = path.join(fixtureRoot, "tsconfig.json");
    fs.writeFileSync(
      sourcePath,
      `${JSON.stringify({
        compilerOptions: { jsxImportSource: "./node_modules/vue" },
      })}\n`,
    );
    assert.equal(rewriteOutsideJsxImportSource(fixtureRoot, sourcePath, fixtureRoot), null);
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});

test("an outside jsxImportSource is left alone when the fixture has no local package", () => {
  const { fixtureRoot, outer, outsideVue } = scaffold();
  try {
    const sourcePath = path.join(fixtureRoot, "tsconfig.json");
    fs.writeFileSync(
      sourcePath,
      `${JSON.stringify({
        compilerOptions: { jsxImportSource: path.relative(fixtureRoot, outsideVue) },
      })}\n`,
    );
    assert.equal(rewriteOutsideJsxImportSource(fixtureRoot, sourcePath, fixtureRoot), null);
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});

test("child jsxImportSource replaces a parent path specifier", () => {
  const { fixtureRoot, outer, outsideVue } = scaffold();
  try {
    writeLocalVue(fixtureRoot);
    fs.writeFileSync(
      path.join(fixtureRoot, "tsconfig.app.json"),
      `${JSON.stringify({
        compilerOptions: { jsxImportSource: path.relative(fixtureRoot, outsideVue) },
      })}\n`,
    );
    const sourcePath = path.join(fixtureRoot, "tsconfig.check.json");
    fs.writeFileSync(
      sourcePath,
      `${JSON.stringify({
        extends: "./tsconfig.app.json",
        compilerOptions: { jsxImportSource: "preact" },
      })}\n`,
    );
    assert.equal(rewriteOutsideJsxImportSource(fixtureRoot, sourcePath, fixtureRoot), null);
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});

test("overlay write keeps existing compilerOptions and vueCompilerOptions", () => {
  const { fixtureRoot, outer, outsideVue } = scaffold();
  try {
    writeLocalVue(fixtureRoot);
    const sourcePath = path.join(fixtureRoot, "tsconfig.json");
    fs.writeFileSync(
      sourcePath,
      `${JSON.stringify({
        compilerOptions: { jsxImportSource: path.relative(fixtureRoot, outsideVue) },
      })}\n`,
    );
    const overlayPath = path.join(fixtureRoot, ".vize-isolated-tsconfig.json");
    fs.writeFileSync(
      overlayPath,
      `${JSON.stringify({
        extends: "./tsconfig.json",
        compilerOptions: { paths: { vue: ["./node_modules/vue"] } },
        vueCompilerOptions: { strictTemplates: true },
      })}\n`,
    );
    const overlay = applyIsolatedJsxOverlay(fixtureRoot, sourcePath, { path: overlayPath });
    assert.equal(overlay.path, overlayPath);
    assert.deepEqual(JSON.parse(fs.readFileSync(overlayPath, "utf8")), {
      extends: "./tsconfig.json",
      compilerOptions: {
        paths: { vue: ["./node_modules/vue"] },
        jsxImportSource: "./node_modules/vue",
      },
      vueCompilerOptions: { strictTemplates: true },
    });
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});

test("shared overlay helper writes jsxImportSource when path overlay is empty", () => {
  const { fixtureRoot, outer, outsideVue } = scaffold();
  try {
    writeLocalVue(fixtureRoot);
    const sourcePath = path.join(fixtureRoot, "tsconfig.json");
    fs.writeFileSync(
      sourcePath,
      `${JSON.stringify({
        compilerOptions: { jsxImportSource: path.relative(fixtureRoot, outsideVue) },
      })}\n`,
    );
    const overlay = applyIsolatedTypecheckOverlays(fixtureRoot, sourcePath);
    assert.equal(overlay.path, path.join(fixtureRoot, ".vize-isolated-tsconfig.json"));
    assert.deepEqual(JSON.parse(fs.readFileSync(overlay.path, "utf8")), {
      extends: "./tsconfig.json",
      compilerOptions: {
        paths: { vue: ["./node_modules/vue"] },
        jsxImportSource: "./node_modules/vue",
      },
    });
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});

test("baseline post-process retargets jsxImportSource relative to the generated config", () => {
  const { fixtureRoot, outer, outsideVue } = scaffold();
  try {
    writeLocalVue(fixtureRoot);
    const sourcePath = path.join(fixtureRoot, "tsconfig.json");
    fs.writeFileSync(
      sourcePath,
      `${JSON.stringify({
        compilerOptions: { jsxImportSource: path.relative(fixtureRoot, outsideVue) },
      })}\n`,
    );
    const baselineDir = path.join(fixtureRoot, ".vize-baseline");
    fs.mkdirSync(baselineDir);
    const baselinePath = path.join(baselineDir, "tsconfig.json");
    fs.writeFileSync(
      baselinePath,
      `${JSON.stringify({
        extends: "../tsconfig.json",
        compilerOptions: { rootDir: ".." },
      })}\n`,
    );
    assert.equal(
      applyIsolatedJsxBaseline(fixtureRoot, sourcePath, baselinePath),
      "../node_modules/vue",
    );
    assert.deepEqual(JSON.parse(fs.readFileSync(baselinePath, "utf8")), {
      extends: "../tsconfig.json",
      compilerOptions: { rootDir: "..", jsxImportSource: "../node_modules/vue" },
    });
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});
