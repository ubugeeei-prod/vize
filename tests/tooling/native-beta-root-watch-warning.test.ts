import assert from "node:assert/strict";
import { test } from "node:test";
import { fixture, observe, at, type Packet } from "./support/native-beta-root-warning-fixture.ts";
import { requireNoUnknownNativeWrites } from "./support/native-beta-dto/native-io.ts";

const object = (value: unknown) => value as Packet;
test("complete warning retains its exact unanswered SDK watcher and unproven EOF relation", () => {
  const f = fixture("A"),
    result = observe(f);
  assert.equal(result.kind, "completed-warning-unanswered-sdk");
  assert.equal(result.completedStderr.bytes, 32);
  assert.equal(result.observedPairs.lsp.server.unanswered.length, 1);
  assert.deepEqual(result.unansweredRegistration, f.server.at(-1)!.packet);
  assert.equal(result.observedPairs.api.client.responses, 2);
  assert.deepEqual(
    result.stderrWrites.map((row) => row.eofReturnOrder),
    ["UNPROVEN"],
  );
  for (const key of [
    "nativeQualification",
    "candidateFunctionalPass",
    "lifecyclePass",
    "registrationPass",
  ] as const)
    assert.equal(result[key], false);
});
test("terminal warning keeps completed root header separate from the unknown 32-byte attempt", () => {
  const f = fixture("B"),
    result = observe(f);
  assert.equal(result.kind, "unobserved-warning-attempt");
  assert.equal(result.observedPairs.lsp.server.unanswered.length, 0);
  assert.equal(result.unknownAttempts[0]!.requestedBytes, 32);
  assert.equal(result.unknownAttempts[0]!.returnedBytes, null);
  assert.equal(result.unknownAttempts[0]!.deliveredBytes, null);
  assert.equal(result.unknownAttempts[0]!.actualUnknownReturnTime, null);
  assert.equal(result.unknownAttempts[0]!.eofReturnOrder, "UNPROVEN");
  for (const key of [
    "nativeQualification",
    "candidateFunctionalPass",
    "lifecyclePass",
    "registrationPass",
    "stderrOutputComplete",
    "rawIOComplete",
  ] as const)
    assert.equal(result[key], false);
  assert.throws(() => requireNoUnknownNativeWrites(f.owner.unknownWrites));
  assert.deepEqual(
    result.clientPackets,
    f.client.map((row) => row.packet),
  );
  assert.deepEqual(result.api, f.api);
});
test("source, SDK, owner, root, payload, paired channels and temporal counterexamples refuse", () => {
  type F = ReturnType<typeof fixture>;
  const common: Array<[string, (f: F) => void]> = [
    [
      "after",
      (f) => {
        f.context.role = "after";
      },
    ],
    [
      "source",
      (f) => {
        f.context.source = "0".repeat(40);
      },
    ],
    [
      "beta",
      (f) => {
        f.context.sdkVersion = "7.1.0-beta";
      },
    ],
    [
      "image",
      (f) => {
        f.context.sdkSha256 = "0".repeat(64);
      },
    ],
    [
      "foreign root",
      (f) => {
        f.context.projectRoots = ["/tmp/foreign"];
      },
    ],
    [
      "owner",
      (f) => {
        f.owner.nativeOwner = 999;
      },
    ],
    [
      "missing exit",
      (f) => {
        f.owner.threadExits.pop();
      },
    ],
    [
      "nonzero exit",
      (f) => {
        f.owner.threadExits[0]!.code = 1;
      },
    ],
    [
      "cycle",
      (f) => {
        f.owner.members[1]!.parent = 44;
      },
    ],
    [
      "wrong root ACK",
      (f) => {
        f.client.at(-1)!.packet.result = "wrong";
      },
    ],
    [
      "wrong root glob",
      (f) => {
        object(f.server[3]!.packet.params).registrations = [];
      },
    ],
    [
      "wrong initialized image label",
      (f) => {
        object(f.server[0]!.packet.result).serverInfo = { name: "typescript", version: "7.0.2" };
      },
    ],
    [
      "API source path",
      (f) => {
        object(object(f.api.client[1]).params).file = "/tmp/foreign/tsconfig.json";
      },
    ],
    [
      "API extra response",
      (f) => {
        f.api.server = [...f.api.server, { jsonrpc: "2.0", id: 999, result: null }];
      },
    ],
    [
      "API error",
      (f) => {
        object(f.api.server[1]).error = { code: -1 };
      },
    ],
    [
      "extra stderr",
      (f) => {
        f.stderr.push({ at: at(17), data: Buffer.from("other") });
      },
    ],
    [
      "stderr before EOF entry",
      (f) => {
        f.stderr[0]!.at = at(10);
      },
    ],
    [
      "observed EOF return after output",
      (f) => {
        f.eof[0]!.completion!.returnedAt = at(22);
      },
    ],
    [
      "forged bracket event",
      (f) => {
        f.eof[0]!.completion!.returnedBy!.at = at(22);
      },
    ],
    [
      "EOF owner",
      (f) => {
        f.eof[0]!.tid = 999;
      },
    ],
  ];
  for (const kind of ["A", "B"] as const)
    for (const [name, mutate] of common) {
      const f = fixture(kind);
      mutate(f);
      assert.throws(() => observe(f), `${kind}: ${name}`);
    }
  const source = fixture("A");
  assert.throws(() =>
    observe(source, (io) => {
      io.find((row) => row.fd === 2)!.tid = 999;
    }),
  );
});
test("neither a missing SDK request nor an invented cancellation is accepted as the complete-warning case", () => {
  const answered = fixture("A");
  answered.client.push({ packet: { jsonrpc: "2.0", id: "ts3", result: null }, at: at(16) });
  assert.throws(() => observe(answered));
  const missing = fixture("A");
  missing.server.pop();
  assert.throws(() => observe(missing));
  const reordered = fixture("A");
  [reordered.server[5], reordered.server[6]] = [reordered.server[6]!, reordered.server[5]!];
  assert.throws(() => observe(reordered));
  const wrongSDK = fixture("A");
  object(wrongSDK.server.at(-1)!.packet.params).registrations = [];
  assert.throws(() => observe(wrongSDK));
  const cancellation = fixture("A");
  cancellation.stderr[0]!.data = Buffer.from(
    "1 paths ineligible for watching\nerrors updating watches: invented\n",
  );
  assert.throws(() => observe(cancellation));
});
test("wrong terminal body, count, fd, delivery, marker, lifetime or extra attempt refuses", () => {
  type F = ReturnType<typeof fixture>;
  const changes: Array<[string, (f: F) => void]> = [
    [
      "body",
      (f) => {
        f.owner.unknownWrites[0]!.data = Buffer.from("other");
      },
    ],
    [
      "count",
      (f) => {
        f.owner.unknownWrites[0]!.requestedBytes = 31;
      },
    ],
    [
      "fd",
      (f) => {
        f.owner.unknownWrites[0]!.fd = 1;
      },
    ],
    [
      "marker",
      (f) => {
        f.owner.unknownWrites[0]!.rawResult = "? <unavailable>";
      },
    ],
    [
      "zero delivery",
      (f) => {
        Object.assign(f.owner.unknownWrites[0]!, { returnedBytes: 0 });
      },
    ],
    [
      "completed outcome",
      (f) => {
        Object.assign(f.owner.unknownWrites[0]!, { outcome: "completed" });
      },
    ],
    [
      "before exit request",
      (f) => {
        f.owner.unknownWrites[0]!.at = at(17);
      },
    ],
    [
      "after writer exit",
      (f) => {
        f.owner.unknownWrites[0]!.at = at(23);
      },
    ],
    [
      "foreign attempt",
      (f) => {
        f.owner.unknownWrites[0]!.tid = 999;
      },
    ],
    [
      "extra unknown",
      (f) => {
        f.owner.unknownWrites.push(f.owner.unknownWrites[0]!);
      },
    ],
    [
      "foreign unfinished",
      (f) => {
        f.owner.unfinishedOperations[0]!.tid = 44;
      },
    ],
    [
      "unknown IO hidden as unfinished",
      (f) => {
        f.owner.unfinishedOperations[0]!.raw = "write(2, missing";
      },
    ],
  ];
  for (const [name, mutate] of changes) {
    const f = fixture("B");
    mutate(f);
    assert.throws(() => observe(f), name);
  }
});
