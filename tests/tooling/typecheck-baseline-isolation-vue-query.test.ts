import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import { isolateUniqueVueQueryPackages } from "../../legacy-tools/fixtures/typecheck-baseline-isolation-unique.mjs";

/**
 * Vue Query and Apollo Composable live in Vize's `tests/package.json`.
 * TypeScript can climb into those copies and load Vize's Vue beside the
 * fixture (#4461). Unique isolation links the fixture store copy when an
 * ancestor is reachable.
 */

function scaffold(names: string[]) {
  const outer = fs.realpathSync(
    fs.mkdtempSync(path.join(os.tmpdir(), "vize-isolation-vue-query-")),
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

test("ancestor query packages with one in-fixture copy each are linked from those copies", () => {
  const { fixtureRoot, outer } = scaffold(["@tanstack/vue-query", "@vue/apollo-composable"]);
  try {
    writeStoreCopy(fixtureRoot, "@tanstack+vue-query@5.101.0", "@tanstack/vue-query");
    writeStoreCopy(fixtureRoot, "@vue+apollo-composable@4.2.2", "@vue/apollo-composable");
    assert.deepEqual(isolateUniqueVueQueryPackages(fixtureRoot), [
      {
        name: "@tanstack/vue-query",
        target: "node_modules/.pnpm/@tanstack+vue-query@5.101.0/node_modules/@tanstack/vue-query",
      },
      {
        name: "@vue/apollo-composable",
        target:
          "node_modules/.pnpm/@vue+apollo-composable@4.2.2/node_modules/@vue/apollo-composable",
      },
    ]);
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});

test("query packages no ancestor provides are left alone", () => {
  const { fixtureRoot, outer } = scaffold([]);
  try {
    writeStoreCopy(fixtureRoot, "@tanstack+vue-query@5.101.0", "@tanstack/vue-query");
    writeStoreCopy(fixtureRoot, "@vue+apollo-composable@4.2.2", "@vue/apollo-composable");
    assert.deepEqual(isolateUniqueVueQueryPackages(fixtureRoot), []);
    assert.equal(
      fs.existsSync(path.join(fixtureRoot, "node_modules", "@tanstack", "vue-query")),
      false,
    );
    assert.equal(
      fs.existsSync(path.join(fixtureRoot, "node_modules", "@vue", "apollo-composable")),
      false,
    );
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});

test("an already hoisted vue-query is left for isolation; this repair does not relink it", () => {
  const { fixtureRoot, outer } = scaffold(["@tanstack/vue-query"]);
  try {
    writeStoreCopy(fixtureRoot, "@tanstack+vue-query@5.101.0", "@tanstack/vue-query");
    const hoisted = path.join(fixtureRoot, "node_modules", "@tanstack", "vue-query");
    fs.mkdirSync(hoisted, { recursive: true });
    fs.writeFileSync(path.join(hoisted, "package.json"), `{"name":"@tanstack/vue-query"}\n`);
    assert.deepEqual(isolateUniqueVueQueryPackages(fixtureRoot), []);
    assert.equal(fs.lstatSync(hoisted).isSymbolicLink(), false);
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});
