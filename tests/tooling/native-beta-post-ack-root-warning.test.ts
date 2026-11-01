import assert from "node:assert/strict";
import { test } from "node:test";
import { inflightFixture, inflightArgs } from "./support/native-beta-inflight-warning-fixture.ts";
import { fixture, observe, type Packet, at } from "./support/native-beta-root-warning-fixture.ts";
import {
  historicalInflightTranscript,
  observeRootWatchWarning as strictObserver,
} from "./support/native-beta-dto/historical-inflight-root-warning.ts";
import {
  historicalPostAckTranscript,
  observeHistoricalPostAckRootWarning,
  observeRootWatchWarning,
} from "./support/native-beta-dto/historical-post-ack-root-warning.ts";
import { requireNoUnknownNativeWrites } from "./support/native-beta-dto/native-io.ts";
import { buildRoleDisposition } from "./support/native-beta-dto/role-disposition.ts";

function postAckFixture() {
  const f = inflightFixture();
  f.server.find((row) => row.packet.id === "ts2")!.at = at(10);
  f.server.sort((a, b) => a.at.localeCompare(b.at));
  return f;
}
const fresh = () => inflightArgs(postAckFixture());
type Args = ReturnType<typeof fresh>;
const object = (value: unknown) => value as Packet;

test("post-ACK paired channels yield only a historical negative, with whole source rows retained", () => {
  const args = fresh(),
    before = JSON.stringify(args);
  assert.throws(() => historicalInflightTranscript(args[0], args[1], args[2], args[6]));
  assert.throws(() => strictObserver(...args));
  const proof = historicalPostAckTranscript(args[0], args[1], args[2], args[6]);
  assert.ok(args[1].indexOf(proof.configuration) < args[1].indexOf(proof.shutdownAck));
  assert.ok(args[1].indexOf(proof.shutdownAck) < args[1].indexOf(proof.watcher));
  const result = observeRootWatchWarning(...args);
  assert.equal(result.kind, "post-ack-root-warning-negative");
  assert.deepEqual(result, observeHistoricalPostAckRootWarning(...args));
  assert.equal(JSON.stringify(args), before);
  assert.deepEqual(result.clientPackets, args[0]);
  assert.deepEqual(result.serverPackets, args[1]);
  assert.deepEqual(result.api, args[6]);
  if (result.kind !== "post-ack-root-warning-negative") assert.fail("new negative only");
  assert.deepEqual(result.sourceIoRows, args[3]);
  assert.deepEqual(result.nativeOwnerSourceRows, args[5]);
  assert.deepEqual(result.sourceEofRowsUninterpreted, args[4]);
  assert.equal(result.completedStderr.bytes, 97);
  assert.deepEqual(result.unknownAttempts[0]!.sourceRow, args[5].unknownWrites[0]);
  for (const name of ["returnedBytes", "deliveredBytes", "actualUnknownReturnTime"] as const)
    assert.equal(result.unknownAttempts[0]![name], null);
  assert.equal(result.unknownAttempts[0]!.eofReturnOrder, "UNPROVEN");
  for (const name of [
    "nativeQualification",
    "candidateFunctionalPass",
    "lifecyclePass",
    "registrationPass",
    "rawIOComplete",
    "stderrOutputComplete",
  ] as const)
    assert.equal(result[name], false);
  assert.throws(() => requireNoUnknownNativeWrites(args[5].unknownWrites));
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
  for (const name of [
    "nativeQualification",
    "dtoFunctionalPass",
    "lifecyclePass",
    "registrationPass",
    "rawIOComplete",
    "stderrOutputComplete",
  ] as const)
    assert.equal(disposition[name], false);
});

test("the old strict exports and old full observation objects remain strict and identical", () => {
  const args = inflightArgs(inflightFixture());
  const proof = historicalInflightTranscript(args[0], args[1], args[2], args[6]);
  assert.deepEqual(Object.keys(proof), [
    "clientValue",
    "serverValue",
    "context",
    "api",
    "config",
    "root",
    "initialize",
    "initialized",
    "projectInfo",
    "projectReply",
    "watcher",
    "watcherAck",
    "shutdown",
    "shutdownAck",
  ]);
  assert.deepEqual(observeRootWatchWarning(...args), strictObserver(...args));
  for (const kind of ["A", "B"] as const) {
    const f = fixture(kind);
    let saved: Args[3] | undefined;
    const expected = observe(f, (io) => {
      saved = io;
    });
    assert.ok(saved);
    const oldArgs: Args = [
      f.client.map((row) => row.packet),
      f.server.map((row) => row.packet),
      f.context,
      saved,
      f.eof,
      f.owner,
      f.api,
    ];
    assert.deepEqual(observeRootWatchWarning(...oldArgs), expected);
  }
  const post = fresh();
  assert.throws(() => historicalInflightTranscript(post[0], post[1], post[2], post[6]));
  assert.throws(() => strictObserver(...post));
});

test("transcript-only checking is distinct from a full source-owned actor observation", () => {
  const args = fresh();
  historicalPostAckTranscript(args[0], args[1], args[2], args[6]);
  assert.throws(() =>
    observeHistoricalPostAckRootWarning(args[0], args[1], args[2], [], args[4], args[5], args[6]),
  );
  // This class deliberately does not infer any EOF return from the source rows.
  args[4] = [];
  const result = observeHistoricalPostAckRootWarning(...args);
  assert.deepEqual(result.sourceEofRowsUninterpreted, []);
  assert.equal(result.lifecyclePass, false);
  assert.equal(result.rawIOComplete, false);
});

test("exact source/role/image/root/pairing/API and post-ACK channel order are required", () => {
  const mutations: Array<[string, (a: Args) => void]> = [
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
      "executable",
      (a) => {
        a[2].sdkExecutable = "/foreign/tsc";
      },
    ],
    [
      "root",
      (a) => {
        a[2].projectRoots = ["/tmp/foreign"];
      },
    ],
    [
      "prefix",
      (a) => {
        a[6].precedingNonProtocolBytes = 6;
      },
    ],
    [
      "API config",
      (a) => {
        object(object(a[6].client[1]).params).file = "/tmp/foreign/tsconfig.json";
      },
    ],
    [
      "extra API reply",
      (a) => {
        a[6].server = [...a[6].server, { jsonrpc: "2.0", id: 999, result: null }];
      },
    ],
    [
      "missing API",
      (a) => {
        a[6].client = [];
      },
    ],
    [
      "wrong ACK",
      (a) => {
        object(a[0].find((row) => object(row).id === "ts2")).result = "wrong";
      },
    ],
    [
      "missing ACK",
      (a) => {
        a[0] = a[0].filter((row) => object(row).id !== "ts2");
      },
    ],
    [
      "duplicate ACK",
      (a) => {
        a[0] = [...a[0], a[0].find((row) => object(row).id === "ts2")!];
      },
    ],
    [
      "pre-ACK root",
      (a) => {
        const rows = [...a[1]],
          watch = rows.splice(
            rows.findIndex((row) => object(row).id === "ts2"),
            1,
          )[0]!;
        rows.splice(
          rows.findIndex((row) => object(row).id === 4),
          0,
          watch,
        );
        a[1] = rows;
      },
    ],
    [
      "configuration after ACK",
      (a) => {
        const rows = [...a[1]],
          config = rows.splice(
            rows.findIndex((row) => object(row).id === "ts1"),
            1,
          )[0]!;
        rows.push(config);
        a[1] = rows;
      },
    ],
    [
      "extra watcher",
      (a) => {
        a[1] = [...a[1], { ...object(a[1].find((row) => object(row).id === "ts2")), id: "ts3" }];
      },
    ],
  ];
  for (const [name, mutate] of mutations) {
    const args = fresh();
    mutate(args);
    assert.throws(
      () => historicalPostAckTranscript(args[0], args[1], args[2], args[6]),
      assert.AssertionError,
      name,
    );
  }
});

test("actor graph, one exact unknown32, completed97 same-writer header and no failed/unfinished IO are required", () => {
  const mutations: Array<[string, (a: Args) => void]> = [
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
      "duplicate graph",
      (a) => {
        a[5].members.push(a[5].members[0]!);
      },
    ],
    [
      "foreign writer",
      (a) => {
        a[5].unknownWrites[0]!.tid = 999;
      },
    ],
    [
      "extra unknown",
      (a) => {
        a[5].unknownWrites.push(a[5].unknownWrites[0]!);
      },
    ],
    [
      "missing unknown",
      (a) => {
        a[5].unknownWrites = [];
      },
    ],
    [
      "wrong fd",
      (a) => {
        a[5].unknownWrites[0]!.fd = 1;
      },
    ],
    [
      "wrong file",
      (a) => {
        a[5].unknownWrites[0]!.file = "syscalls.999";
      },
    ],
    [
      "unknown marker",
      (a) => {
        a[5].unknownWrites[0]!.rawResult = "? <unavailable>";
      },
    ],
    [
      "incomplete arguments",
      (a) => {
        Object.assign(a[5].unknownWrites[0]!, { argumentsComplete: false });
      },
    ],
    [
      "iov syscall",
      (a) => {
        a[5].unknownWrites[0]!.syscall = "writev";
      },
    ],
    [
      "body",
      (a) => {
        a[5].unknownWrites[0]!.data = Buffer.from("other");
      },
    ],
    [
      "count",
      (a) => {
        a[5].unknownWrites[0]!.requestedBytes = 31;
      },
    ],
    [
      "invented return",
      (a) => {
        Object.assign(a[5].unknownWrites[0]!, { returnedBytes: 0 });
      },
    ],
    [
      "invented completion",
      (a) => {
        a[5].unknownWrites[0]!.completion = { returnedAt: at(18), returnedBy: null };
      },
    ],
    [
      "failed",
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
      "header body",
      (a) => {
        a[3].find((row) => row.fd === 2)!.data = Buffer.from("wrong");
      },
    ],
    [
      "header writer",
      (a) => {
        a[3].find((row) => row.fd === 2)!.tid = 42;
      },
    ],
    [
      "header no completion",
      (a) => {
        delete a[3].find((row) => row.fd === 2)!.completion;
      },
    ],
    [
      "extra IO header",
      (a) => {
        (a[3] as unknown[]).push({ ...a[3].find((row) => row.fd === 2)! });
      },
    ],
    [
      "missing whole LSP",
      (a) => {
        (a[3] as unknown[]).splice(0, 1);
      },
    ],
  ];
  for (const [name, mutate] of mutations) {
    const args = fresh();
    mutate(args);
    assert.throws(() => observeHistoricalPostAckRootWarning(...args), assert.AssertionError, name);
  }
});
