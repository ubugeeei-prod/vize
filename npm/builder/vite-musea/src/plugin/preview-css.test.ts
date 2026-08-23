import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import test from "node:test";
import { resolvePreviewCssPath } from "./preview-css.js";

void test("package specifiers stay unresolved for the bundler", () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "musea-preview-css-"));
  try {
    assert.equal(resolvePreviewCssPath(root, "normalize.css"), "normalize.css");
    assert.equal(
      resolvePreviewCssPath(root, "@fontsource/inter/index.css"),
      "@fontsource/inter/index.css",
    );
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

void test("dot-relative paths resolve against the project root", () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "musea-preview-css-"));
  try {
    assert.equal(
      resolvePreviewCssPath(root, "./src/styles/tokens.css"),
      path.resolve(root, "./src/styles/tokens.css"),
    );
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

void test("existing project-relative files resolve against the project root", () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "musea-preview-css-"));
  try {
    const relative = path.join("src", "styles", "main.css");
    const absolute = path.join(root, relative);
    fs.mkdirSync(path.dirname(absolute), { recursive: true });
    fs.writeFileSync(absolute, "button{color:red}");
    assert.equal(resolvePreviewCssPath(root, relative), absolute);
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

void test("absolute paths are unchanged", () => {
  const absolute = path.resolve("/tmp/musea-preview.css");
  assert.equal(resolvePreviewCssPath("/other-root", absolute), absolute);
});
