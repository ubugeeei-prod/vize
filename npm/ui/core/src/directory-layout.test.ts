import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";

import { test } from "vite-plus/test";

const sourceRoot = path.resolve("src");
const familySfcPattern =
  /^families\/[a-z0-9]+(?:-[a-z0-9]+)*\/[a-z0-9]+(?:-[a-z0-9]+)*\/[a-z0-9]+(?:-[a-z0-9]+)*\.vue$/u;

const grandfatheredRootSfcFiles = [
  "collapsible-content.vue",
  "collapsible-root.vue",
  "collapsible-trigger.vue",
  "deterministic-id-provider.vue",
  "error-summary.vue",
  "link-anchor.vue",
  "primitive-element.vue",
  "transition.vue",
] as const;

test("new public SFCs live in family directories", () => {
  assert.deepEqual(rootSfcFiles(), grandfatheredRootSfcFiles);
});

test("family SFCs keep area and primitive directory segments", () => {
  const files = collectVueFiles(path.join(sourceRoot, "families"))
    .map((filename) => toPosixPath(path.relative(sourceRoot, filename)))
    .sort();
  const offenders = files.filter((filename) => !familySfcPattern.test(filename));

  assert.ok(files.length > 0);
  assert.deepEqual(offenders, []);
});

function rootSfcFiles(): readonly string[] {
  return fs
    .readdirSync(sourceRoot, { withFileTypes: true })
    .filter((entry) => entry.isFile() && entry.name.endsWith(".vue"))
    .map((entry) => entry.name)
    .sort();
}

function collectVueFiles(directory: string): readonly string[] {
  return fs.readdirSync(directory, { withFileTypes: true }).flatMap((entry) => {
    const filename = path.join(directory, entry.name);
    if (entry.isDirectory()) return collectVueFiles(filename);
    return entry.isFile() && entry.name.endsWith(".vue") ? [filename] : [];
  });
}

function toPosixPath(filename: string): string {
  return filename.split(path.sep).join(path.posix.sep);
}
