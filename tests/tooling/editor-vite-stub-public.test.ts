import assert from "node:assert/strict";
import {
  mkdtempSync,
  mkdirSync,
  readFileSync,
  realpathSync,
  rmSync,
  symlinkSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";
import { sha256 } from "../../tools/support/config/windows_consumer/common.ts";
import {
  inventory,
  writeFixture,
} from "../../tools/support/release/editor_vite_stub/filesystem.ts";
import { injectLink } from "../../tools/support/release/editor_vite_stub/editor.ts";

function fixture(run: (root: string) => void): void {
  const root = realpathSync(mkdtempSync(join(tmpdir(), "editor-custody-control-")));
  try {
    run(root);
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
}
test("full installation inventory detects same-size byte mutation and link-only retargeting", () =>
  fixture((root) => {
    mkdirSync(join(root, "pkg"));
    mkdirSync(join(root, "other"));
    writeFileSync(join(root, "pkg/index.js"), "export const value = 7;\n");
    writeFileSync(join(root, "other/index.js"), "export const value = 7;\n");
    const link = join(root, "alias");
    const kind = process.platform === "win32" ? "junction" : "dir";
    symlinkSync(join(root, "pkg"), link, kind);
    const before = inventory(root);
    writeFileSync(join(root, "pkg/index.js"), "export const value = 8;\n");
    assert.throws(() => assert.deepEqual(inventory(root), before));
    writeFileSync(join(root, "pkg/index.js"), "export const value = 7;\n");
    assert.deepEqual(inventory(root), before);
    rmSync(link);
    symlinkSync(join(root, "other"), link, kind);
    assert.throws(() => assert.deepEqual(inventory(root), before));
  }));
test("stale whole-link injection uses an actual Windows junction and preserves the external tree", () =>
  fixture((root) => {
    const temporary = join(root, "private"),
      mirror = join(temporary, "mirror"),
      modules = join(root, "external/node_modules");
    mkdirSync(join(mirror, "node_modules"), { recursive: true });
    mkdirSync(modules, { recursive: true });
    writeFileSync(join(modules, "sentinel"), "external byte custody\n");
    const before = inventory(modules);
    const injected = injectLink(mirror, temporary, modules);
    assert.equal(injected.kind, process.platform === "win32" ? "junction" : "dir");
    assert.equal(injected.resolved, modules);
    assert.deepEqual(inventory(modules), before);
    // This is a falsification control for the observer, not an imitation of the product repair.
    writeFileSync(join(mirror, "node_modules/sentinel"), "unintended write\n");
    assert.throws(() => assert.deepEqual(inventory(modules), before));
    assert.throws(() => injectLink(mirror, temporary, modules), /only an owned mirror directory/);
    assert.throws(() => injectLink(join(root, "external"), temporary, modules));
  }));
test("reported fluent config, app, declaration stub and installation controls remain immutable", () =>
  fixture((root) => {
    const directory = new URL(
      "../_fixtures/differential/typechecker/editor-vite-stub-11145/",
      import.meta.url,
    );
    const project = readFileSync(new URL("project.json", directory));
    const installation = readFileSync(new URL("installation.json", directory));
    assert.equal(
      sha256(project),
      "8e13ee81f8a0086e34e44f2ec49f7dc6ecde19d55694a75bf3415b212d120f5a",
    );
    assert.equal(
      sha256(installation),
      "cf67d99370a6339ed1144f22b9fa84f71a15ef7a67a809a7a4fa72462dc83daf",
    );
    writeFixture(root, { "src/app.vue": "original\n" }, () => true);
    assert.equal(readFileSync(join(root, "src/app.vue"), "utf8"), "original\n");
    assert.throws(() => writeFixture(root, { "../escape": "bad" }, () => true), /fixture escaped/);
  }));
