import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { loadLspManifest } from "../differential/lsp-manifest.ts";
import { inspectLspObservation } from "../differential/lsp-report.ts";
import { referenceObservation, rewriteFrames } from "./support/lsp/differential-reference.ts";

const loaded = loadLspManifest(
  fileURLToPath(new URL("../_fixtures/differential/lsp/manifest.json", import.meta.url)),
);
const fixture = loaded.cases.find(({ id }) => id === "lsp/regression/editor-import-sorting-6098");
assert.ok(fixture);
const independentReference = readFileSync(
  new URL(
    "../../crates/vize_glyph/tests/fixtures/sort-imports/UserCard.sorted.vue",
    import.meta.url,
  ),
  "utf8",
);

test("editor import sorting binds the pinned config and complete document response", () => {
  assert.equal(fixture.data.method, "textDocument/formatting");
  assert.equal(fixture.entry, "App.vue");
  const config = fixture.files.find(({ runtimePath }) => runtimePath === "vite.config.mjs");
  assert.ok(config);
  assert.equal(
    config.bytes.toString("utf8"),
    "export default { vue: { formatter: { sortImports: {} } } };\n",
  );
  assert.deepEqual(fixture.requests, [
    {
      params: { options: { tabSize: 2, insertSpaces: true } },
      result: [
        {
          range: { start: { line: 0, character: 0 }, end: { line: 14, character: 0 } },
          newText: independentReference,
        },
      ],
    },
  ]);
  assert.deepEqual(inspectLspObservation(fixture, referenceObservation(fixture)), [
    { requestId: 2, state: "equal" },
  ]);
});

test("editor sorting rejects wrong requests and detects changed whole output or range", () => {
  for (const mutation of ["method", "options", "missing-edit", "text", "range"] as const) {
    const observation = referenceObservation(fixture);
    const client = mutation === "method" || mutation === "options";
    rewriteFrames(observation, client ? "client" : "server", (messages) => {
      const message = messages.find(({ id }) => id === 2);
      assert.ok(message);
      if (mutation === "method") message.method = "textDocument/rangeFormatting";
      else if (mutation === "options")
        message.params = { ...message.params, options: { tabSize: 8, insertSpaces: false } };
      else if (mutation === "missing-edit") message.result = [];
      else {
        assert.ok(Array.isArray(message.result));
        const edit = message.result[0] as { range: unknown; newText: string };
        if (mutation === "text") edit.newText = edit.newText.trimEnd();
        else edit.range = { start: { line: 1, character: 0 }, end: { line: 14, character: 0 } };
      }
    });
    if (client) assert.throws(() => inspectLspObservation(fixture, observation), mutation);
    else
      assert.deepEqual(
        inspectLspObservation(fixture, observation),
        [{ requestId: 2, state: "different" }],
        mutation,
      );
  }
});
