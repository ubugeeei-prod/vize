import assert from "node:assert/strict";
import { test } from "node:test";
import { observeHistoricalSplitWatchLog } from "./support/native-beta-dto/stable-watcher-mixed.ts";
import {
  historicalSource,
  stableImageSha256,
  stableWatchActor,
  observeStableSplitWatchLog,
  type WatcherContext,
  type WatcherTimedIo,
} from "./support/native-beta-dto/stable-watcher-boundary.ts";
import {
  parseNativeIo,
  type NativeTimedObservation,
  type nativeOwnerTerminalState,
} from "./support/native-beta-dto/native-io.ts";
type Packet = Record<string, unknown>;
const at = (value: number) => `1793423752.${String(value).padStart(6, "0")}`;
function fixture() {
  // Constructed contracts; authenticated T700f replay is retained separately.
  const project = "/tmp/.tmpmixed1",
    config = project + "/tsconfig.json";
  const context: WatcherContext = {
    source: historicalSource,
    role: "before",
    sdkVersion: "7.0.2",
    sdkSha256: stableImageSha256,
    sdkExecutable: "/sdk/node_modules/@typescript/typescript-linux-x64/lib/tsc",
    projectRoots: [project],
  };
  const packet = (value: Packet, time: number, tid: number, line: number) => ({
    packet: value,
    at: at(time),
    tid,
    line,
  });
  const client = [
    packet(
      {
        jsonrpc: "2.0",
        id: 1,
        method: "initialize",
        params: {
          rootPath: project + "/node_modules/.vize/corsa-overlay",
          rootUri: `file://${project}/node_modules/.vize/corsa-overlay`,
        },
      },
      1,
      38,
      2,
    ),
    packet(
      {
        jsonrpc: "2.0",
        id: 3,
        method: "custom/projectInfo",
        params: { textDocument: { uri: `file://${project}/main.ts` } },
      },
      3,
      38,
      5,
    ),
    packet({ jsonrpc: "2.0", id: 4, method: "shutdown" }, 5, 38, 10),
    packet({ jsonrpc: "2.0", id: "ts2", result: null }, 8, 38, 12),
  ];
  const server = [
    packet(
      {
        jsonrpc: "2.0",
        id: 1,
        result: { serverInfo: { name: "typescript-go", version: "7.0.2" } },
      },
      2,
      52,
      40,
    ),
    packet({ jsonrpc: "2.0", id: 3, result: { configFilePath: config } }, 4, 52, 45),
    packet(
      {
        jsonrpc: "2.0",
        id: "ts2",
        method: "client/registerCapability",
        params: {
          registrations: [
            {
              id: `root files for ${config} watcher 6.0`,
              method: "workspace/didChangeWatchedFiles",
              registerOptions: { watchers: [{ globPattern: project + "/**/*", kind: 7 }] },
            },
          ],
        },
      },
      6,
      52,
      49,
    ),
    packet({ jsonrpc: "2.0", id: 4, result: null }, 7, 52, 50),
    packet(
      {
        jsonrpc: "2.0",
        method: "window/logMessage",
        params: { type: 3, message: `Added new watch: root files for ${config} watcher 6.0` },
      },
      12,
      38,
      14,
    ),
  ];
  const parts = [
    `\t${project}/**/*\n`,
    "\n",
    "1 paths ineligible for watching\n",
    "errors updating watches: [failed to register file watcher: context canceled failed to register file watcher: context canceled]\n",
  ];
  const eof: Array<NativeTimedObservation & { fd: number }> = [
    {
      at: at(9),
      tid: 38,
      line: 13,
      startedLine: 13,
      fd: 0,
      completion: {
        returnedAt: null,
        returnedBy: { at: at(12), line: 14, basis: "next-same-thread-event" },
      },
    },
  ];
  const owner: ReturnType<typeof nativeOwnerTerminalState> = {
    nativeOwner: 10,
    members: [
      { tid: 10, parent: null },
      { tid: 38, parent: 10 },
      { tid: 43, parent: 10 },
      { tid: 52, parent: 10 },
    ],
    threadExits: [10, 38, 43, 52].map((tid) => ({ tid, at: at(20), code: 0 })),
    exitGroups: [{ tid: 43, at: at(16), code: 0 }],
    unknownWrites: [],
    failedWrites: [],
    zeroCapacityReads: [],
    unfinishedOperations: [],
  };
  return { context, client, server, parts, eof, owner };
}
function inputs(f: ReturnType<typeof fixture>) {
  const frame = (value: Packet) => {
    const body = Buffer.from(JSON.stringify(value));
    return Buffer.concat([Buffer.from(`Content-Length: ${body.length}\r\n\r\n`), body]);
  };
  const io: WatcherTimedIo[] = [
    ...f.client.map((row) => ({
      ...row,
      op: "read",
      fd: 0,
      data: frame(row.packet),
      completion: { returnedAt: row.at, returnedBy: null },
    })),
    ...f.server.map((row) => ({
      ...row,
      op: "write",
      fd: 1,
      data: frame(row.packet),
      completion: { returnedAt: row.at, returnedBy: null },
    })),
    ...f.parts.map((part, index) => ({
      at: at([10, 11, 13, 14][index] ?? 15 + index),
      tid: 43,
      line: 29 + index,
      op: "write",
      fd: 2,
      data: Buffer.from(part),
      completion: {
        returnedAt: null,
        returnedBy: {
          at: at(15 + index),
          line: 30 + index,
          basis: "next-same-thread-event" as const,
        },
      },
    })),
  ];
  return {
    client: f.client.map((row) => row.packet),
    server: f.server.map((row) => row.packet),
    io,
  };
}
function observe(f: ReturnType<typeof fixture>, change?: (io: WatcherTimedIo[]) => void) {
  const value = inputs(f);
  change?.(value.io);
  return observeHistoricalSplitWatchLog(
    value.client,
    value.server,
    f.context,
    value.io,
    f.eof,
    f.owner,
  );
}
test("mixed return intervals are explicit observations with no qualification credit", () => {
  const f = fixture(),
    result = observe(f);
  assert.equal(result.schema, "vize.nativeStableWatcher.unprovenEofReturnObservation.v1");
  assert.deepEqual(
    result.stderrWrites.map((row) => ("eofReturnOrder" in row ? row.eofReturnOrder : null)),
    ["UNPROVEN", "UNPROVEN", "PROVEN", "PROVEN"],
  );
  for (const key of [
    "nativeQualification",
    "candidateFunctionalPass",
    "lifecyclePass",
    "registrationPass",
  ] as const)
    assert.equal(result[key], false);
  assert.equal(result.nativeStdinEof.completion!.returnedAt, null);
  assert.equal(
    Buffer.concat(result.stderrWrites.map((row) => Buffer.from(row.hex, "hex"))).length,
    182,
  );
  const value = inputs(f);
  assert.throws(() => stableWatchActor(value.client, value.server, f.context, value.io, f.eof));
  assert.throws(() =>
    observeStableSplitWatchLog(value.client, value.server, f.context, value.io, f.eof),
  );
});
test("fully proven strict witness retains its exact previous result and failed retry policy", () => {
  const f = fixture();
  f.eof[0]!.completion = { returnedAt: at(9), returnedBy: null };
  const failed = parseNativeIo('1793423752.000001 write(2, "\\x61", 1) = -1 EAGAIN')!;
  assert.equal(failed.outcome, "failed");
  f.owner.failedWrites.push({ ...failed, tid: 43, file: "syscalls.43", line: 1 });
  const value = inputs(f);
  assert.deepEqual(
    observe(f),
    observeStableSplitWatchLog(value.client, value.server, f.context, value.io, f.eof),
  );
});
test("wrong actors, bracket claims, incomplete frames or extra payload cannot be observations", () => {
  const cases: Array<(f: ReturnType<typeof fixture>) => void> = [
    (f) => {
      f.context.role = "after";
    },
    (f) => {
      f.context.source = "0".repeat(40);
    },
    (f) => {
      f.context.sdkVersion = "7.1.0-beta";
    },
    (f) => {
      f.context.sdkSha256 = "0".repeat(64);
    },
    (f) => {
      f.context.sdkExecutable = "/other/tsc";
    },
    (f) => {
      f.context.projectRoots = [];
    },
    (f) => {
      f.owner.nativeOwner = 99;
    },
    (f) => {
      f.owner.members.pop();
    },
    (f) => {
      f.owner.threadExits.pop();
    },
    (f) => {
      f.owner.threadExits[0]!.code = 1;
    },
    (f) => {
      f.eof[0]!.tid = 999;
    },
    (f) => {
      f.eof[0]!.fd = 7;
    },
    (f) => {
      f.eof[0]!.completion!.returnedAt = at(16);
    },
    (f) => {
      f.eof[0]!.completion!.returnedBy = null;
    },
    (f) => {
      f.eof[0]!.completion!.returnedBy!.basis = "explicit-resume";
    },
    (f) => {
      f.eof[0]!.completion!.returnedBy!.at = at(17);
    },
    (f) => {
      f.eof[0]!.completion!.returnedBy!.line = 15;
    },
    (f) => {
      f.parts.push("\n");
    },
    (f) => {
      f.parts[0] = "\t/tmp/foreign/**/*\n";
    },
    (f) => {
      f.client.pop();
    },
    (f) => {
      f.server.pop();
    },
    (f) => {
      f.server[4]!.packet.params = { type: 3, message: "wrong header" };
    },
  ];
  for (const change of cases) {
    const f = fixture();
    change(f);
    assert.throws(() => observe(f));
  }
  assert.throws(() =>
    observe(fixture(), (io) => {
      io.find((row) => row.fd === 2)!.at = at(8);
    }),
  );
  assert.throws(() =>
    observe(fixture(), (io) => {
      io.find((row) => row.fd === 2)!.tid = 999;
    }),
  );
  assert.throws(() =>
    observe(fixture(), (io) => {
      io.find((row) => row.fd === 1)!.data = Buffer.from("Content-Length: 99\r\n\r\n{}");
    }),
  );
});
test("actual unknown and failed write-attempt metadata refuses only the mixed observation", () => {
  for (const result of ["?", "-1 EAGAIN"]) {
    const f = fixture(),
      write = parseNativeIo(`1793423752.000010 write(2, "\\x61", 1) = ${result}`)!;
    if (write.outcome === "unknown")
      f.owner.unknownWrites.push({ ...write, tid: 43, file: "syscalls.43", line: 29 });
    else {
      assert.equal(write.outcome, "failed");
      f.owner.failedWrites.push({ ...write, tid: 43, file: "syscalls.43", line: 29 });
    }
    assert.throws(() => observe(f));
  }
});
