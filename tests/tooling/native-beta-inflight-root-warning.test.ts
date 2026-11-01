import assert from "node:assert/strict";
import { test } from "node:test";
import { inflightFixture, inflightArgs } from "./support/native-beta-inflight-warning-fixture.ts";
import { fixture, observe, at, type Packet } from "./support/native-beta-root-warning-fixture.ts";
import { observeHistoricalRootWatchWarning } from "./support/native-beta-dto/historical-root-watch-warning.ts";
import {
  historicalInflightTranscript,
  historicalInflightLifecycle,
  observeRootWatchWarning,
} from "./support/native-beta-dto/historical-inflight-root-warning.ts";
import { requireNoUnknownNativeWrites } from "./support/native-beta-dto/native-io.ts";
import { buildRoleDisposition } from "./support/native-beta-dto/role-disposition.ts";

test("the inflight warning preserves null delivery, unproven EOF returns and zero qualification", () => {
  const args = inflightArgs(inflightFixture());
  assert.throws(() => observeHistoricalRootWatchWarning(...args));
  const result = observeRootWatchWarning(...args);
  assert.equal(result.kind, "inflight-unobserved-warning-attempt");
  assert.equal(result.observedRootRegistrationAcknowledged, true);
  assert.equal(result.unknownAttempts[0]!.requestedBytes, 32);
  for (const key of ["returnedBytes", "deliveredBytes", "actualUnknownReturnTime"] as const)
    assert.equal(result.unknownAttempts[0]![key], null);
  assert.deepEqual(
    [...result.stderrWrites, ...result.unknownAttempts].map((row) => row.eofReturnOrder),
    Array(4).fill("UNPROVEN"),
  );
  for (const key of [
    "nativeQualification",
    "candidateFunctionalPass",
    "lifecyclePass",
    "registrationPass",
    "rawIOComplete",
    "stderrOutputComplete",
  ] as const)
    assert.equal(result[key], false);
  assert.throws(() => requireNoUnknownNativeWrites(args[5].unknownWrites));
  assert.deepEqual(result.clientPackets, args[0]);
  assert.deepEqual(result.serverPackets, args[1]);
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
    "rawIOComplete",
    "stderrOutputComplete",
  ] as const)
    assert.equal(disposition[key], false);
  assert.equal(disposition.observedRootRegistrationAcknowledged, true);
});

test("old completed and post-exit observations retain their complete original outputs", () => {
  for (const kind of ["A", "B"] as const) {
    const f = fixture(kind),
      expected = observe(f);
    // The fixture's original observe path supplies its unchanged synthetic IO.
    let saved: Parameters<typeof observeRootWatchWarning>[3] | undefined;
    observe(f, (io) => {
      saved = io;
    });
    assert.ok(saved);
    assert.deepEqual(
      observeRootWatchWarning(
        f.client.map((row) => row.packet),
        f.server.map((row) => row.packet),
        f.context,
        saved,
        f.eof,
        f.owner,
        f.api,
      ),
      expected,
    );
  }
});

test("wrong source, role, image, actor, EOF, body, outcome and lifetime refuse", () => {
  type Args = Parameters<typeof observeRootWatchWarning>;
  const controls: Array<[string, (a: Args) => void]> = [
    [
      "after",
      (a) => {
        a[2].role = "after";
      },
    ],
    [
      "source",
      (a) => {
        a[2].source = "0".repeat(40);
      },
    ],
    [
      "beta",
      (a) => {
        a[2].sdkVersion = "7.1.0-beta";
      },
    ],
    [
      "image",
      (a) => {
        a[2].sdkSha256 = "0".repeat(64);
      },
    ],
    [
      "root",
      (a) => {
        a[2].projectRoots = ["/tmp/foreign"];
      },
    ],
    [
      "owner",
      (a) => {
        a[5].nativeOwner = 999;
      },
    ],
    [
      "cycle",
      (a) => {
        a[5].members[1]!.parent = 44;
      },
    ],
    [
      "missing exit",
      (a) => {
        a[5].threadExits.pop();
      },
    ],
    [
      "nonzero exit",
      (a) => {
        a[5].threadExits[0]!.code = 1;
      },
    ],
    [
      "group owner",
      (a) => {
        a[5].exitGroups[0]!.tid = 999;
      },
    ],
    [
      "nonzero group",
      (a) => {
        a[5].exitGroups[0]!.code = 1;
      },
    ],
    [
      "group result",
      (a) => {
        a[5].exitGroups[0]!.rawResult = "0";
      },
    ],
    [
      "extra group",
      (a) => {
        a[5].exitGroups.push(a[5].exitGroups[0]!);
      },
    ],
    [
      "attempt before shutdown completion",
      (a) => {
        a[5].unknownWrites[0]!.at = at(9);
      },
    ],
    [
      "attempt after writer exit",
      (a) => {
        a[5].unknownWrites[0]!.at = at(24);
      },
    ],
    [
      "foreign attempt",
      (a) => {
        a[5].unknownWrites[0]!.tid = 999;
      },
    ],
    [
      "attempt body",
      (a) => {
        a[5].unknownWrites[0]!.data = Buffer.from("other");
      },
    ],
    [
      "attempt count",
      (a) => {
        a[5].unknownWrites[0]!.requestedBytes = 31;
      },
    ],
    [
      "attempt fd",
      (a) => {
        a[5].unknownWrites[0]!.fd = 1;
      },
    ],
    [
      "attempt file",
      (a) => {
        a[5].unknownWrites[0]!.file = "syscalls.999";
      },
    ],
    [
      "attempt marker",
      (a) => {
        a[5].unknownWrites[0]!.rawResult = "? <unavailable>";
      },
    ],
    [
      "invented completion",
      (a) => {
        a[5].unknownWrites[0]!.completion = { returnedAt: at(18), returnedBy: null };
      },
    ],
    [
      "invented zero delivery",
      (a) => {
        Object.assign(a[5].unknownWrites[0]!, { returnedBytes: 0 });
      },
    ],
    [
      "extra unknown",
      (a) => {
        a[5].unknownWrites.push(a[5].unknownWrites[0]!);
      },
    ],
    [
      "failed write",
      (a) => {
        a[5].failedWrites.push({ ...a[5].unknownWrites[0]!, outcome: "failed", returnedBytes: -1 });
      },
    ],
    [
      "unfinished",
      (a) => {
        a[5].unfinishedOperations.push({
          tid: 41,
          file: "syscalls.41",
          line: 400,
          at: at(20),
          raw: "unknown",
        });
      },
    ],
    [
      "EOF owner",
      (a) => {
        a[4][0]!.tid = 999;
      },
    ],
    [
      "observed EOF return",
      (a) => {
        a[4][0]!.completion!.returnedAt = at(24);
      },
    ],
    [
      "invented EOF upper",
      (a) => {
        a[4][0]!.completion!.returnedBy!.at = at(22);
      },
    ],
    [
      "EOF upper wrong line",
      (a) => {
        a[4][0]!.completion!.returnedBy!.line = 599;
      },
    ],
    [
      "missing EOF",
      (a) => {
        (a[4] as unknown[]).pop();
      },
    ],
    [
      "extra completed stderr",
      (a) => {
        (a[3] as unknown[]).push({
          ...a[3].find((row) => row.fd === 2)!,
          data: Buffer.from("extra"),
        });
      },
    ],
    [
      "stderr before shutdown ACK completion",
      (a) => {
        a[3].find((row) => row.fd === 2)!.at = at(9);
      },
    ],
    [
      "stderr before EOF entry",
      (a) => {
        a[3].find((row) => row.fd === 2)!.at = at(10);
      },
    ],
    [
      "stderr foreign writer",
      (a) => {
        a[3].find((row) => row.fd === 2)!.tid = 42;
      },
    ],
    [
      "stderr after attempt",
      (a) => {
        a[3].find((row) => row.fd === 2)!.line = 304;
      },
    ],
    [
      "timed RPC missing",
      (a) => {
        (a[3] as unknown[]).splice(0, 1);
      },
    ],
    [
      "whole RPC extra",
      (a) => {
        (a[1] as unknown[]).push({ jsonrpc: "2.0", id: 999, result: null });
      },
    ],
    [
      "API extra",
      (a) => {
        a[6].server = [...a[6].server, { jsonrpc: "2.0", id: 999, result: null }];
      },
    ],
    [
      "API wrong config",
      (a) => {
        ((a[6].client[1] as Packet).params as Packet).file = "/tmp/foreign/tsconfig.json";
      },
    ],
  ];
  for (const [name, mutate] of controls) {
    const args = inflightArgs(inflightFixture());
    mutate(args);
    assert.throws(() => observeRootWatchWarning(...args), name);
  }
});

test("bounded causal proof is explicitly separate from whole timed observer equality", () => {
  const args = inflightArgs(inflightFixture());
  const proof = historicalInflightTranscript(args[0], args[1], args[2], args[6]);
  const config = (args[1] as Packet[]).find((row) => row.id === "ts1")!;
  const subset = args[3].filter(
    (row) => !Buffer.from(row.data).includes(Buffer.from(JSON.stringify(config))),
  );
  historicalInflightLifecycle(proof, subset, args[4], args[5]);
  assert.throws(() =>
    observeRootWatchWarning(args[0], args[1], args[2], subset, args[4], args[5], args[6]),
  );
});

test("coherently changed whole RPC payloads, ACKs, source roots and extra watchers refuse", () => {
  type F = ReturnType<typeof inflightFixture>;
  const object = (value: unknown) => value as Packet;
  const controls: Array<[string, (f: F) => void]> = [
    [
      "root glob",
      (f) => {
        const rows = object(f.server.find((row) => row.packet.id === "ts2")!.packet.params)
          .registrations as Packet[];
        const watchers = object(rows[0]!.registerOptions).watchers as Packet[];
        watchers[0]!.globPattern = "/tmp/foreign/**/*";
      },
    ],
    [
      "root ACK",
      (f) => {
        f.client.find((row) => row.packet.id === "ts2")!.packet.result = "wrong";
      },
    ],
    [
      "configuration registration",
      (f) => {
        object(f.server.find((row) => row.packet.id === "ts1")!.packet.params).registrations = [];
      },
    ],
    [
      "missing configuration registration",
      (f) => {
        f.server = f.server.filter((row) => row.packet.id !== "ts1");
        f.client = f.client.filter((row) => row.packet.id !== "ts1");
      },
    ],
    [
      "wrong server image label",
      (f) => {
        object(f.server[0]!.packet.result).serverInfo = { name: "typescript", version: "7.0.2" };
      },
    ],
    [
      "foreign project",
      (f) => {
        f.server.find((row) => row.packet.id === 3)!.packet.result = {
          configFilePath: "/tmp/foreign/tsconfig.json",
        };
      },
    ],
    [
      "extra unanswered watcher",
      (f) => {
        f.server.push({
          at: at(10),
          packet: { ...f.server.find((row) => row.packet.id === "ts2")!.packet, id: "ts3" },
        });
      },
    ],
  ];
  for (const [name, mutate] of controls) {
    const f = inflightFixture();
    mutate(f);
    assert.throws(() => observeRootWatchWarning(...inflightArgs(f)), name);
  }
  const args = inflightArgs(inflightFixture());
  const ack = args[3].find(
    (row) => row.op === "write" && Buffer.from(row.data).includes(Buffer.from('"id":4')),
  )!;
  ack.completion!.returnedBy!.at = at(12);
  assert.throws(
    () => observeRootWatchWarning(...args),
    "shutdown ACK completion must be proven before the first stderr entry",
  );
  const beforeHeader = inflightArgs(inflightFixture());
  beforeHeader[5].unknownWrites[0]!.at = at(13);
  assert.throws(
    () => observeRootWatchWarning(...beforeHeader),
    "attempt cannot precede a completed header part",
  );
});
