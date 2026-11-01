import assert from "node:assert/strict";
import type { FixtureData, LspFixture } from "./lsp-types.ts";

export const FORMATTER_SLOT_POLICY_ID = "lsp/regression/formatter-slot-style-policy-6098";
export const FORMATTER_SLOT_POLICY_FIXED =
  '<template>\n  <MyPanel v-slot:default="slot">{{ slot.item }}</MyPanel>\n  <MyPanel><template v-slot:header>Header</template></MyPanel>\n</template>\n';
export const FORMATTER_SLOT_POLICY_CONFIG =
  'export default { vue: {"formatter":{"normalizeDirectiveShorthands":true},"linter":{"preset":"incremental","ruleOptions":{"vue/v-slot-style":{"atComponent":"longform","default":"v-slot","named":"longform"}},"rules":{"vue/v-slot-style":"error"}}} };\n';

/** A new authored host-policy contract; no old transport/native oracle is implied. */
export function loadFormatterSlotPolicy(
  data: FixtureData,
  files: { runtimePath: string; bytes: Buffer }[],
  expected: unknown,
): LspFixture["requests"] {
  assert.equal(data.id, FORMATTER_SLOT_POLICY_ID);
  assert.equal(data.method, "textDocument/formatting");
  assert.equal(data.provenance.fixCommit, "699c512b2e8df3c69c2b3f957549a2287a33f372");
  assert.equal(data.documentVersion, 1);
  assert.equal(data.openDocument, undefined);
  assert.equal(data.sourceSession, undefined);
  assert.equal(data.entry, "Slots.vue");
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
    ["Slots.vue", "vite.config.mjs"],
  );
  assert.equal(files[0]!.bytes.toString("utf8"), FORMATTER_SLOT_POLICY_FIXED);
  assert.equal(files[1]!.bytes.toString("utf8"), FORMATTER_SLOT_POLICY_CONFIG);
  assert.deepEqual(expected, [
    { method: "textDocument/formatting", params: { options }, result: null },
    {
      method: "textDocument/rangeFormatting",
      params: {
        range: { start: { line: 0, character: 0 }, end: { line: 4, character: 0 } },
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
