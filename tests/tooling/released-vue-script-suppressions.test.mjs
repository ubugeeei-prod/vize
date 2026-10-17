import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import test from "node:test";
import { sha256 } from "../../tools/support/release/typed_defaults/custody.mjs";
import {
  fixtureSha256,
  loadFixtures,
} from "../../tools/support/release/vue_script_suppressions/fixtures.mjs";
import {
  prepareOutput,
  rejectOverrides,
} from "../../tools/support/release/vue_script_suppressions/main.mjs";
import { verifyPacket } from "../../tools/support/release/vue_script_suppressions/observe.mjs";

await test("one immutable packet expands all four real-Linter regression contracts", () => {
  const { fixture, bytes } = loadFixtures();
  assert.equal(sha256(bytes), fixtureSha256);
  assert.equal(fixture.authority.commit, "8f2e12d8519205bf2b5eae2db0e5e323a45a57d0");
  assert.equal(fixture.authority.tests.length, 4);
  const counts = fixture.authority.tests.map(
    (name) => fixture.cases.filter(({ sourceTest }) => sourceTest === name).length,
  );
  assert.deepEqual(counts, [12, 48, 8, 4]);
});

await test("public custody refuses every runtime override before installation", () => {
  rejectOverrides({});
  for (const name of [
    "VIZE_BINDING_PATH",
    "NODE_OPTIONS",
    "NODE_PATH",
    "CORSA_PATH",
    "VIZE_BINARY",
    "SOURCE_SHA",
    "NAPI_RS_FORCE_WASI",
  ]) {
    assert.throws(() => rejectOverrides({ [name]: "injected" }), /runtime override must be absent/);
  }
});

await test("existing output is never reused or overwritten", () => {
  const parent = fs.mkdtempSync(path.join(os.tmpdir(), "vize-public-suppressions-refusal-"));
  try {
    const output = path.join(parent, "evidence");
    fs.mkdirSync(output);
    fs.writeFileSync(path.join(output, "sentinel"), "keep whole failed evidence");
    assert.throws(() => prepareOutput(output), { code: "EEXIST" });
    assert.equal(
      fs.readFileSync(path.join(output, "sentinel"), "utf8"),
      "keep whole failed evidence",
    );
    assert.deepEqual(fs.readdirSync(output), ["sentinel"]);
  } finally {
    fs.rmSync(parent, { recursive: true });
  }
});

await test("derived law refuses foreign findings, process failure, and input mutation", () => {
  const sourceCase = { id: "unit-only", filename: "Control.vue", expectedRuleIds: ["script/a"] };
  const actual = [
    {
      file: "Control.vue",
      messages: [{ ruleId: "script/a", severity: 2 }],
      errorCount: 1,
      warningCount: 0,
    },
  ];
  const packet = {
    rawCaptureComplete: true,
    error: null,
    signal: null,
    status: 1,
    stderrBase64: "",
    stdoutBase64: Buffer.from(JSON.stringify(actual)).toString("base64"),
    inputs: [
      {
        name: "Control.vue",
        readError: null,
        text: "original",
        afterText: "original",
        sha256: "same",
        afterSha256: "same",
      },
    ],
  };
  assert.deepEqual(verifyPacket(sourceCase, packet), actual);
  const foreign = structuredClone(actual);
  foreign[0].messages.push({ ruleId: "script/foreign", severity: 2 });
  assert.throws(() =>
    verifyPacket(sourceCase, {
      ...packet,
      stdoutBase64: Buffer.from(JSON.stringify(foreign)).toString("base64"),
    }),
  );
  assert.throws(() => verifyPacket(sourceCase, { ...packet, signal: "SIGTERM" }));
  assert.throws(() => verifyPacket(sourceCase, { ...packet, rawCaptureComplete: false }));
  assert.throws(() => verifyPacket(sourceCase, { ...packet, status: 0 }));
  assert.throws(() =>
    verifyPacket(sourceCase, {
      ...packet,
      stdoutBase64: Buffer.from("not json").toString("base64"),
    }),
  );
  assert.throws(() =>
    verifyPacket(sourceCase, {
      ...packet,
      inputs: [{ ...packet.inputs[0], afterText: "rewritten" }],
    }),
  );
  assert.throws(() =>
    verifyPacket(sourceCase, {
      ...packet,
      inputs: [{ ...packet.inputs[0], readError: { name: "ENOENT" } }],
    }),
  );
});
