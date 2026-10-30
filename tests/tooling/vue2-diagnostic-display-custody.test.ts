import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

type Packet = {
  errorCount: number;
  fileCount: number;
  warningCount: number;
  files: Array<{ file: string; diagnostics: string[] }>;
};

type Receipt = {
  schema: string;
  version: number;
  sourceRevision: string;
  producer: {
    workflowRun: number;
    job: number;
    artifactId: number;
    artifact: string;
    artifactSha256: string;
    recipe: string;
    binarySha256: string;
  };
  previous: { path: string; sha256: string };
  successor: { path: string; sha256: string };
  fileCount: number;
  diagnosticCount: number;
  changedFileCount: number;
  changedDiagnosticCount: number;
  display: { before: string; after: string };
  changes: Array<{ file: string; diagnosticIndices: number[] }>;
};

const root = fileURLToPath(new URL("../../", import.meta.url));
const receiptBytes = fs.readFileSync(
  path.join(
    root,
    "tests/_fixtures/differential/typechecker/vue2-elm-context-custody/display-successor.json",
  ),
);
const receipt = JSON.parse(receiptBytes.toString("utf8")) as Receipt;
const previousBytes = fs.readFileSync(path.join(root, receipt.previous.path));
const successorBytes = fs.readFileSync(path.join(root, receipt.successor.path));
const previous = JSON.parse(previousBytes.toString("utf8")) as Packet;
const successor = JSON.parse(successorBytes.toString("utf8")) as Packet;
const sha256 = (bytes: Uint8Array) => createHash("sha256").update(bytes).digest("hex");

/** Reconstruct only the 170 reviewed, indexed type-display changes. The entire
 * preserved packet remains authoritative for every other diagnostic facet.
 */
function assertDisplaySuccessor(prior: Packet, current: Packet): void {
  assert.deepEqual(
    { errors: prior.errorCount, warnings: prior.warningCount, files: prior.fileCount },
    { errors: 408, warnings: 0, files: 61 },
  );
  assert.equal(prior.files.length, 61);
  assert.equal(
    prior.files.reduce((count, file) => count + file.diagnostics.length, 0),
    408,
  );
  const expected = structuredClone(prior);
  const changed = new Set<string>();
  const changedFiles = new Set<string>();
  for (const witness of receipt.changes) {
    assert.equal(changedFiles.has(witness.file), false, "one finite witness per changed file");
    changedFiles.add(witness.file);
    const file = expected.files.find((candidate) => candidate.file === witness.file);
    assert.ok(file, "every display change belongs to the complete preserved packet");
    for (const index of witness.diagnosticIndices) {
      assert.ok(Number.isSafeInteger(index) && index >= 0 && index < file.diagnostics.length);
      const identity = `${witness.file}:${index}`;
      assert.equal(changed.has(identity), false, "one display change per diagnostic");
      changed.add(identity);
      const before = file.diagnostics[index];
      const at = before.indexOf(receipt.display.before);
      assert.notEqual(at, -1, "the exact old type display is present");
      assert.equal(before.indexOf(receipt.display.before, at + 1), -1);
      file.diagnostics[index] = before.replace(receipt.display.before, receipt.display.after);
    }
  }
  assert.equal(changedFiles.size, 33);
  assert.equal(changed.size, 170);
  assert.deepEqual(current, expected, "only the finite type-display witnesses may change");
}

void test("Vue2 display successor retains both historical packets and all 408 diagnostics", () => {
  assert.equal(
    sha256(receiptBytes),
    "0d50dd25b44c13a70c82dba781342c863dc8c8c5a477e8820d06361fff3d45e0",
  );
  assert.equal(receipt.schema, "vize.vue2DiagnosticDisplayCustody");
  assert.equal(receipt.version, 1);
  assert.equal(receipt.sourceRevision, "bdd1a5c3ede92fcc252839965ee00f108681928e");
  assert.deepEqual(receipt.producer, {
    workflowRun: 41101082471,
    job: 124218273561,
    artifactId: 12998523533,
    artifact: "vue-parity-snapshot-differences",
    artifactSha256: "bfa98790150c2b090fac619e59e81048ba7d92a79c20745f50c6c0471ef61040",
    recipe: "cargo build --profile ci -p vize --features legacy",
    binarySha256: "1edb7770e5dc828e3a7259f9d5de5cddba509d63d5d45316717a9d9e3060b14f",
  });
  assert.deepEqual(receipt.previous, {
    path: "tests/snapshots/check/__snapshots__/vue2-elm-context-check.snap",
    sha256: "478e366358a67bffca96b2329c07ec74750b202819d1449e3831c32db7fbe3dd",
  });
  assert.deepEqual(receipt.successor, {
    path: "tests/snapshots/check/__snapshots__/vue2-elm-display-check.snap",
    sha256: "663f90741037634caac9c483c5e78089f597a117f25f333b50747e7ed7732482",
  });
  assert.deepEqual(receipt.display, { before: "type '__VizeThis'", after: "type '$This'" });
  assert.deepEqual(
    {
      files: receipt.fileCount,
      diagnostics: receipt.diagnosticCount,
      changedFiles: receipt.changedFileCount,
      changedDiagnostics: receipt.changedDiagnosticCount,
    },
    { files: 61, diagnostics: 408, changedFiles: 33, changedDiagnostics: 170 },
  );
  assert.equal(sha256(previousBytes), receipt.previous.sha256);
  assert.equal(sha256(successorBytes), receipt.successor.sha256);
  assert.equal(
    sha256(
      fs.readFileSync(path.join(root, "tests/snapshots/check/__snapshots__/vue2-elm-check.snap")),
    ),
    "8f06c4ce93b5dba1558c541cc7dd7b11dafdfc665e4dcbf6babf115e257c6c6f",
  );
  assertDisplaySuccessor(previous, successor);
});

void test("Vue2 display custody rejects message, coordinate, context and inventory drift", () => {
  const changed = receipt.changes[0];
  const mutateDiagnostic = (packet: Packet, mutate: (diagnostic: string) => string) => {
    const file = packet.files.find((candidate) => candidate.file === changed.file)!;
    const index = changed.diagnosticIndices[0];
    file.diagnostics[index] = mutate(file.diagnostics[index]);
  };
  const mutations: Array<(packet: Packet) => void> = [
    (packet) => {
      packet.errorCount -= 1;
    },
    (packet) => {
      packet.files.pop();
    },
    (packet) => {
      packet.files.reverse();
    },
    (packet) => {
      packet.files[0].file += ".foreign";
    },
    (packet) => {
      packet.files[0].diagnostics.push("error:1:1 [TS9999] unexpected");
    },
    (packet) => mutateDiagnostic(packet, (text) => text.replace("type '$This'", "type 'any'")),
    (packet) => mutateDiagnostic(packet, (text) => text.replace(/\[TS\d+\]/, "[TS9999]")),
    (packet) => mutateDiagnostic(packet, (text) => text.replace(/^error:\d+:/, "error:999:")),
    (packet) => mutateDiagnostic(packet, (text) => `${text} (foreign context)`),
    (packet) => mutateDiagnostic(packet, (text) => text.replace("does not exist", "exists")),
  ];
  for (const mutate of mutations) {
    const copy = structuredClone(successor);
    mutate(copy);
    assert.notDeepEqual(copy, successor, "each mutation changes an observed packet facet");
    assert.throws(() => assertDisplaySuccessor(previous, copy), assert.AssertionError);
  }
});
