import assert from "node:assert/strict";
import { readdirSync } from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const sourceRoot = fileURLToPath(new URL("../src", import.meta.url));

function sourceFiles(directory: string): string[] {
  return readdirSync(directory, { withFileTypes: true }).flatMap((entry) => {
    const absolute = path.join(directory, entry.name);
    return entry.isDirectory() ? sourceFiles(absolute) : [path.relative(sourceRoot, absolute)];
  });
}

test("playground components use kebab-case file names", () => {
  const invalidNames = sourceFiles(sourceRoot)
    .filter((file) => file.endsWith(".vue"))
    .filter((file) => !/^[a-z0-9]+(?:-[a-z0-9]+)*\.vue$/u.test(path.basename(file)));

  assert.deepEqual(invalidNames, []);
});

test("component styles stay co-located in their SFCs", () => {
  const externalStyles = sourceFiles(sourceRoot).filter((file) => file.endsWith(".css"));

  assert.deepEqual(externalStyles, []);
});
