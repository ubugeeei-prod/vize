import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { asRecord, asString, parseJson } from "../../json-value.ts";
import { sha256 } from "../../config/windows_consumer/common.ts";
import { macOriginal, requiredFix } from "./mac_contract.ts";

/** All 30 original packets are retained, including every unknown observation field. */
export function macReference() {
  const value = asRecord(
    parseJson(
      readFileSync(
        new URL(
          "../../../../tests/tooling/fixtures/formatter-profiles-public-mac-reference.json",
          import.meta.url,
        ),
        "utf8",
      ),
    ),
  );
  assert.equal(value.schema, "vize.formatterMac11264.fullSourceReference.v1");
  const origin = asRecord(value.origin);
  assert.equal(origin.head, "3b2e6f315dc87570a05fafe9b901959de29a4c7a");
  assert.equal(origin.runnerSource, "96c6a1252eb329913b46f3eb782ec3520d94d0a9");
  assert.equal(origin.actualMerge, requiredFix);
  assert.equal(origin.run, 41273511930);
  assert.equal(origin.job, 124751267841);
  assert.equal(origin.artifactId, 13070104149);
  assert.equal(
    origin.archiveSha256,
    "d47842542688d4fad8726c95d1ff1af413239e8d80a5c8c50eb7cb2a92cecd9b",
  );
  assert.equal(
    origin.junitSha256,
    "fff95a06f899a18815d6252826080468a57af3308f39282841cafb1276c75cf8",
  );
  const raw = Buffer.from(asString(value.streamRawBase64), "base64");
  assert.equal(raw.toString("base64"), value.streamRawBase64);
  assert.equal(raw.length, 16093);
  assert.equal(value.streamBytes, raw.length);
  assert.equal(sha256(raw), "ec9f5a8d6c098fc93f9af7809fbc7459499597d24abb8fd8509fbb354ab0ebd9");
  assert.equal(value.streamSha256, sha256(raw));
  const stream = new TextDecoder("utf-8", { fatal: true }).decode(raw);
  assert(stream.endsWith("\n"));
  const prefix = "formatter-separated-bools-process:";
  const packets = stream
    .slice(0, -1)
    .split("\n")
    .map((line) => {
      assert(line.startsWith(prefix));
      return asRecord(parseJson(line.slice(prefix.length)));
    });
  assert.equal(packets.length, 30);
  const original = macOriginal();
  const selected = packets.filter(
    (packet) => JSON.stringify(packet.argv) === JSON.stringify(original.equals),
  );
  assert.equal(selected.length, 2, "original equals first write and rewrite");
  const outputs = selected.map((packet) => {
    assert.equal(packet.status, 0);
    assert.equal(packet.success, true);
    assert.equal(packet.stdout, "");
    assert.deepEqual(Object.keys(asRecord(packet.files)), ["a.vue"]);
    return Buffer.from(asString(asRecord(packet.files)["a.vue"]));
  });
  assert.deepEqual(outputs[0], outputs[1]);
  const output = outputs[0]!;
  assert.equal(sha256(output), "baa8e7b1a02bfd36727815527ebc96435f31d281db7dfcdfc0fc3bfa054a4662");
  assert.equal(value.outputSha256, sha256(output));
  const alternatives = [
    ...new Set(packets.map((packet) => asString(asRecord(packet.files)["a.vue"]))),
  ]
    .filter((text) => text !== output.toString("utf8"))
    .map((text) => Buffer.from(text));
  assert.equal(alternatives.length, 4);
  return { value, packets, output, alternatives };
}

export function assertFullFalse(actual: Buffer, reference: Buffer): void {
  assert.deepEqual(
    actual,
    reference,
    "all three disabled options require the entire authenticated false output",
  );
}
