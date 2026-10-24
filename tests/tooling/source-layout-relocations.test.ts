import assert from "node:assert/strict";
import {
  copyFileSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  realpathSync,
  rmSync,
  statSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { dirname, isAbsolute, join, relative, resolve } from "node:path";
import { test } from "node:test";
import { fileURLToPath, pathToFileURL } from "node:url";
import {
  currentSourcePath,
  previousSourcePath,
} from "../../tools/support/source-layout-relocations.ts";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "../..");
const helper = "tools/support/source-layout-relocations.ts";
const inventory = "tests/_fixtures/source-layout/10418.json";

test("declared historical source locations have unique contained live owners", () => {
  const { paths } = JSON.parse(readFileSync(join(root, inventory), "utf8")) as {
    paths: Record<string, string>;
  };
  for (const [before, after] of Object.entries(paths)) {
    assert.equal(currentSourcePath(before), after);
    assert.equal(previousSourcePath(after), before);
    const file = realpathSync(join(root, after));
    const contained = relative(realpathSync(root), file);
    assert(contained.length > 0 && !contained.startsWith("..") && !isAbsolute(contained), after);
    assert(statSync(file).isFile(), after);
  }
  assert.equal(
    currentSourcePath("tests/_fixtures/frozen-original.txt"),
    "tests/_fixtures/frozen-original.txt",
  );
  assert.equal(previousSourcePath("unrelated/current.rs"), "unrelated/current.rs");
});

test("source relocation metadata rejects ambiguous owners and escaping coordinates", async () => {
  const fixture = mkdtempSync(join(tmpdir(), "vize-source-coordinates-"));
  try {
    mkdirSync(dirname(join(fixture, helper)), { recursive: true });
    mkdirSync(dirname(join(fixture, inventory)), { recursive: true });
    copyFileSync(join(root, helper), join(fixture, helper));
    const valid = {
      version: 1,
      issue: 10418,
      from: "3146e4fa530b8b886d8429ae379096bfcc791cad",
      paths: { "crates/before.rs": "crates/after.rs" },
    };
    const rejected = [
      { ...valid, version: 2 },
      { ...valid, from: "different source" },
      { ...valid, paths: { "crates/a.rs": "crates/after.rs", "crates/b.rs": "crates/after.rs" } },
      { ...valid, paths: { "crates/before.rs": "crates/../../outside.rs" } },
      { ...valid, paths: { "crates/before.rs": "/absolute.rs" } },
    ];
    for (const [index, value] of rejected.entries()) {
      writeFileSync(join(fixture, inventory), JSON.stringify(value));
      await assert.rejects(import(`${pathToFileURL(join(fixture, helper)).href}?case=${index}`));
    }
  } finally {
    rmSync(fixture, { recursive: true, force: true });
  }
});
