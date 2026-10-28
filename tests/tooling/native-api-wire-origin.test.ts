import assert from "node:assert/strict";
import test from "node:test";
import { verifyBuffers } from "../../crates/vize_canon/src/binding_origins/native_api_probe/verify-wire.ts";
import {
  array,
  first,
  object,
} from "../../crates/vize_canon/src/binding_origins/native_api_probe/verify-codec.ts";
import { encode, fixture, frame } from "./native-api-wire-origin-fixture.ts";
import type { Fixture } from "./native-api-wire-origin-fixture.ts";

function run(version: "7.0.2" | "7.1.0-beta", mutate?: (data: Fixture) => void) {
  const data = fixture(version);
  mutate?.(data);
  const client = data.clientRaw ?? encode(data.requests),
    native = data.nativeRaw ?? encode(data.responses);
  const carrier = {
    ...data.carrier,
    forwardedClientBytes: client.length,
    observedNativeStdoutBytes: native.length,
  };
  const emitted = new Map<string, unknown>();
  const receipt = verifyBuffers(client, native, carrier, version, (name, value) =>
    emitted.set(name, value),
  );
  return { receipt, emitted };
}
for (const version of ["7.0.2", "7.1.0-beta"] as const) {
  test("synthetic wire custody preserves complete packets for " + version, () => {
    const result = run(version);
    assert.equal(result.receipt.passed, true);
    const firstPacket = object(first(array(result.emitted.get("full-wire-symbol-packets.json"))));
    assert.deepEqual(object(object(firstPacket.response).result).extra, {
      preserved: "🦀",
      array: [null, true, 12.5],
    });
  });
  const controls: [string, (data: Fixture) => void][] = [
    ["duplicate-response", (data) => data.responses.push(structuredClone(first(data.responses)))],
    [
      "missing-response",
      (data) => {
        data.responses.pop();
      },
    ],
    [
      "duplicate-request",
      (data) => {
        const second = data.requests.at(1);
        assert.ok(second);
        second.id = first(data.requests).id;
      },
    ],
    [
      "native-error",
      (data) => {
        first(data.responses).error = { code: -1 };
      },
    ],
    [
      "changed-repeat-extra",
      (data) => {
        const repeated = data.responses.at(2);
        assert.ok(repeated);
        object(object(repeated.result).extra).preserved = "changed";
      },
    ],
    [
      "unknown-ref-kind",
      (data) => {
        object(first(data.responses).result).declarations = ["1.99999./original/probe.ts"];
      },
    ],
    [
      "opposite-version-kind",
      (data) => {
        object(first(data.responses).result).declarations = [
          "1." + (version === "7.0.2" ? 278 : 277) + "./original/probe.ts",
        ];
      },
    ],
    [
      "wrong-image",
      (data) => {
        data.carrier.backendSha256 = "0".repeat(64);
      },
    ],
    [
      "lost-partial-parse",
      (data) => {
        const phase = data.carrier.phases.at(1);
        assert.ok(phase);
        phase.syntaxComplete = true;
      },
    ],
    [
      "truncated-header",
      (data) => {
        data.clientRaw = Buffer.from("Content-Length: 10\r\n");
      },
    ],
    [
      "truncated-body",
      (data) => {
        data.nativeRaw = Buffer.from("Content-Length: 50\r\n\r\n{}");
      },
    ],
    [
      "duplicate-json-field",
      (data) => {
        data.clientRaw = frame(Buffer.from('{"id":1,"id":1}'));
      },
    ],
    [
      "escaped-duplicate-json-field",
      (data) => {
        data.clientRaw = frame(Buffer.from(String.raw`{"id":1,"id":1}`));
      },
    ],
    [
      "invalid-utf8",
      (data) => {
        data.nativeRaw = frame(Buffer.from([255]));
      },
    ],
    [
      "missing-target-query",
      (data) => {
        const request = data.requests.at(1);
        assert.ok(request);
        request.method = "unexpected";
      },
    ],
    [
      "number-string-id",
      (data) => {
        first(data.responses).id = String(first(data.responses).id);
      },
    ],
    [
      "unsafe-number",
      (data) => {
        data.clientRaw = frame(Buffer.from('{"id":9007199254740993}'));
      },
    ],
    [
      "utf8-bom",
      (data) => {
        data.clientRaw = frame(Buffer.concat([Buffer.from([239, 187, 191]), Buffer.from("{}")]));
      },
    ],
  ];
  for (const [name, mutate] of controls) {
    test("synthetic wire refuses " + name + " for " + version, () => {
      assert.throws(() => run(version, mutate));
    });
  }
}
