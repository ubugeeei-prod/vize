import assert from "node:assert/strict";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { loadLspManifest } from "../differential/lsp-manifest.ts";
import { inspectLspObservation } from "../differential/lsp-report.ts";
import { referenceObservation, rewriteFrames } from "./support/lsp/differential-reference.ts";

const loaded = loadLspManifest(
  fileURLToPath(new URL("../_fixtures/differential/lsp/manifest.json", import.meta.url)),
);
const fixture = loaded.cases.find(({ id }) => id === "lsp/regression/art-range-formatting-6098");
assert.ok(fixture);

test("the Art range fixture binds the whole authored request and UTF-16 content edit", () => {
  assert.equal(fixture.data.method, "textDocument/rangeFormatting");
  assert.equal(fixture.requests.length, 1);
  assert.deepEqual(fixture.requests, [
    {
      params: {
        range: { start: { line: 1, character: 27 }, end: { line: 1, character: 28 } },
        options: { tabSize: 2, insertSpaces: true },
      },
      result: [
        {
          range: { start: { line: 1, character: 26 }, end: { line: 1, character: 58 } },
          newText: '\n  <Button\n    data-z="z"\n    data-a="a"\n  />\n',
        },
      ],
    },
  ]);
  assert.deepEqual(inspectLspObservation(fixture, referenceObservation(fixture)), [
    { requestId: 2, state: "equal" },
  ]);
});

test("Art range comparisons detect a document-format request and incomplete or shifted edits", () => {
  for (const mutation of ["method", "selection", "text", "range", "extra-edit"] as const) {
    const observation = referenceObservation(fixture);
    rewriteFrames(
      observation,
      mutation === "method" || mutation === "selection" ? "client" : "server",
      (messages) => {
        const message = messages.find(({ id }) => id === 2);
        assert.ok(message);
        if (mutation === "method") message.method = "textDocument/formatting";
        else if (mutation === "selection")
          message.params = {
            ...message.params,
            range: { start: { line: 1, character: 29 }, end: { line: 1, character: 30 } },
          };
        else {
          assert.ok(Array.isArray(message.result));
          const edit = message.result[0] as { range: unknown; newText: string };
          if (mutation === "text") edit.newText = edit.newText.trim();
          else if (mutation === "range")
            edit.range = { start: { line: 1, character: 28 }, end: { line: 1, character: 60 } };
          else message.result.push({ ...edit });
        }
      },
    );
    if (mutation === "method" || mutation === "selection")
      assert.throws(() => inspectLspObservation(fixture, observation), mutation);
    else
      assert.deepEqual(
        inspectLspObservation(fixture, observation),
        [{ requestId: 2, state: "different" }],
        mutation,
      );
  }
});
