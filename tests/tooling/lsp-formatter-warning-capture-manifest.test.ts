import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { sha256 } from "../differential/harness.mjs";
import { loadLspManifest } from "../differential/lsp-manifest.ts";
import { loadFormatterBindPolicy } from "../differential/lsp-formatter-bind-policy.ts";
import {
  FORMATTER_WARNING_CAPTURE_ID,
  FORMATTER_WARNING_CONSUMER,
  loadFormatterWarningCapture,
} from "../differential/lsp-formatter-warning-capture.ts";
import type { FixtureData } from "../differential/lsp-types.ts";

const packRoot = fileURLToPath(new URL("../_fixtures/differential/lsp/", import.meta.url));
const directory = path.join(packRoot, "formatter-warning-capture-11815");

function fixture() {
  const data = JSON.parse(
    fs.readFileSync(path.join(directory, "case.json"), "utf8"),
  ) as FixtureData;
  assert.ok(data.files);
  const files = data.files.map((file) => ({
    runtimePath: file.runtimePath,
    bytes: fs.readFileSync(path.join(directory, file.source)),
  }));
  const expected = JSON.parse(fs.readFileSync(path.join(directory, data.expected.source), "utf8"));
  return { data, files, expected };
}

test("the new input materializes all three requests and authenticates the actual Rust consumer", () => {
  const loaded = loadLspManifest(path.join(packRoot, "manifest.json"));
  const current = loaded.cases.find((entry) => entry.id === FORMATTER_WARNING_CAPTURE_ID);
  assert.ok(current);
  assert.equal(current.requests.length, 3);
  assert.deepEqual(
    current.requests.map((request) => request.result),
    [null, null, null],
  );
  const original = loaded.cases.find(
    (entry) => entry.id === "lsp/regression/formatter-bind-style-policy-6098",
  );
  assert.ok(original);
  assert.deepEqual(current.requests, original.requests);
});

test("unknown and old IDs cannot reuse the warning-capture profile", () => {
  for (const id of [
    "lsp/regression/unknown-warning",
    "lsp/regression/formatter-bind-style-policy-6098",
  ]) {
    const { data, files, expected } = fixture();
    data.id = id;
    assert.throws(() => loadFormatterWarningCapture(packRoot, data, files, expected));
  }
  const { data, files, expected } = fixture();
  assert.throws(() => loadFormatterBindPolicy(data, files, expected));
});

test("missing warning input and altered source, config, warning or response bytes are refused", () => {
  const missing = fixture();
  missing.files.pop();
  assert.throws(() =>
    loadFormatterWarningCapture(packRoot, missing.data, missing.files, missing.expected),
  );
  for (const index of [0, 1, 2]) {
    const { data, files, expected } = fixture();
    files[index]!.bytes = Buffer.from("changed input\n");
    // A repinned hash cannot replace the fixed semantic contract.
    data.files![index]!.sha256 = sha256(files[index]!.bytes);
    assert.throws(() => loadFormatterWarningCapture(packRoot, data, files, expected));
  }
  const response = fixture();
  response.expected[0].result = [];
  assert.throws(() =>
    loadFormatterWarningCapture(packRoot, response.data, response.files, response.expected),
  );
});

test("a different consumer hash or a hash-valid consumer that leaves inputs unused is refused", () => {
  const changed = fixture();
  changed.data.provenance.witness!.sha256 = "0".repeat(64);
  assert.throws(() =>
    loadFormatterWarningCapture(packRoot, changed.data, changed.files, changed.expected),
  );
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-warning-consumer-"));
  try {
    const fakePack = path.join(root, "tests/_fixtures/differential/lsp");
    const consumer = path.join(root, FORMATTER_WARNING_CONSUMER);
    fs.mkdirSync(path.dirname(consumer), { recursive: true });
    const source =
      "fn every_handler_logs_the_conflict_returns_null_and_keeps_authored_bytes() {}\n#[test]\n";
    fs.writeFileSync(consumer, source);
    const unused = fixture();
    unused.data.provenance.witness!.sha256 = sha256(Buffer.from(source));
    assert.throws(
      () => loadFormatterWarningCapture(fakePack, unused.data, unused.files, unused.expected),
      /unused input/,
    );
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});
