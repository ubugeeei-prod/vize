import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import { isolateUniqueVueEditorPackages } from "../../tools/fixtures/typecheck-baseline-isolation-unique.mjs";

/**
 * TipTap Vue and Vue Flow live in Vize's `tests/package.json`. TypeScript can
 * climb into those copies and load Vize's Vue beside the fixture (#4461).
 * Unique isolation links the fixture store copy when an ancestor is reachable.
 */

function scaffold(names: string[]) {
  const outer = fs.realpathSync(
    fs.mkdtempSync(path.join(os.tmpdir(), "vize-isolation-vue-editor-")),
  );
  const fixtureRoot = path.join(outer, "fixture");
  fs.mkdirSync(fixtureRoot, { recursive: true });
  for (const name of names) {
    const packageRoot = path.join(outer, "node_modules", name);
    fs.mkdirSync(packageRoot, { recursive: true });
    fs.writeFileSync(path.join(packageRoot, "package.json"), `{"name":"${name}"}\n`);
  }
  return { fixtureRoot, outer };
}

function writeStoreCopy(fixtureRoot: string, id: string, name: string) {
  const packageRoot = path.join(fixtureRoot, "node_modules", ".pnpm", id, "node_modules", name);
  fs.mkdirSync(packageRoot, { recursive: true });
  fs.writeFileSync(path.join(packageRoot, "package.json"), `{"name":"${name}"}\n`);
  return packageRoot;
}

test("ancestor editor packages with one in-fixture copy each are linked from those copies", () => {
  const { fixtureRoot, outer } = scaffold(["@tiptap/vue-3", "@vue-flow/core"]);
  try {
    writeStoreCopy(fixtureRoot, "@tiptap+vue-3@3.26.0", "@tiptap/vue-3");
    writeStoreCopy(fixtureRoot, "@vue-flow+core@1.48.2", "@vue-flow/core");
    assert.deepEqual(isolateUniqueVueEditorPackages(fixtureRoot), [
      {
        name: "@tiptap/vue-3",
        target: "node_modules/.pnpm/@tiptap+vue-3@3.26.0/node_modules/@tiptap/vue-3",
      },
      {
        name: "@vue-flow/core",
        target: "node_modules/.pnpm/@vue-flow+core@1.48.2/node_modules/@vue-flow/core",
      },
    ]);
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});

test("editor packages no ancestor provides are left alone", () => {
  const { fixtureRoot, outer } = scaffold([]);
  try {
    writeStoreCopy(fixtureRoot, "@tiptap+vue-3@3.26.0", "@tiptap/vue-3");
    writeStoreCopy(fixtureRoot, "@vue-flow+core@1.48.2", "@vue-flow/core");
    assert.deepEqual(isolateUniqueVueEditorPackages(fixtureRoot), []);
    assert.equal(fs.existsSync(path.join(fixtureRoot, "node_modules", "@tiptap", "vue-3")), false);
    assert.equal(fs.existsSync(path.join(fixtureRoot, "node_modules", "@vue-flow", "core")), false);
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});

test("an already hoisted @tiptap/vue-3 is left for isolation; this repair does not relink it", () => {
  const { fixtureRoot, outer } = scaffold(["@tiptap/vue-3"]);
  try {
    writeStoreCopy(fixtureRoot, "@tiptap+vue-3@3.26.0", "@tiptap/vue-3");
    const hoisted = path.join(fixtureRoot, "node_modules", "@tiptap", "vue-3");
    fs.mkdirSync(hoisted, { recursive: true });
    fs.writeFileSync(path.join(hoisted, "package.json"), `{"name":"@tiptap/vue-3"}\n`);
    assert.deepEqual(isolateUniqueVueEditorPackages(fixtureRoot), []);
    assert.equal(fs.lstatSync(hoisted).isSymbolicLink(), false);
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});
