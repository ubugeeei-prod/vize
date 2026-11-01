import assert from "node:assert/strict";
import type { FixtureData, LspFixture } from "./lsp-types.ts";

export const FORMATTER_BIND_POLICY_ID = "lsp/regression/formatter-bind-style-policy-6098";
export const FORMATTER_BIND_POLICY_FIXED = '<template><a v-bind:title.prop="t"></a></template>\n';
export const FORMATTER_BIND_POLICY_CONFIG =
  'export default { vue: {"formatter":{"normalizeDirectiveShorthands":true},"linter":{"preset":"incremental","ruleOptions":{"vue/v-bind-style":"longform"},"rules":{"vue/v-bind-style":"error"}}} };\n';

/** A new host-policy refusal contract, not an old independent formatter golden. */
export function loadFormatterBindPolicy(
  data: FixtureData,
  files: { runtimePath: string; bytes: Buffer }[],
  expected: unknown,
): LspFixture["requests"] {
  assert.equal(data.id, FORMATTER_BIND_POLICY_ID);
  assert.equal(data.method, "textDocument/formatting");
  assert.equal(data.provenance.fixCommit, "36d76a54ebdef00cfa83f8788224d0c69dad59a9");
  assert.equal(data.documentVersion, 1);
  assert.equal(data.openDocument, undefined, "the authored document must actually be opened");
  assert.equal(data.sourceSession, undefined);
  assert.equal(data.entry, "VBindProp.vue");
  assert.equal(data.request, undefined);
  assert.deepEqual(data.initializationOptions, {
    editor: true,
    lint: false,
    typecheck: false,
    formatting: true,
  });
  const options = { tabSize: 2, insertSpaces: true };
  assert.deepEqual(data.options, options);
  assert.deepEqual(
    files.map((file) => file.runtimePath),
    ["VBindProp.vue", "vite.config.mjs"],
  );
  assert.equal(files[0]!.bytes.toString("utf8"), FORMATTER_BIND_POLICY_FIXED);
  assert.equal(files[1]!.bytes.toString("utf8"), FORMATTER_BIND_POLICY_CONFIG);
  assert.deepEqual(expected, [
    { method: "textDocument/formatting", params: { options }, result: null },
    {
      method: "textDocument/rangeFormatting",
      params: {
        range: { start: { line: 0, character: 12 }, end: { line: 0, character: 40 } },
        options,
      },
      result: null,
    },
    {
      method: "textDocument/onTypeFormatting",
      params: { position: { line: 0, character: 24 }, ch: "\n", options },
      result: null,
    },
  ]);
  return expected as LspFixture["requests"];
}
