import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import { generateWorkspace } from "../../tools/benchmarks/scripts/check-workspace.ts";

const SHAPE = {
  packages: 8,
  modulesPerPackage: 3,
  depsPerPackage: 2,
  vuePackages: 2,
  appComponents: 6,
  importsPerComponent: 3,
};

function listing(root: string): string[] {
  const files: string[] = [];
  const walk = (dir: string) => {
    for (const entry of fs
      .readdirSync(dir, { withFileTypes: true })
      .sort((a, b) => a.name.localeCompare(b.name))) {
      const full = path.join(dir, entry.name);
      if (entry.isSymbolicLink())
        files.push(`${path.relative(root, full)} -> ${fs.readlinkSync(full)}`);
      else if (entry.isDirectory()) walk(full);
      else files.push(`${path.relative(root, full)} ${fs.readFileSync(full, "utf8").length}`);
    }
  };
  walk(root);
  return files;
}

test("the #9727 workspace fixture is deterministic and links packages pnpm-style", () => {
  const base = fs.mkdtempSync(path.join(os.tmpdir(), "vize-bench-check-workspace-"));
  try {
    const links = { vue: base, typescript: base };
    const app = generateWorkspace(path.join(base, "one"), SHAPE, links);
    generateWorkspace(path.join(base, "two"), SHAPE, links);
    assert.deepEqual(listing(path.join(base, "one")), listing(path.join(base, "two")));

    // Every workspace link resolves to a generated package manifest.
    const appLinks = path.join(app, "node_modules", "@ws");
    assert.equal(fs.readdirSync(appLinks).length, SHAPE.packages);
    for (const name of fs.readdirSync(appLinks)) {
      assert.ok(fs.existsSync(path.join(appLinks, name, "package.json")), name);
    }
    const nested = path.join(base, "one", "packages", "pkg-007", "node_modules", "@ws");
    for (const name of fs.readdirSync(nested)) {
      assert.ok(fs.existsSync(path.join(nested, name, "src", "index.ts")), name);
    }

    // Only the last packages carry SFCs, and nothing depends on them.
    const vuePackages = fs
      .readdirSync(path.join(base, "one", "packages"))
      .filter((name) =>
        fs.existsSync(path.join(base, "one", "packages", name, "src", "Widget.vue")),
      );
    assert.deepEqual(vuePackages, ["pkg-006", "pkg-007"]);
    assert.ok(!fs.readdirSync(nested).includes("pkg-006"));
    const components = fs.readdirSync(path.join(app, "src", "components"));
    assert.equal(components.length, SHAPE.appComponents);
  } finally {
    fs.rmSync(base, { recursive: true, force: true });
  }
});
