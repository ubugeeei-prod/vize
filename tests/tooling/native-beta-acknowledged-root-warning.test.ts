import assert from "node:assert/strict";
import { test } from "node:test";
import { fixture, observe, at, type Packet } from "./support/native-beta-root-warning-fixture.ts";
const object = (value: unknown) => value as Packet;

import { inflightArgs, inflightFixture } from "./support/native-beta-inflight-warning-fixture.ts";
import {
  matchesHistoricalRootWarning,
  observeAcknowledgedRootWarning,
  observeRootWatchWarning as observeCurrentRootWarning,
} from "./support/native-beta-dto/historical-acknowledged-root-warning.ts";
import { buildRoleDisposition } from "./support/native-beta-dto/role-disposition.ts";

function acknowledgedArgs() {
  const f = inflightFixture();
  f.owner.unknownWrites = [];
  f.stderr.push({ at: at(17), data: Buffer.from("1 paths ineligible for watching\n") });
  const args = inflightArgs(f);
  for (const [i, row] of args[3].filter((row) => row.fd === 2).entries()) {
    row.tid = 43;
    row.line = 101 + i;
    row.startedLine = row.line;
    row.completion = {
      returnedAt: null,
      returnedBy: {
        at: i === 3 ? at(24) : at(13 + i),
        line: i === 3 ? 600 : row.line + 1,
        basis: "next-same-thread-event",
      },
    };
  }
  args[4][0]!.completion = {
    returnedAt: null,
    returnedBy: { at: at(12), line: 101, basis: "next-same-thread-event" },
  };
  return args;
}
type AcknowledgedArgs = ReturnType<typeof acknowledgedArgs>;

test("fully acknowledged root header+warning retains exact EOF proof without cancellation or credit", () => {
  const args = acknowledgedArgs(),
    before = JSON.stringify(args);
  const result = observeCurrentRootWarning(...args);
  assert.equal(result.kind, "acknowledged-root-header-warning");
  assert.deepEqual(result, observeAcknowledgedRootWarning(...args));
  assert.equal(JSON.stringify(args), before);
  assert.equal(result.completedStderr.bytes, 129);
  assert.equal(result.stderrOutputComplete, true);
  assert.equal(result.rawIOComplete, true);
  assert.equal(result.observedRootRegistrationAcknowledged, true);
  for (const key of [
    "nativeQualification",
    "candidateFunctionalPass",
    "lifecyclePass",
    "registrationPass",
  ] as const)
    assert.equal(result[key], false);
  assert.deepEqual(
    result.stderrWrites.map((row) => row.eofReturnOrder),
    ["PROVEN", "PROVEN", "PROVEN", "PROVEN"],
  );
  assert.equal(result.stderrWrites.at(-1)!.exitGroupReturnOrder, "UNPROVEN");
  assert.deepEqual(result.clientPackets, args[0]);
  assert.deepEqual(result.serverPackets, args[1]);
  assert.deepEqual(result.api, args[6]);
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

test("acknowledged header+warning rejects source, complete owner, paired channel and temporal mutants", () => {
  const mutations: Array<[string, (args: AcknowledgedArgs) => void]> = [
    [
      "after",
      (a) => {
        a[2].role = "after";
      },
    ],
    [
      "foreign source",
      (a) => {
        a[2].source = "0".repeat(40);
      },
    ],
    [
      "foreign image",
      (a) => {
        a[2].sdkSha256 = "0".repeat(64);
      },
    ],
    [
      "wrong SDK",
      (a) => {
        a[2].sdkVersion = "7.1.0-beta";
      },
    ],
    [
      "foreign executable",
      (a) => {
        a[2].sdkExecutable = "/foreign/tsc";
      },
    ],
    [
      "foreign root",
      (a) => {
        a[2].projectRoots = ["/tmp/foreign"];
      },
    ],
    [
      "wrong root ACK",
      (a) => {
        object(a[0].find((v) => object(v).id === "ts2")).result = "wrong";
      },
    ],
    [
      "missing root ACK",
      (a) => {
        a[0] = a[0].filter((v) => object(v).id !== "ts2");
      },
    ],
    [
      "duplicate root ACK",
      (a) => {
        a[0] = [...a[0], a[0].find((v) => object(v).id === "ts2")!];
      },
    ],
    [
      "unknown response field",
      (a) => {
        object(a[0].find((v) => object(v).id === "ts2")).extra = true;
      },
    ],
    [
      "missing raw channel",
      (a) => {
        a[3] = a[3].filter((v) => !(v.op === "read" && v.fd === 0));
      },
    ],
    [
      "API foreign path",
      (a) => {
        object(object(a[6].client[1]).params).file = "/tmp/foreign/tsconfig.json";
      },
    ],
    [
      "API extra reply",
      (a) => {
        a[6].server = [...a[6].server, { jsonrpc: "2.0", id: 999, result: null }];
      },
    ],
    [
      "API prefix",
      (a) => {
        a[6].precedingNonProtocolBytes = 0;
      },
    ],
    [
      "unknown write",
      (a) => {
        a[5].unknownWrites = fixture("B").owner.unknownWrites;
      },
    ],
    [
      "failed write",
      (a) => {
        a[5].failedWrites.push({
          ...fixture("B").owner.unknownWrites[0]!,
          outcome: "failed",
          returnedBytes: -1,
        });
      },
    ],
    [
      "unfinished",
      (a) => {
        a[5].unfinishedOperations = fixture("B").owner.unfinishedOperations;
      },
    ],
    [
      "foreign owner",
      (a) => {
        a[5].nativeOwner = 999;
      },
    ],
    [
      "owner cycle",
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
      "nonzero group",
      (a) => {
        a[5].exitGroups[0]!.code = 1;
      },
    ],
    [
      "missing EOF",
      (a) => {
        a[4] = [];
      },
    ],
    [
      "wrong EOF fd",
      (a) => {
        a[4][0]!.fd = 3;
      },
    ],
    [
      "foreign EOF",
      (a) => {
        a[4][0]!.tid = 999;
      },
    ],
    [
      "ACK after EOF",
      (a) => {
        a[4][0]!.line = 0;
        a[4][0]!.at = at(9);
      },
    ],
    [
      "unproven EOF",
      (a) => {
        a[4][0]!.tid = 44;
        a[4][0]!.completion!.returnedBy = {
          at: at(23),
          line: 600,
          basis: "next-same-thread-event",
        };
      },
    ],
    [
      "forged EOF bound",
      (a) => {
        a[4][0]!.completion!.returnedBy!.at = at(22);
      },
    ],
    [
      "missing completion",
      (a) => {
        delete a[3].find((v) => v.fd === 2)!.completion;
      },
    ],
    [
      "unowned stderr",
      (a) => {
        a[3].find((v) => v.fd === 2)!.tid = 999;
      },
    ],
    [
      "wrong warning",
      (a) => {
        a[3].filter((v) => v.fd === 2).at(-1)!.data = Buffer.from("other\n");
      },
    ],
    [
      "extra cancellation",
      (a) => {
        a[3].filter((v) => v.fd === 2).at(-1)!.data = Buffer.from(
          "1 paths ineligible for watching\nerrors updating watches: invented\n",
        );
      },
    ],
  ];
  for (const [name, mutate] of mutations) {
    const args = acknowledgedArgs();
    mutate(args);
    assert.throws(() => observeAcknowledgedRootWarning(...args), assert.AssertionError, name);
  }
});

test("dispatch is exact and old historical observers and cancellation-positive code stay separate", () => {
  const bytes = Buffer.from(
    "Added new watch: root files for /tmp/.tmpUnit01/tsconfig.json watcher 6.0\n\t/tmp/.tmpUnit01/**/*\n\n1 paths ineligible for watching\n",
  );
  assert.equal(matchesHistoricalRootWarning(bytes), true);
  assert.equal(
    matchesHistoricalRootWarning(Buffer.from("1 paths ineligible for watching\n")),
    true,
  );
  for (const bad of [
    bytes.subarray(1),
    Buffer.concat([bytes, Buffer.from("other")]),
    Buffer.from(bytes.toString().replace("/tmp/.tmpUnit01/**/*", "/tmp/foreign/**/*")),
  ])
    assert.equal(matchesHistoricalRootWarning(bad), false);
  for (const kind of ["A", "B"] as const) {
    const f = fixture(kind);
    let saved: AcknowledgedArgs[3] | undefined;
    const old = observe(f, (io) => {
      saved = io;
    });
    assert.ok(saved);
    assert.deepEqual(
      observeCurrentRootWarning(
        f.client.map((r) => r.packet),
        f.server.map((r) => r.packet),
        f.context,
        saved,
        f.eof,
        f.owner,
        f.api,
      ),
      old,
    );
  }
});
