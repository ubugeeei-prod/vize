import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import test from "node:test";

import { collectVueLikeFilesFromTargets } from "./files.ts";

void test("collectVueLikeFilesFromTargets returns deduplicated absolute paths in stable order", () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "oxlint-files-"));
  try {
    fs.mkdirSync(path.join(root, "src"));
    const alpha = path.join(root, "src", "Alpha.vue");
    const zeta = path.join(root, "src", "Zeta.vue");
    fs.writeFileSync(alpha, "<template />\n");
    fs.writeFileSync(zeta, "<template />\n");

    assert.deepEqual(
      collectVueLikeFilesFromTargets(root, ["src/Zeta.vue", "src/Alpha.vue", "src/Zeta.vue"]),
      [alpha, zeta],
    );
  } finally {
    fs.rmSync(root, { force: true, recursive: true });
  }
});

void test("collectVueLikeFilesFromTargets includes standalone HTML inputs", () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "oxlint-files-"));
  try {
    fs.mkdirSync(path.join(root, "src"));
    const html = path.join(root, "src", "index.html");
    const htm = path.join(root, "src", "legacy.htm");
    const ignored = path.join(root, "src", "notes.md");
    fs.writeFileSync(html, "<div></div>\n");
    fs.writeFileSync(htm, "<div></div>\n");
    fs.writeFileSync(ignored, "# notes\n");

    assert.deepEqual(collectVueLikeFilesFromTargets(root, ["src"]), [html, htm].sort());
  } finally {
    fs.rmSync(root, { force: true, recursive: true });
  }
});

void test("collectVueLikeFilesFromTargets ignores supported-extension directories", () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "oxlint-files-"));
  try {
    const sourceDir = path.join(root, "src");
    const componentDir = path.join(sourceDir, "Directory.vue");
    fs.mkdirSync(componentDir, { recursive: true });

    const alpha = path.join(sourceDir, "Alpha.vue");
    const nested = path.join(componentDir, "Nested.vue");
    fs.writeFileSync(alpha, "<template />\n");
    fs.writeFileSync(nested, "<template />\n");

    assert.deepEqual(collectVueLikeFilesFromTargets(root, ["src"]), [nested, alpha].sort());
    assert.deepEqual(
      collectVueLikeFilesFromTargets(root, ["src/**/*.vue"]),
      [nested, alpha].sort(),
    );
  } finally {
    fs.rmSync(root, { force: true, recursive: true });
  }
});

void test("TSRX originals are discovered as carrier inputs without classifying them as host scripts", (t) => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-tsrx-files-"));
  t.after(() => fs.rmSync(root, { recursive: true, force: true }));
  fs.mkdirSync(path.join(root, "src"));
  for (const name of ["Avatar.tsrx", "Original.tsx", "Original.jsx", "Original.vue"])
    fs.writeFileSync(path.join(root, "src", name), "");
  const originals = [path.join(root, "src/Avatar.tsrx"), path.join(root, "src/Original.vue")];
  assert.deepEqual(collectVueLikeFilesFromTargets(root, ["src"]), originals);
  assert.deepEqual(collectVueLikeFilesFromTargets(root, ["src/*.tsrx"]), originals.slice(0, 1));
  assert.deepEqual(
    collectVueLikeFilesFromTargets(root, ["src/Avatar.tsrx"]),
    originals.slice(0, 1),
  );
});
