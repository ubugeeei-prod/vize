import assert from "node:assert/strict";
import type { FixtureData, LspFixture } from "./lsp-types.ts";

export const FORMATTER_ON_POLICY_ID = "lsp/regression/formatter-on-style-policy-6098";
export const FORMATTER_ON_POLICY_FIXED =
  '<template>\n  <button v-on:click="save" v-on:keyup="cancel" v-on="listeners">Save</button>\n</template>\n';
export const FORMATTER_ON_POLICY_CONFIG =
  'export default { vue: {"formatter":{"normalizeDirectiveShorthands":true},"linter":{"preset":"incremental","ruleOptions":{"vue/v-on-style":"longform"},"rules":{"vue/v-on-style":"error"}}} };\n';

/** A new authored host-policy contract; no old transport/native oracle is implied. */
export function loadFormatterOnPolicy(
  data: FixtureData,
  files: { runtimePath: string; bytes: Buffer }[],
  expected: unknown,
): LspFixture["requests"] {
  assert.equal(data.id, FORMATTER_ON_POLICY_ID);
  assert.equal(data.method, "textDocument/formatting");
  assert.equal(data.provenance.fixCommit, "b46054f5ca8cf890f13d6e75488c052a442ba39b");
  assert.equal(data.documentVersion, 1);
  assert.equal(data.openDocument, undefined);
  assert.equal(data.sourceSession, undefined);
  assert.equal(data.entry, "Events.vue");
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
    ["Events.vue", "vite.config.mjs"],
  );
  assert.equal(files[0]!.bytes.toString("utf8"), FORMATTER_ON_POLICY_FIXED);
  assert.equal(files[1]!.bytes.toString("utf8"), FORMATTER_ON_POLICY_CONFIG);
  assert.deepEqual(expected, [
    { method: "textDocument/formatting", params: { options }, result: null },
    {
      method: "textDocument/rangeFormatting",
      params: {
        range: { start: { line: 0, character: 0 }, end: { line: 3, character: 0 } },
        options,
      },
      result: null,
    },
    {
      method: "textDocument/onTypeFormatting",
      params: {
        position: { line: 1, character: 2 },
        ch: "\n",
        options,
      },
      result: null,
    },
  ]);
  return expected as LspFixture["requests"];
}
