import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { resolve } from "node:path";
import test from "node:test";
import {
  buildDocsBackgroundScript,
  createDocsBackgroundHtml,
  docsBackgroundCanvasId,
} from "./background.ts";

const themeDir = fileURLToPath(new URL(".", import.meta.url));

test("buildDocsBackgroundScript inlines the shared vein bootstrap", () => {
  const script = buildDocsBackgroundScript(themeDir);
  const prefix = `${script.split("\n").slice(0, 8).join("\n")}\n`;
  const snapshotPath = resolve(
    themeDir,
    "__snapshots__",
    "background-script-prefix.snap",
  );

  assert.equal(prefix, readFileSync(snapshotPath, "utf-8"));
  assert.doesNotMatch(script, /__VERT_SRC__|__FRAG_SRC__/);
  assert.match(script, /document\.readyState === "loading"/);
  assert.match(script, /getElementById\("vein-canvas"\)/);
});

test("createDocsBackgroundHtml reuses the shared canvas id", () => {
  assert.equal(
    createDocsBackgroundHtml(),
    `<canvas id="${docsBackgroundCanvasId}" style="position:fixed;top:0;left:0;width:100%;height:100%;z-index:-1;pointer-events:none;"></canvas>`,
  );
});
