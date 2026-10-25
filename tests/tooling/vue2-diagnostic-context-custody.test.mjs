import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const root = fileURLToPath(new URL("../../", import.meta.url));
const custodyBytes = fs.readFileSync(
  path.join(root, "tests/_fixtures/differential/typechecker/vue2-elm-context-custody/custody.json"),
);
const custody = JSON.parse(custodyBytes.toString("utf8"));
const historicalBytes = fs.readFileSync(path.join(root, custody.historical.path));
const successorBytes = fs.readFileSync(path.join(root, custody.successor.path));
const historical = JSON.parse(historicalBytes.toString("utf8"));
const successor = JSON.parse(successorBytes.toString("utf8"));
const sha256 = (bytes) => createHash("sha256").update(bytes).digest("hex");

function diagnosticIdentity(text) {
  const parsed = /^(error|warning|hint|info):(\d+):(\d+)(?: \[TS(\d+)\])? ([\s\S]+)$/.exec(text);
  assert.notEqual(parsed, null, "complete canonical diagnostic must parse");
  return {
    line: Number(parsed[2]),
    column: Number(parsed[3]),
    code: parsed[4] === undefined ? null : Number(parsed[4]),
    canonicalMessage: parsed[5],
  };
}

/** Construct the complete expected successor from the preserved old packet and
 * independently pinned authored line witnesses. No output text is stripped.
 */
function assertSuccessor(previous, current) {
  assert.deepEqual(
    { errors: previous.errorCount, warnings: previous.warningCount, files: previous.fileCount },
    { errors: 408, warnings: 0, files: 61 },
  );
  assert.equal(previous.files.length, 61);
  assert.equal(
    previous.files.reduce((sum, file) => sum + file.diagnostics.length, 0),
    408,
  );
  assert.equal(custody.witnesses.length, 92);
  const expected = structuredClone(previous);
  const changed = new Set();
  for (const witness of custody.witnesses) {
    const file = expected.files.find((item) => item.file === witness.file);
    assert.notEqual(file, undefined, "every reviewed source belongs to the complete packet");
    const key = `${witness.file}:${witness.diagnosticIndex}`;
    assert.equal(changed.has(key), false, "one reviewed addition per diagnostic");
    changed.add(key);
    const canonical = file.diagnostics[witness.diagnosticIndex];
    assert.deepEqual(diagnosticIdentity(canonical), {
      line: witness.line,
      column: witness.column,
      code: witness.code,
      canonicalMessage: witness.canonicalMessage,
    });
    assert.ok(witness.column <= witness.sourceLine.length + 1, "authored UTF-16 position");
    // The reviewed JSON context contract limits the source line to160 Unicode
    // scalar values. These inputs came from the pinned upstream blob, not the
    // diagnostic renderer or a current compiler invocation.
    const characters = [...witness.sourceLine.trim()];
    const excerpt = characters.slice(0, 160).join("") + (characters.length > 160 ? "..." : "");
    const binding = witness.binding === null ? "" : `; binding: ${witness.binding}`;
    file.diagnostics[witness.diagnosticIndex] = `${canonical} (source: ${excerpt}${binding})`;
  }
  assert.equal(changed.size, 92);
  assert.deepEqual(current, expected, "every historical facet plus the complete reviewed context");
}

test("Vue2 packet custody preserves all historical bytes and the complete source-built successor", () => {
  assert.equal(custody.schema, "vize.vue2DiagnosticContextCustody");
  assert.equal(custody.version, 1);
  assert.equal(
    sha256(custodyBytes),
    "10e5d0b563a94fd36636e6f6261e7b2a6a5ca333a0a5ead9e63d5ad413ab72af",
  );
  assert.equal(custody.sourceRevision, "c8c4e052e6002c05ae3a4ef645458b25033da1eb");
  assert.deepEqual(custody.producer, {
    workflowRun: 40166711835,
    job: 121089153833,
    artifact: "vue-parity-snapshot-differences",
  });
  assert.equal(
    sha256(historicalBytes),
    "8f06c4ce93b5dba1558c541cc7dd7b11dafdfc665e4dcbf6babf115e257c6c6f",
  );
  assert.equal(sha256(historicalBytes), custody.historical.sha256);
  assert.equal(
    sha256(successorBytes),
    "478e366358a67bffca96b2329c07ec74750b202819d1449e3831c32db7fbe3dd",
  );
  assert.equal(sha256(successorBytes), custody.successor.sha256);
  assert.equal(custody.upstream.revision, "b85ff0220366be863868269177a4abf74c7a85ca");
  assert.equal(custody.upstream.license, "GPL-2.0-only");
  assert.equal(custody.upstream.files.length, 19);
  assert.deepEqual(
    {
      files: custody.fileCount,
      diagnostics: custody.diagnosticCount,
      additions: custody.contextAdditions,
      sourceCharacters: custody.contextSourceCharacters,
    },
    { files: 61, diagnostics: 408, additions: 92, sourceCharacters: 160 },
  );
  assert.deepEqual(
    [...new Set(custody.witnesses.map((witness) => witness.file))].sort((left, right) =>
      left < right ? -1 : left > right ? 1 : 0,
    ),
    custody.upstream.files.map((file) => file.file),
  );
  assertSuccessor(historical, successor);
});

test("Vue2 successor custody rejects canonical, coordinate, context and inventory changes", () => {
  for (const mutate of [
    (packet) => {
      packet.files[0].file += ".unexpected";
    },
    (packet) => {
      packet.files[0].diagnostics.push("error:1:1 [TS9999] unexpected");
    },
    (packet) => {
      packet.errorCount -= 1;
    },
    (packet) => {
      const witness = custody.witnesses[0];
      const file = packet.files.find((item) => item.file === witness.file);
      file.diagnostics[witness.diagnosticIndex] = file.diagnostics[witness.diagnosticIndex].replace(
        "source:",
        "changed:",
      );
    },
    (packet) => {
      const witness = custody.witnesses[0];
      const file = packet.files.find((item) => item.file === witness.file);
      file.diagnostics[witness.diagnosticIndex] = file.diagnostics[witness.diagnosticIndex].replace(
        witness.canonicalMessage,
        "altered canonical message",
      );
    },
    (packet) => {
      const file = packet.files.find((item) => item.diagnostics.length > 0);
      file.diagnostics[0] = file.diagnostics[0].replace(/^error:\d+:/, "error:999:");
    },
  ]) {
    const mutated = structuredClone(successor);
    mutate(mutated);
    assert.throws(() => assertSuccessor(historical, mutated), assert.AssertionError);
  }
});
