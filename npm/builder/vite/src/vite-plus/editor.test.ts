import assert from "node:assert/strict";
import { test } from "node:test";
import { parse } from "jsonc-parser";
import { recommendEditor } from "./editor.ts";

void test("editor recommendations preserve comments, existing extensions and explicit tool ownership", () => {
  const extensions = recommendEditor(
    '{ // team defaults\n "recommendations": ["Vitest.explorer"],\n}',
    true,
  );
  assert.match(extensions, /team defaults/);
  assert.deepEqual(parse(extensions).recommendations, [
    "Vitest.explorer",
    "VoidZero.vite-plus-extension-pack",
    "ubugeeei.vize",
  ]);
  assert.equal(recommendEditor(extensions, true), extensions);
  const settings = recommendEditor(
    JSON.stringify({
      "editor.defaultFormatter": "oxc.oxc-vscode",
      "oxc.enable": true,
      "vize.typecheck.enable": false,
      "[vue]": { "editor.defaultFormatter": "Vue.volar" },
    }),
    false,
  );
  const parsed = parse(settings);
  assert.equal(parsed["editor.defaultFormatter"], "oxc.oxc-vscode");
  assert.equal(parsed["oxc.enable"], true);
  assert.equal(parsed["vize.typecheck.enable"], false);
  assert.equal(parsed["[vue]"]["editor.defaultFormatter"], "Vue.volar");
  assert.equal(parsed["[vue]"]["editor.formatOnSave"], true);
  assert.equal(recommendEditor(settings, false), settings);
});

void test("a new editor setup assigns only Vue formatting to Vize", () => {
  const settings = parse(recommendEditor("{}", false));
  assert.equal(settings["[vue]"]["editor.defaultFormatter"], "ubugeeei.vize");
  assert.equal(settings["editor.defaultFormatter"], undefined);
  assert.equal(settings["vize.lint.enable"], true);
  assert.equal(settings["vize.formatting.enable"], true);
});

void test("malformed editor files are rejected before planning writes", () => {
  assert.throws(() => recommendEditor("{ broken", false), /preserved/);
  assert.throws(() => recommendEditor('{"recommendations":false}', true), /string array/);
});
