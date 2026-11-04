import assert from "node:assert/strict";
import { cpSync, mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join, relative, resolve } from "node:path";
import { test, type TestContext } from "node:test";
import { fileURLToPath } from "node:url";
import {
  declarationClosure,
  declarationParser,
} from "./template-alias-paired-11822-declarations.ts";
import { copyPublishedPackage } from "./template-alias-paired-11822-packages.ts";
import { sharedHelperClosure } from "./template-alias-paired-11822-protocol.ts";
import type { PackageManifest } from "./template-alias-paired-11822-types.ts";

const typescript = declarationParser(process.env.TEMPLATE_ALIAS_TYPESCRIPT_API);
const sourcePackage = resolve(dirname(fileURLToPath(import.meta.url)), "../../../npm/cli");

function fixture(t: TestContext): string {
  const root = mkdtempSync(join(tmpdir(), "mapper-declaration-closure-"));
  t.after(() => rmSync(root, { recursive: true, force: true }));
  const packageRoot = join(root, "node_modules/vize");
  cpSync(sourcePackage, packageRoot, {
    recursive: true,
    filter: (path) => !relative(sourcePackage, path).split("/").includes("node_modules"),
  });
  // Test-only declarations stand in for the pack step, which is never run locally.
  mkdirSync(join(packageRoot, "dist"), { recursive: true });
  for (const file of ["index.d.mts", "config.d.mts"])
    writeFileSync(join(packageRoot, "dist", file), "export {};\n");
  mkdirSync(join(root, "node_modules/vue"), { recursive: true });
  writeFileSync(join(root, "node_modules/vue/package.json"), '{"name":"vue","version":"fixture"}');
  return packageRoot;
}

test("actual shared SDK globals and slots are physically required before copy and helper admission", (t) => {
  const packageRoot = fixture(t);
  const workspace = resolve(packageRoot, "../..");
  const admitted = sharedHelperClosure(workspace, typescript);
  assert.ok(
    admitted.physicalDeclarations.files.some(({ path }) => path === "virtual-ts-globals.d.ts"),
  );
  assert.ok(
    admitted.physicalDeclarations.files.some(({ path }) => path === "virtual-ts-slots.d.ts"),
  );
  assert.ok(
    admitted.physicalDeclarations.unreachableShippedDeclarations.includes(
      "src/config/module-runtime.d.mts",
    ),
  );
  assert.equal(
    admitted.physicalDeclarations.files.some(
      ({ path }) => path === "src/config/module-runtime.d.mts",
    ),
    false,
  );
  for (const name of ["virtual-ts-globals.d.ts", "virtual-ts-slots.d.ts"]) {
    const original = readFileSync(join(packageRoot, name));
    rmSync(join(packageRoot, name));
    assert.throws(
      () => copyPublishedPackage(packageRoot, join(workspace, name + ".copy")),
      /missing declared publish path/,
    );
    assert.throws(
      () => sharedHelperClosure(workspace, typescript),
      /missing declared publish path/,
    );
    writeFileSync(join(packageRoot, name), original);
  }
  assert.deepEqual(sharedHelperClosure(workspace, typescript), admitted);
});

test("all conditional types exports and recursive relative imports are required even when unused", (t) => {
  const root = fixture(t);
  const manifest = JSON.parse(readFileSync(join(root, "package.json"), "utf8")) as PackageManifest;
  manifest.exports = {
    ...(manifest.exports as Record<string, unknown>),
    "./unused": { require: { types: "./unused.d.cts" } },
  };
  writeFileSync(join(root, "package.json"), JSON.stringify(manifest));
  writeFileSync(join(root, "unused.d.cts"), 'export type { Leaf } from "./unlisted-leaf.cjs";\n');
  writeFileSync(
    join(root, "unlisted-leaf.d.cts"),
    '/// <reference path="./reference-only.d.ts" />\nexport type Leaf = string;\nexport type Again = import("./unused.cjs");\n',
  );
  writeFileSync(
    join(root, "reference-only.d.ts"),
    "type Literal = 'import(\"./comment-only-missing.js\")';\n",
  );
  const closure = declarationClosure(root, typescript);
  assert.ok(closure.files.some(({ path }) => path === "unlisted-leaf.d.cts"));
  assert.ok(closure.files.some(({ path }) => path === "reference-only.d.ts"));
  for (const [file, pattern] of [
    ["unused.d.cts", /missing declaration target/],
    ["unlisted-leaf.d.cts", /missing relative declaration/],
    ["reference-only.d.ts", /missing declaration target/],
  ] as const) {
    const bytes = readFileSync(join(root, file));
    rmSync(join(root, file));
    assert.throws(() => declarationClosure(root, typescript), pattern);
    writeFileSync(join(root, file), bytes);
  }
});
