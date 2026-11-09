import assert from "node:assert/strict";
import { test } from "node:test";
import { withoutReplayAliasRows } from "../../tools/support/compat/content-mapper/callback-owner-replay-aliases.ts";

const text = [
  "  for (const [entry] of __vize_v_for_source_7) {",
  "  // Component props in v-for scope: entry in [state]",
  "  {",
  "  const __vize_v_for_source_9 = __vForList([state]);",
  "  for (const [entry] of __vize_v_for_source_9) {",
  "    void entry;",
  "",
].join("\n");
const template = text.indexOf("[entry]") + 1;
const replay = text.lastIndexOf("[entry]") + 1;
const packet = () => ({
  text,
  mappings: [
    [template, 5, 40, 5, 0, 1048575],
    [replay, 5, 40, 5, 0, 1048575],
  ],
  diagnostics: [],
  diagnosticDirectives: {
    directives: [
      [0, 0, 0, template, 0, 0],
      [0, 0, template + 5, replay, 0, 0],
      [0, 0, replay + 5, text.length, 0, 0],
    ],
    unusedExpectDirectiveDiagnostics: [],
  },
});

test("only a loop replay's alias rows leave, merging the gap they split", () => {
  assert.deepEqual(withoutReplayAliasRows(packet()), {
    ...packet(),
    mappings: [[template, 5, 40, 5, 0, 1048575]],
    diagnosticDirectives: {
      directives: [
        [0, 0, 0, template, 0, 0],
        [0, 0, template + 5, text.length, 0, 0],
      ],
      unusedExpectDirectiveDiagnostics: [],
    },
  });
  const unmapped = packet();
  unmapped.mappings.pop();
  assert.equal(withoutReplayAliasRows(unmapped), undefined);
  const unsplit = packet();
  unsplit.diagnosticDirectives.directives.pop();
  assert.throws(() => withoutReplayAliasRows(unsplit), /splits one ignored gap/u);
});
