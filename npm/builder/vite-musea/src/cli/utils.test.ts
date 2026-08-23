import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import test from "node:test";

import { parseArtFile, scanArtFiles } from "./utils.ts";

void test("scanArtFiles skips unreadable directories", async () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-musea-scan-"));
  const locked = path.join(root, "locked");

  try {
    fs.mkdirSync(locked);
    fs.chmodSync(locked, 0);
    fs.writeFileSync(path.join(root, "Button.art.vue"), "<art></art>");

    const files = await scanArtFiles(root);

    assert.deepEqual(files, [path.join(root, "Button.art.vue")]);
  } finally {
    fs.chmodSync(locked, 0o700);
    fs.rmSync(root, { recursive: true, force: true });
  }
});

function writeArtFile(root: string, source: string): string {
  const artPath = path.join(root, "Button.art.vue");
  fs.writeFileSync(artPath, source);
  return artPath;
}

void test("parseArtFile reports native draft for unknown art status", async () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-musea-parse-wip-"));
  const warnings: string[] = [];
  const warn = console.warn;
  console.warn = (message: string) => {
    warnings.push(String(message));
  };

  try {
    const artPath = writeArtFile(
      root,
      `<art title="Button" component="Button" status="wip">
  <variant name="Default" default>
    <Button />
  </variant>
</art>
`,
    );
    const parsed = await parseArtFile(artPath);
    assert.equal(parsed?.metadata.status, "draft");
    assert.deepEqual(warnings, [
      `[musea] ${artPath}: unknown status "wip"; falling back to "draft" (expected "draft" | "ready" | "deprecated")`,
    ]);
  } finally {
    console.warn = warn;
    fs.rmSync(root, { recursive: true, force: true });
  }
});

void test("parseArtFile keeps native ready status", async () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-musea-parse-ready-"));
  const warnings: string[] = [];
  const warn = console.warn;
  console.warn = (message: string) => {
    warnings.push(String(message));
  };

  try {
    const artPath = writeArtFile(
      root,
      `<art title="Button" component="Button" status="ready">
  <variant name="Default" default>
    <Button />
  </variant>
</art>
`,
    );
    const parsed = await parseArtFile(artPath);
    assert.equal(parsed?.metadata.status, "ready");
    assert.deepEqual(warnings, []);
  } finally {
    console.warn = warn;
    fs.rmSync(root, { recursive: true, force: true });
  }
});

void test("parseArtFile returns null when native parse fails", async () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-musea-parse-bad-"));
  const errors: string[] = [];
  const error = console.error;
  console.error = (message: string) => {
    errors.push(String(message));
  };

  try {
    const artPath = writeArtFile(root, "<art></art>");
    const parsed = await parseArtFile(artPath);
    assert.equal(parsed, null);
    assert.equal(errors.length, 1);
    assert.equal(
      errors[0],
      `[musea] Failed to process Button.art.vue: Missing required 'title' attribute in <art> block`,
    );
  } finally {
    console.error = error;
    fs.rmSync(root, { recursive: true, force: true });
  }
});
