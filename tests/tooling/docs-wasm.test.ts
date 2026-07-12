import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

test("WASM guide uses the public nested SFC compile result", () => {
  const guide = fs.readFileSync(path.join(root, "docs/content/guide/wasm.md"), "utf8");

  assert.doesNotMatch(guide, /result\.code\b/);
  assert.doesNotMatch(
    guide,
    /const\s+\{\s*code(?:\s*,\s*errors)?\s*\}\s*=\s*(?:compiler\.)?compileSfc\b/,
  );
  assert.match(guide, /result\.script\.code/);
  assert.match(guide, /result\.template\?\.code/);
  assert.match(guide, /result\.css/);
  assert.match(guide, /result\.errors/);
});
