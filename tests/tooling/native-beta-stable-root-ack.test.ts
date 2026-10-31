import assert from "node:assert/strict";
import { test } from "node:test";
import { observeHistoricalSplitWatchLog } from "./support/native-beta-dto/stable-watcher-mixed.ts";
import {
  historicalSource,
  stableImageSha256,
  stableWatchContext,
  stableWatchActor,
  type WatcherContext,
  type WatcherTimedIo,
} from "./support/native-beta-dto/stable-watcher-boundary.ts";
import {
  parseNativeIo,
  type NativeTimedObservation,
  type nativeOwnerTerminalState,
} from "./support/native-beta-dto/native-io.ts";
type Packet = Record<string, unknown>;
const at = (value: number) => `1793475156.${String(value).padStart(6, "0")}`;
const frame = (packet: Packet) => {
  const body = Buffer.from(JSON.stringify(packet));
  return Buffer.concat([Buffer.from(`Content-Length: ${body.length}\r\n\r\n`), body]);
};
function fixture() {
  // Constructed contracts only; the separately retained failed a0e trace is real.
  const project = "/tmp/.tmpack123",
    config = project + "/tsconfig.json";
  const context: WatcherContext = {
    source: historicalSource,
    role: "before",
    sdkVersion: "7.0.2",
    sdkSha256: stableImageSha256,
    sdkExecutable: "/sdk/node_modules/@typescript/typescript-linux-x64/lib/tsc",
    projectRoots: [project],
  };
  const client: Packet[] = [
    {
      jsonrpc: "2.0",
      id: 1,
      method: "initialize",
      params: {
        rootPath: project + "/node_modules/.vize/corsa-overlay",
        rootUri: `file://${project}/node_modules/.vize/corsa-overlay`,
      },
    },
    {
      jsonrpc: "2.0",
      id: 3,
      method: "custom/projectInfo",
      params: { textDocument: { uri: `file://${project}/main.ts` } },
    },
    { jsonrpc: "2.0", id: 4, method: "shutdown" },
    { jsonrpc: "2.0", id: "ts2", result: null },
  ];
  const server: Packet[] = [
    { jsonrpc: "2.0", id: 1, result: { serverInfo: { name: "typescript-go", version: "7.0.2" } } },
    { jsonrpc: "2.0", id: 3, result: { configFilePath: config } },
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
    { jsonrpc: "2.0", id: 4, result: null },
    {
      jsonrpc: "2.0",
      method: "window/logMessage",
      params: { type: 3, message: `Added new watch: root files for ${config} watcher 6.0` },
    },
  ];
  const io: WatcherTimedIo[] = [
    ...client.slice(0, 3).map((packet, index) => ({
      at: at(1 + index * 2),
      tid: 38,
      line: 1 + index * 2,
      op: "read",
      fd: 0,
      data: frame(packet),
      completion: { returnedAt: at(1 + index * 2), returnedBy: null },
    })),
    ...server.map((packet, index) => ({
      at: at([2, 4, 7, 8, 16][index]!),
      tid: 52,
      line: 10 + index,
      op: "write",
      fd: 1,
      data: frame(packet),
      completion: { returnedAt: at([2, 4, 7, 8, 16][index]!), returnedBy: null },
    })),
    {
      at: at(6),
      tid: 43,
      line: 10,
      op: "read",
      fd: 0,
      data: Buffer.from("Content-Length: 42\r\n\r\n"),
      completion: {
        returnedAt: null,
        returnedBy: { at: at(20), line: 11, basis: "next-same-thread-event" },
      },
    },
    {
      at: at(10),
      tid: 38,
      line: 12,
      op: "read",
      fd: 0,
      data: Buffer.from(JSON.stringify(client[3])),
      completion: {
        returnedAt: null,
        returnedBy: { at: at(11), line: 13, basis: "next-same-thread-event" },
      },
    },
    ...[
      `\t${project}/**/*\n`,
      "\n",
      "1 paths ineligible for watching\n",
      "errors updating watches: [failed to register file watcher: context canceled failed to register file watcher: context canceled]\n",
    ].map((text, index) => ({
      at: at(12 + index),
      tid: 52,
      line: 25 + index,
      op: "write",
      fd: 2,
      data: Buffer.from(text),
      completion: { returnedAt: at(12 + index), returnedBy: null },
    })),
  ];
  const eof: Array<NativeTimedObservation & { fd: number }> = [
    {
      at: at(11),
      tid: 38,
      line: 13,
      fd: 0,
      completion: {
        returnedAt: null,
        returnedBy: { at: at(19), line: 14, basis: "next-same-thread-event" },
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
    threadExits: [
      { tid: 10, at: at(30), line: 90, code: 0 },
      { tid: 38, at: at(19), line: 14, code: 0 },
      { tid: 43, at: at(20), line: 11, code: 0 },
      { tid: 52, at: at(25), line: 50, code: 0 },
    ],
    exitGroups: [],
    unknownWrites: [],
    failedWrites: [],
    zeroCapacityReads: [],
    unfinishedOperations: [],
  };
  return { client, server, context, io, eof, owner };
}
type Fixture = ReturnType<typeof fixture>;
const run = (f: Fixture) =>
  observeHistoricalSplitWatchLog(f.client, f.server, f.context, f.io, f.eof, f.owner);
const header = (f: Fixture) => f.io.find((row) => row.tid === 43)!;
const body = (f: Fixture) => f.io.find((row) => row.tid === 38 && row.line === 12)!;
test("split root ACK retains its unproven header order and independently proven body order", () => {
  const f = fixture(),
    result = run(f);
  assert.ok("rootAckFragmentOrder" in result);
  assert.ok(result.rootAckFragmentOrder);
  assert.equal(result.rootAckFragmentOrder.headerReturnBeforeBodyEntry, "UNPROVEN");
  assert.equal(result.rootAckFragmentOrder.headerReturnBeforeEofEntry, "UNPROVEN");
  assert.equal(result.rootAckFragmentOrder.bodyReturnBeforeEofEntry, "PROVEN");
  assert.equal(result.rootAckFragmentOrder.header.completion!.returnedAt, null);
  assert.equal(result.nativeStdinEof.completion!.returnedAt, null);
  assert.deepEqual(
    result.stderrWrites.map((row) => ("eofReturnOrder" in row ? row.eofReturnOrder : null)),
    ["UNPROVEN", "UNPROVEN", "UNPROVEN", "UNPROVEN"],
  );
  for (const key of [
    "nativeQualification",
    "candidateFunctionalPass",
    "lifecyclePass",
    "registrationPass",
  ] as const)
    assert.equal(result[key], false);
  assert.equal(result.observedRootRegistrationAcknowledged, true);
  for (const strict of [stableWatchContext, stableWatchActor])
    assert.throws(() => strict(f.client, f.server, f.context, f.io, f.eof));
});
test("source, role, image, root, frames, actual fragment identities and terminal bounds stay closed", () => {
  const changes: Array<(f: Fixture) => void> = [
    (f) => {
      f.context.role = "after";
    },
    (f) => {
      f.context.source = "other";
    },
    (f) => {
      f.context.sdkVersion = "7.1.0-beta";
    },
    (f) => {
      f.context.sdkSha256 = "a".repeat(64);
    },
    (f) => {
      f.context.projectRoots = ["/tmp/foreign"];
    },
    (f) => {
      header(f).data = Buffer.from("Content-Length: 41\r\n\r\n");
    },
    (f) => {
      header(f).completion!.returnedAt = at(12);
    },
    (f) => {
      header(f).completion!.returnedBy!.basis = "explicit-resume";
    },
    (f) => {
      header(f).completion!.returnedBy!.at = at(5);
    },
    (f) => {
      header(f).line = 13;
    },
    (f) => {
      body(f).line = 11;
    },
    (f) => {
      body(f).tid = 52;
    },
    (f) => {
      body(f).startedLine = 11;
    },
    (f) => {
      body(f).data = Buffer.from(JSON.stringify({ jsonrpc: "2.0", id: "ts2", result: true }));
    },
    (f) => {
      f.eof[0]!.completion!.returnedAt = at(12);
    },
    (f) => {
      f.eof[0]!.completion!.returnedBy!.line = 15;
    },
    (f) => {
      f.owner.threadExits[1]!.code = 1;
    },
    (f) => {
      f.owner.threadExits[1]!.tid = 52;
    },
    (f) => {
      delete f.owner.threadExits[1]!.line;
    },
    (f) => {
      f.owner.threadExits[1]!.at = at(18);
    },
    (f) => {
      f.owner.threadExits[2]!.code = 1;
    },
    (f) => {
      f.owner.threadExits[3]!.at = at(11);
    },
    (f) => {
      f.owner.threadExits[3]!.line = 24;
    },
    (f) => {
      f.owner.threadExits.push({ ...f.owner.threadExits[2]! });
    },
    (f) => {
      f.owner.members[1]!.parent = 38;
    },
    (f) => {
      f.owner.nativeOwner = 38;
    },
    (f) => {
      f.io.push({ ...header(f) });
    },
    (f) => {
      f.io.find((row) => row.fd === 2)!.at = at(9);
    },
    (f) => {
      f.io.find((row) => row.fd === 2)!.at = at(21);
    },
    (f) => {
      f.io.find((row) => row.fd === 2)!.data = Buffer.from("different");
    },
    (f) => {
      f.server[2] = { ...f.server[2], params: { registrations: [] } };
    },
    (f) => {
      f.client.pop();
    },
  ];
  for (const change of changes) {
    const f = fixture();
    change(f);
    assert.throws(() => run(f));
  }
});
test("unknown and known-failed attempts do not become this complete historical fragment shape", () => {
  for (const result of ["?", "-1 EAGAIN (Resource temporarily unavailable)"]) {
    const f = fixture(),
      attempt = parseNativeIo(`${at(13)} write(2, "\\x78", 1) = ${result}`);
    assert.ok(attempt && attempt.op === "write" && attempt.outcome !== "completed");
    const metadata = { tid: 52, file: "syscalls.52", line: 27, startedLine: 27 };
    if (attempt.outcome === "unknown") f.owner.unknownWrites.push({ ...attempt, ...metadata });
    else f.owner.failedWrites.push({ ...attempt, ...metadata });
    assert.throws(() => run(f));
  }
});
test("complete stderr bytes and per-attempt evidence survive an OS write partition change", () => {
  const f = fixture(),
    writes = f.io.filter((row) => row.fd === 2);
  f.io = f.io.filter((row) => row.fd !== 2);
  f.io.push({ ...writes[0]!, data: Buffer.concat(writes.map((row) => Buffer.from(row.data))) });
  const result = run(f);
  assert.equal(result.stderrWrites.length, 1);
  assert.equal(
    "eofReturnOrder" in result.stderrWrites[0]! && result.stderrWrites[0].eofReturnOrder,
    "UNPROVEN",
  );
  assert.equal(result.nativeQualification, false);
});
