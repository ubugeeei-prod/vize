import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import { test } from "node:test";

const fixture = new URL(
  "../_fixtures/differential/typechecker/vue2-rename-context-custody/",
  import.meta.url,
);
const bytes = (name) => fs.readFileSync(new URL(name, fixture));
const read = (name) => JSON.parse(bytes(name).toString("utf8"));
const sha256 = (value) => createHash("sha256").update(value).digest("hex");
const custody = read("custody.json");
const historical = read("historical.json");
const successor = read("successor.json");
const source = bytes("TransactionTable.vue.txt").toString("utf8");
const displayCustody = read("display-custody.json");
const displaySuccessor = read("display-successor.json");
const priorScriptMessage =
  "error:50:14 [TS2339] Property 'list' does not exist on type '__VizeThis'.";
const currentScriptMessage = "error:50:14 [TS2339] Property 'list' does not exist on type '$This'.";

function assertSuccessor(previous, current) {
  const sourceLine = source.split("\n")[1];
  assert.equal(sourceLine.slice(19, 23), "list", "original authored template range");
  assert.equal(source.split("\n")[49].slice(13, 17), "list", "original authored script range");
  assert.equal(sourceLine.trim(), '<el-table :data="list" style="width: 100%;padding-top: 15px;">');
  const expected = structuredClone(previous);
  for (const name of ["rename", "dataKey"]) {
    const canonical = expected[name].files[0].diagnostics[0];
    assert.equal(
      canonical,
      "error:2:20 [TS2339] Property 'list' does not exist on the component instance.",
    );
    // Construct the reviewed presentation addition from an independent pinned
    // upstream line. Never remove context from the current diagnostic packet.
    expected[name].files[0].diagnostics[0] =
      `${canonical} (source: ${sourceLine.trim()}; binding: 'data')`;
  }
  assert.deepEqual(
    current,
    expected,
    "the complete original packets plus one reviewed context per broken phase",
  );
}

void test("Vue2 rename custody preserves historical expectations, source and complete successor packets", () => {
  assert.equal(
    sha256(bytes("custody.json")),
    "be83444f06dcff0dbcd6aad8ccdbc0f52ae38419f9a3cc8d7a3d1331ede9beeb",
  );
  assert.equal(custody.schema, "vize.vue2RenameContextCustody");
  assert.equal(custody.version, 1);
  assert.equal(custody.sourceRevision, "284c7c4f6a7a1a50d9acec8d877c47aae21cfe42");
  assert.equal(custody.producer.workflowRun, 40171762851);
  assert.equal(custody.producer.job, 121103891221);
  assert.equal(custody.producer.artifact, 12591225041);
  assert.equal(custody.sourceBuild.sourceRevision, custody.sourceRevision);
  assert.equal(custody.upstream.revision, "6858a9ad67483025f6a9432a926beb9327037be3");
  assert.equal(custody.upstream.license, "MIT");
  assert.equal(sha256(bytes("LICENSE")), custody.upstream.licenseSha256);
  assert.equal(
    sha256(bytes("TransactionTable.vue.txt")),
    "4f32ac35959e8e062b4bfe535523bb5c003ebb9ff783a10eae5f22db869572f3",
  );
  assert.equal(sha256(bytes("TransactionTable.vue.txt")), custody.upstream.sha256);
  assert.equal(
    sha256(bytes("historical.json")),
    "68129b8221ac20f9f9c57169d5e89dbf9f14da7b62e28adcbcbb247347e2f422",
  );
  assert.equal(sha256(bytes("historical.json")), custody.historical.sha256);
  assert.equal(
    sha256(bytes("successor.json")),
    "83f580c929b91a3ded1ff3c61cc53b39e61b030c14f46518c82c7dffdbfbb7dd",
  );
  assert.equal(sha256(bytes("successor.json")), custody.successor.sha256);
  assert.equal(custody.historical.sources.length, 2);
  for (const original of custody.historical.sources) {
    assert.equal(sha256(bytes(original.archived)), original.sha256);
  }
  assert.deepEqual(
    { variants: custody.variants, phases: custody.phases },
    { variants: 4, phases: 3 },
  );
  assert.deepEqual(
    Object.values(historical).map((packet) => ({
      files: packet.fileCount,
      errors: packet.errorCount,
      warnings: packet.warningCount,
    })),
    [
      { files: 1, errors: 0, warnings: 0 },
      { files: 1, errors: 1, warnings: 0 },
      { files: 1, errors: 2, warnings: 0 },
    ],
  );
  assertSuccessor(historical, successor);
});

void test("Vue2 successor custody rejects source, binding, canonical, coordinate and inventory mutations", () => {
  for (const mutate of [
    (packet) => {
      packet.rename.files[0].diagnostics[0] = packet.rename.files[0].diagnostics[0].replace(
        "source:",
        "changed:",
      );
    },
    (packet) => {
      packet.rename.files[0].diagnostics[0] = packet.rename.files[0].diagnostics[0].replace(
        "binding: 'data'",
        "binding: 'other'",
      );
    },
    (packet) => {
      packet.rename.files[0].diagnostics[0] = packet.rename.files[0].diagnostics[0].replace(
        "Property 'list'",
        "Property 'other'",
      );
    },
    (packet) => {
      packet.rename.files[0].diagnostics[0] = packet.rename.files[0].diagnostics[0].replace(
        "error:2:20",
        "error:3:21",
      );
    },
    (packet) => {
      packet.dataKey.files[0].diagnostics[1] += " altered script message";
    },
    (packet) => {
      packet.dataKey.files[0].diagnostics.reverse();
    },
    (packet) => {
      packet.dataKey.files[0].diagnostics.pop();
    },
    (packet) => {
      packet.dataKey.errorCount = 1;
    },
    (packet) => {
      packet.rename.files[0].file += ".unexpected";
    },
    (packet) => {
      packet.clean.files[0].diagnostics.push("error:1:1 [TS9999] unexpected");
    },
  ]) {
    const mutated = structuredClone(successor);
    mutate(mutated);
    assert.throws(() => assertSuccessor(historical, mutated), assert.AssertionError);
  }
});

function assertDisplaySuccessor(previous, current) {
  const expected = structuredClone(previous);
  assert.equal(expected.dataKey.files[0].diagnostics[1], priorScriptMessage);
  expected.dataKey.files[0].diagnostics[1] = currentScriptMessage;
  assert.deepEqual(current, expected, "only the one reviewed script type display changes");
}

void test("Vue2 edit-cycle display successor preserves both archived packets and complete phases", () => {
  assert.equal(
    sha256(bytes("display-custody.json")),
    "0f8668f3a9dd01471cc74c27e3534de783966c404723e2c73d25d710c5230f5a",
  );
  assert.equal(displayCustody.schema, "vize.vue2RenameDisplayCustody");
  assert.equal(displayCustody.version, 1);
  assert.equal(displayCustody.sourceRevision, "09cc1727c687fa2baa30ecd1dcab854ec72ca597");
  assert.equal(displayCustody.displaySourceRevision, "32d80c319cfb413194ac934e160451a357643e38");
  assert.deepEqual(displayCustody.producer, {
    workflowRun: 41112686389,
    job: 124258822990,
    artifactId: 13003633335,
    artifact: "vue2-editor-context-09cc1727c687fa2baa30ecd1dcab854ec72ca597-41112686389",
    artifactSha256: "f480ea7bf1635fbef9bec683882afeb0b000b36db7ab515e12051f27dd2b783c",
    recipe: "cargo build --profile ci -p vize --features legacy",
    binarySha256: "1caa99ba176467b6f05ac78977dcda933741c4e79d1e7f1264f7638100bd560c",
    jobLogSha256: "0b149f9344c6cf0885732e4376b141c4e77683a98f1799c41522e52dc9c2f90d",
  });
  assert.deepEqual(displayCustody.previous, {
    path: "successor.json",
    sha256: "83f580c929b91a3ded1ff3c61cc53b39e61b030c14f46518c82c7dffdbfbb7dd",
  });
  assert.deepEqual(displayCustody.successor, {
    path: "display-successor.json",
    sha256: "afc64c945a0295389696d3530f0d12bf0561a4e1970e1d3300a6d6e05001eca9",
  });
  assert.deepEqual(displayCustody.changes, [
    {
      phase: "dataKey",
      fileIndex: 0,
      diagnosticIndex: 1,
      before: priorScriptMessage,
      after: currentScriptMessage,
    },
  ]);
  assert.equal(sha256(bytes(displayCustody.previous.path)), displayCustody.previous.sha256);
  assert.equal(sha256(bytes(displayCustody.successor.path)), displayCustody.successor.sha256);
  assertSuccessor(historical, successor);
  assertDisplaySuccessor(successor, displaySuccessor);
});

void test("Vue2 edit-cycle display custody rejects arbitrary message, range and inventory changes", () => {
  for (const mutate of [
    (packet) => {
      packet.dataKey.files[0].diagnostics[1] = priorScriptMessage;
    },
    (packet) => {
      packet.dataKey.files[0].diagnostics[1] =
        "error:50:14 [TS2339] Property 'other' does not exist on type '$This'.";
    },
    (packet) => {
      packet.dataKey.files[0].diagnostics[1] =
        "error:51:15 [TS2339] Property 'list' does not exist on type '$This'.";
    },
    (packet) => {
      packet.dataKey.files[0].diagnostics[1] =
        "error:50:14 [TS9999] Property 'list' does not exist on type '$This'.";
    },
    (packet) => {
      packet.dataKey.files[0].diagnostics[0] += " foreign context";
    },
    (packet) => {
      packet.dataKey.files[0].diagnostics.reverse();
    },
    (packet) => {
      packet.dataKey.files[0].diagnostics.pop();
    },
    (packet) => {
      packet.dataKey.errorCount = 1;
    },
    (packet) => {
      packet.dataKey.files[0].file += ".foreign";
    },
    (packet) => {
      packet.clean.files[0].diagnostics.push("error:1:1 [TS9999] unexpected");
    },
    (packet) => {
      packet.rename.files[0].diagnostics[0] += " changed";
    },
    (packet) => {
      packet.extra = "opaque unexpected field";
    },
  ]) {
    const mutated = structuredClone(displaySuccessor);
    mutate(mutated);
    assert.notDeepEqual(mutated, displaySuccessor, "each control changes the whole packet");
    assert.throws(() => assertDisplaySuccessor(successor, mutated), assert.AssertionError);
  }
});
