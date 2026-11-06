import assert from "node:assert/strict";
import { test } from "node:test";
import { inflightArgs, inflightFixture } from "./support/native-beta-inflight-warning-fixture.ts";
import { historicalWarningTranscript } from "./support/native-beta-dto/historical-inflight-transcript.ts";
import { observeRegisteredUnavailableRootWarning } from "./support/native-beta-dto/historical-registered-unavailable-root-warning.ts";
import { observeRootWatchWarning } from "./support/native-beta-dto/historical-acknowledged-root-warning.ts";
import { buildRoleDisposition } from "./support/native-beta-dto/role-disposition.ts";
import { at, type Packet } from "./support/native-beta-root-warning-fixture.ts";

function registeredUnavailableArgs() {
  const fixture = inflightFixture();
  const config = "/tmp/.tmpUnit01/tsconfig.json";
  fixture.server.push({
    at: at(12),
    packet: {
      jsonrpc: "2.0",
      method: "window/logMessage",
      params: {
        type: 3,
        message: `Added new watch: root files for ${config} watcher 6.0`,
      },
    },
  });
  fixture.server.sort((left, right) => left.at.localeCompare(right.at));
  fixture.stderr.splice(0, fixture.stderr.length, { at: at(15), data: Buffer.from("\n") });
  fixture.owner.unknownWrites[0]!.at = at(17);
  fixture.owner.unknownWrites[0]!.line = 303;
  fixture.owner.unknownWrites[0]!.startedLine = 303;
  return inflightArgs(fixture);
}
type Args = ReturnType<typeof registeredUnavailableArgs>;
const object = (value: unknown) => value as Packet;

test("registered watch log plus unavailable warning remains a separate zero-credit refusal", () => {
  const args = registeredUnavailableArgs();
  assert.throws(() => historicalWarningTranscript(args[0], args[1], args[2], args[6]));
  const result = observeRootWatchWarning(...args);
  assert.deepEqual(result, observeRegisteredUnavailableRootWarning(...args));
  assert.equal(result.kind, "registered-unavailable-root-warning");
  assert.equal(result.completedStderr.bytes, 1);
  assert.equal(result.completedStderr.hex, "0a");
  assert.equal(result.unknownAttempts[0]!.sourceRow.requestedBytes, 32);
  for (const key of [
    "nativeQualification",
    "candidateFunctionalPass",
    "lifecyclePass",
    "registrationPass",
    "stderrOutputComplete",
    "rawIOComplete",
  ] as const)
    assert.equal(result[key], false);
  assert.equal(result.observedRootRegistrationAcknowledged, true);
  const disposition = buildRoleDisposition({
    source: args[2].source,
    role: args[2].role,
    sdk: args[2].sdkVersion,
    baseline: false,
    stableRegistrationBaseline: true,
    registrationRefusals: 0,
    nativeOwners: 1,
    watcher: [],
    split: [],
    terminal: [],
    rootWarnings: [{ observation: result }],
  });
  for (const key of [
    "nativeQualification",
    "dtoFunctionalPass",
    "lifecyclePass",
    "registrationPass",
  ] as const)
    assert.equal(disposition[key], false);
});

test("registered unavailable observer rejects channel, body, owner and order mutants", () => {
  const mutations: Array<[string, (args: Args) => void]> = [
    [
      "wrong log",
      (args) => {
        const log = args[1].find((packet) => object(packet).method === "window/logMessage")!;
        object(object(log).params).message = "Added new watch: foreign";
      },
    ],
    [
      "extra stderr",
      (args) => {
        args[3].push({ ...args[3].find((row) => row.fd === 2)!, line: 302 });
      },
    ],
    [
      "wrong stderr",
      (args) => {
        args[3].find((row) => row.fd === 2)!.data = Buffer.from("header\n");
      },
    ],
    [
      "wrong warning",
      (args) => {
        args[5].unknownWrites[0]!.data = Buffer.from("other");
      },
    ],
    [
      "completed warning",
      (args) => {
        args[5].unknownWrites[0]!.rawResult = "32";
      },
    ],
    [
      "foreign owner",
      (args) => {
        args[5].unknownWrites[0]!.tid = 999;
      },
    ],
    [
      "warning before newline completion",
      (args) => {
        args[5].unknownWrites[0]!.startedLine = 299;
        args[5].unknownWrites[0]!.at = at(15);
      },
    ],
    [
      "warning after exit request",
      (args) => {
        args[5].unknownWrites[0]!.at = at(19);
      },
    ],
  ];
  for (const [name, mutate] of mutations) {
    const args = registeredUnavailableArgs();
    mutate(args);
    assert.throws(
      () => observeRegisteredUnavailableRootWarning(...args),
      assert.AssertionError,
      name,
    );
  }
});
