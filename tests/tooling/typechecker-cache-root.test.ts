import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import {
  cachePathContains,
  inferCacheSemanticRoot,
  rootOutsideOwnedNamespace,
} from "../../tools/benchmarks/scripts/typechecker-cache-root.ts";

void test("actual explicit generated include roots qualify only when every primary input is listed", () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-native-include-control-"));
  try {
    const project = path.join(root, "project"),
      virtual = path.join(root, "mirror");
    fs.mkdirSync(path.join(project, "src"), { recursive: true });
    fs.mkdirSync(path.join(virtual, "src"), { recursive: true });
    const entries = [{ path: "src/First.vue" }, { path: "src/Second.vue" }];
    for (const entry of entries) {
      fs.writeFileSync(path.join(project, entry.path), "<template/>");
      fs.writeFileSync(path.join(virtual, entry.path + ".ts"), "export {};");
    }
    assert.equal(rootOutsideOwnedNamespace(project, virtual), true);
    assert.equal(rootOutsideOwnedNamespace(virtual, virtual), false);
    assert.equal(rootOutsideOwnedNamespace(root, virtual), false);
    assert.equal(
      rootOutsideOwnedNamespace(path.join(virtual, "src"), virtual),
      false,
      "a semantic root nested under the cache must fail independently of metadata/mirror equality",
    );
    assert.equal(
      cachePathContains(root, path.join(root, "..cache")),
      true,
      "a child segment starting with two dots is not parent traversal",
    );
    const request = {
      projectRoot: project,
      entries,
      virtualRoot: virtual,
      configPath: path.join(virtual, "tsconfig.json"),
      config: { include: entries.map((entry) => entry.path + ".ts") },
    };
    assert.equal(inferCacheSemanticRoot(request).qualified, true);
    assert.equal(
      inferCacheSemanticRoot({ ...request, config: { include: ["src/First.vue.ts"] } }).qualified,
      false,
      "an unlisted primary input cannot be admitted only because its mirror exists",
    );
    assert.equal(
      inferCacheSemanticRoot({ ...request, config: { include: ["src/*.vue.ts"] } }).qualified,
      false,
      "this observer does not pretend an authored glob is an exact generated-root census",
    );
    assert.equal(
      inferCacheSemanticRoot({ ...request, config: { files: request.config.include } }).qualified,
      true,
    );
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

void test("workspace dependency SFCs stay in the census without becoming independent roots", () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-dependency-root-control-"));
  try {
    const primary = path.join(root, "app"),
      virtual = path.join(root, "mirror");
    fs.mkdirSync(path.join(primary, "src"), { recursive: true });
    fs.mkdirSync(path.join(root, "packages/widget"), { recursive: true });
    fs.mkdirSync(path.join(virtual, "src"), { recursive: true });
    fs.writeFileSync(path.join(primary, "src/App.vue"), "<template/>");
    fs.writeFileSync(path.join(primary, "src/Control.vue"), "<template/>");
    fs.writeFileSync(path.join(root, "packages/widget/Widget.vue"), "<template/>");
    for (const name of ["App", "Control"])
      fs.writeFileSync(path.join(virtual, "src", name + ".vue.ts"), "export {};");
    const input = {
      projectRoot: root,
      primaryRoot: primary,
      entries: [
        { path: "app/src/App.vue" },
        { path: "app/src/Control.vue" },
        { path: "packages/widget/Widget.vue" },
      ],
      virtualRoot: virtual,
      configPath: path.join(virtual, "tsconfig.json"),
      config: { include: ["src/App.vue.ts", "src/Control.vue.ts"] },
    };
    const witness = inferCacheSemanticRoot(input);
    assert.equal(witness.qualified, true);
    assert.equal(witness.sourceFiles, 3);
    assert.equal(witness.primaryFiles, 2);
    assert.equal(witness.dependencyFiles, 1);
    assert.equal(witness.dependencies[0].path, "packages/widget/Widget.vue");
    assert.equal(witness.root, fs.realpathSync(primary));
    assert.equal(witness.mappings.length, 2);
    assert.equal(
      inferCacheSemanticRoot({ ...input, config: { include: ["src/App.vue.ts"] } }).qualified,
      false,
      "a missing configured primary input cannot be reclassified as a dependency by the inferred mirror root",
    );
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});
