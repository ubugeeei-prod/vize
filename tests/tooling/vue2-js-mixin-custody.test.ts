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

const root = fileURLToPath(new URL("../../", import.meta.url));
const read = (relativePath: string) => fs.readFileSync(path.join(root, relativePath));
const sha256 = (bytes: Uint8Array) => createHash("sha256").update(bytes).digest("hex");
const receipt = JSON.parse(
  read(
    "tests/_fixtures/differential/typechecker/vue2-elm-context-custody/js-mixin-successor.json",
  ).toString("utf8"),
);
const previousBytes = read(receipt.previous.path);
const successorBytes = read(receipt.successor.path);
const previous = JSON.parse(previousBytes.toString("utf8")) as Packet;
const successor = JSON.parse(successorBytes.toString("utf8")) as Packet;
const removedFile = "src/page/profile/profile.vue";
const removedDiagnostic =
  "error:179:29 [TS2551] Property 'getImgPath' does not exist on type '$This'. Did you mean 'imgpath'?";

function assertJsMixinSuccessor(current: Packet): void {
  assert.deepEqual([previous.fileCount, previous.warningCount, previous.errorCount], [61, 0, 408]);
  const expected = structuredClone(previous);
  const file = expected.files.find((entry) => entry.file === removedFile);
  assert.ok(file);
  assert.equal(file.diagnostics.filter((diagnostic) => diagnostic === removedDiagnostic).length, 1);
  file.diagnostics = file.diagnostics.filter((diagnostic) => diagnostic !== removedDiagnostic);
  expected.errorCount -= 1;
  assert.equal(
    expected.files.reduce((count, entry) => count + entry.diagnostics.length, 0),
    407,
  );
  assert.deepEqual(
    current,
    expected,
    "only the defined mixin method's false positive may disappear",
  );
}

void test("Vue2 JS mixin successor preserves all other 407 diagnostics and historical custody", () => {
  assert.equal(receipt.schema, "vize.vue2JsMixinDiagnosticCustody");
  assert.equal(receipt.version, 1);
  assert.equal(receipt.sourceRevision, "b5cb2e0db3861f5b2e86cbdde53384342bfef5e6");
  assert.deepEqual(receipt.producer, {
    workflowRun: 41748109798,
    job: 126306116774,
    artifactId: 13285580302,
    artifact: "vue-parity-snapshot-differences",
    artifactSha256: "9a3ba03d78287268f2ff1ecfa78234b8cfe13fe3cabfce8aef09184babf95d63",
    recipe: "cargo build --profile ci -p vize --features legacy",
    binarySha256: "03ce603baed14a20d861401098de2310b310f954e3b8522068fc664c1b553732",
  });
  assert.deepEqual(receipt.previous, {
    path: "tests/snapshots/check/__snapshots__/vue2-elm-display-check.snap",
    sha256: "663f90741037634caac9c483c5e78089f597a117f25f333b50747e7ed7732482",
  });
  assert.deepEqual(receipt.successor, {
    path: "tests/snapshots/check/__snapshots__/vue2-elm-js-mixin-check.snap",
    sha256: "ce43a2b3c6acb6be5745258ea8708b981e7164986054854e8902c8680b19900b",
  });
  assert.deepEqual(receipt.removed, { file: removedFile, diagnostic: removedDiagnostic });
  assert.deepEqual(receipt.upstream, {
    repository: "bailicangdu/vue2-elm",
    revision: "b85ff0220366be863868269177a4abf74c7a85ca",
    componentBlob: "9aa9d42a6134c17c487739539bd639d64b02bf96",
    mixinBlob: "57b1dc0cbdbd29d19c30f511157aa49011737524",
  });
  assert.deepEqual(
    [
      receipt.fileCount,
      receipt.previousDiagnosticCount,
      receipt.diagnosticCount,
      receipt.changedFileCount,
      receipt.removedDiagnosticCount,
    ],
    [61, 408, 407, 1, 1],
  );
  assert.equal(sha256(previousBytes), receipt.previous.sha256);
  assert.equal(sha256(successorBytes), receipt.successor.sha256);
  assertJsMixinSuccessor(successor);
});

void test("Vue2 JS mixin custody rejects unrelated diagnostic and inventory loss", () => {
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
      packet.files.find((entry) => entry.file === removedFile)!.diagnostics.pop();
    },
    (packet) => {
      packet.files.find((entry) => entry.file === removedFile)!.diagnostics[0] += " changed";
    },
  ];
  for (const mutate of mutations) {
    const copy = structuredClone(successor);
    mutate(copy);
    assert.throws(() => assertJsMixinSuccessor(copy), assert.AssertionError);
  }
});
