import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { sha256 } from "../differential/harness.mjs";
import { loadOriginalHighlightRequests } from "../differential/lsp-highlight-manifest.ts";
import { loadLspManifest } from "../differential/lsp-manifest.ts";
import { inspectLspObservation } from "../differential/lsp-report.ts";
import type { JsonRpcMessage } from "../differential/lsp-types.ts";
import { referenceObservation, rewriteFrames } from "./support/lsp/differential-reference.ts";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const pack = path.join(root, "tests/_fixtures/differential/lsp");
const loaded = loadLspManifest(path.join(pack, "manifest.json"));
const originals = loaded.cases.slice(7);

void test("original five highlight sessions retain source, settings and all nine authored requests", () => {
  assert.equal(originals.length, 5);
  assert.deepEqual(
    originals.map((fixture) => fixture.requests.length),
    [1, 4, 1, 1, 2],
  );
  assert.equal(
    originals.reduce((total, fixture) => total + fixture.requests.length, 0),
    9,
  );
  for (const fixture of originals) {
    assert.deepEqual(fixture.initializationOptions, {
      editor: true,
      lint: false,
      typecheck: false,
    });
    assert.equal(fixture.data.provenance.fixCommit, "ddef7d9268425346f5b01b3da5ed592eb544b6dc");
    assert(
      inspectLspObservation(fixture, referenceObservation(fixture)).every(
        (row) => row.state === "equal",
      ),
    );
  }
  assert.equal(
    sha256(originals[0].files[0].bytes),
    "00b388e44d389ca87a73008dcc543baf4d90c4ee97b825ed888add69234be7eb",
  );
  assert.equal(
    sha256(originals[1].files[0].bytes),
    "a6b9b98ede93ce6a05925dab002db48152949441499a8e362befe2078de3b135",
  );
  assert.equal(originals[3].files[0].bytes.filter((byte) => byte === 13).length, 4);
  assert.equal(originals[4].files[0].bytes.length, 0);
  assert.deepEqual(
    originals[4].requests.map((request) => [request.method, request.result]),
    [
      ["textDocument/documentHighlight", null],
      ["textDocument/hover", null],
    ],
  );
});

void test("whole highlight responses reject order, kind, omission, extra fields and null drift", () => {
  for (const [index, mutate] of [
    [
      1,
      (message: JsonRpcMessage) => {
        assert(Array.isArray(message.result));
        message.result.reverse();
      },
    ],
    [
      1,
      (message: JsonRpcMessage) => {
        assert(Array.isArray(message.result));
        message.result[0].kind = 2;
      },
    ],
    [
      1,
      (message: JsonRpcMessage) => {
        assert(Array.isArray(message.result));
        delete message.result[0].kind;
      },
    ],
    [
      1,
      (message: JsonRpcMessage) => {
        assert(Array.isArray(message.result));
        message.result[0].tooltip = "extra";
      },
    ],
    [
      4,
      (message: JsonRpcMessage) => {
        message.result = [];
      },
    ],
    [
      4,
      (message: JsonRpcMessage) => {
        message.error = null;
      },
    ],
  ] as Array<[number, (message: JsonRpcMessage) => void]>) {
    const fixture = originals[index];
    const observation = referenceObservation(fixture);
    rewriteFrames(observation, "server", (messages) => mutate(messages[2]));
    assert(inspectLspObservation(fixture, observation).some((row) => row.state === "different"));
  }
  const empty = originals[4];
  const wrongMethod = referenceObservation(empty);
  rewriteFrames(wrongMethod, "client", (messages) => {
    messages[4].method = "textDocument/documentHighlight";
  });
  assert.throws(() => inspectLspObservation(empty, wrongMethod));
});

void test("highlight admission rejects changed original witness and dropped or redirected requests", () => {
  const temporary = fs.mkdtempSync(path.join(os.tmpdir(), "lsp-highlight-custody-"));
  try {
    fs.cpSync(pack, temporary, { recursive: true });
    fs.appendFileSync(path.join(temporary, "highlight-original/witness.test.ts.txt"), "changed");
    assert.throws(() => loadLspManifest(path.join(temporary, "manifest.json")), /SHA256 mismatch/);
  } finally {
    fs.rmSync(temporary, { recursive: true, force: true });
  }
  const fixture = originals[4];
  const expected = JSON.parse(
    fs.readFileSync(path.join(pack, fixture.inputs.root, fixture.data.expected.source), "utf8"),
  );
  assert.throws(() => loadOriginalHighlightRequests(pack, fixture.data, expected.slice(0, 1)));
  const redirected = structuredClone(expected);
  redirected[1].method = "textDocument/documentHighlight";
  assert.throws(() => loadOriginalHighlightRequests(pack, fixture.data, redirected));
  const recaptured = structuredClone(fixture.data);
  recaptured.initializationOptions = { editor: true, lint: false, typecheck: false, native: true };
  assert.throws(() => loadOriginalHighlightRequests(pack, recaptured, expected));
});
