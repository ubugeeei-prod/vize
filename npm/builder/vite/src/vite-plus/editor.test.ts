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
  assert.deepEqual(Object.keys(settings), ["[vue]"]);
});

void test("project LSP feature policy stays authoritative and disabled formatting stays unclaimed", () => {
  for (const policy of [{ formatting: false }, { enabled: false, formatting: true }]) {
    const source = '{ // another formatter owns Vue\n "other.setting": true,\n}\n';
    assert.equal(recommendEditor(source, false, policy), source);
  }
  const settings = parse(
    recommendEditor("{}", false, { lint: true, formatting: true, typecheck: false, editor: false }),
  );
  assert.deepEqual(settings, {
    "[vue]": { "editor.defaultFormatter": "ubugeeei.vize", "editor.formatOnSave": true },
  });
});

void test("adding extension recommendations retains comments inside the array", () => {
  const source = `// team extension policy
{
  "recommendations": [
    // keep the test explorer
    "Vitest.explorer", /* inline rationale */
    // retain Volar alongside Vize linting
    "Vue.volar", // trailing comment
  ],
  "unwantedRecommendations": ["other.extension"],
}
`;
  const result = recommendEditor(source, true);
  for (const comment of [
    "// team extension policy",
    "// keep the test explorer",
    "/* inline rationale */",
    "// retain Volar alongside Vize linting",
    "// trailing comment",
  ])
    assert.ok(result.includes(comment), comment);
  assert.deepEqual(parse(result), {
    recommendations: [
      "Vitest.explorer",
      "Vue.volar",
      "VoidZero.vite-plus-extension-pack",
      "ubugeeei.vize",
    ],
    unwantedRecommendations: ["other.extension"],
  });
  assert.equal(recommendEditor(result, true), result);
});

void test("extension recommendations compare IDs without case and keep existing spelling", () => {
  const source = '{"recommendations":["voidzero.vite-plus-extension-pack"]}';
  const result = recommendEditor(source, true);
  assert.deepEqual(parse(result).recommendations, [
    "voidzero.vite-plus-extension-pack",
    "ubugeeei.vize",
  ]);
  assert.equal(recommendEditor(result, true), result);
});

void test("malformed editor files are rejected before planning writes", () => {
  assert.throws(() => recommendEditor("{ broken", false), /preserved/);
  assert.throws(() => recommendEditor('{"recommendations":false}', true), /string array/);
});
