import assert from "node:assert/strict";
import { test } from "node:test";
import { observeTerminalStableStderr } from "./support/native-beta-dto/terminal-stderr.ts";
import {
  historicalSource,
  stableImageSha256,
  type WatcherContext,
  type WatcherTimedIo,
} from "./support/native-beta-dto/stable-watcher-boundary.ts";

type Packet = Record<string, unknown>;
const at = (n: number) => `1793411181.${String(n).padStart(6, "0")}`;
const cancellation =
  "errors updating watches: [failed to register file watcher: context canceled failed to register file watcher: context canceled]\n";
// Constructed contract controls; authenticated actual915 replay is separate.
function fixture() {
  const project = "/tmp/terminal-watch-unit",
    config = project + "/tsconfig.json";
  const context: WatcherContext = {
    source: historicalSource,
    role: "before",
    sdkVersion: "7.0.2",
    sdkSha256: stableImageSha256,
    sdkExecutable: "/sdk/node_modules/@typescript/typescript-linux-x64/lib/tsc",
    projectRoots: [project],
  };
  const row = (packet: Packet, time: number) => ({ packet, at: at(time) });
  const client = [
    row(
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
    ),
    row(
      {
        jsonrpc: "2.0",
        id: 3,
        method: "custom/projectInfo",
        params: { textDocument: { uri: `file://${project}/main.ts` } },
      },
      3,
    ),
    row({ jsonrpc: "2.0", id: 4, method: "shutdown" }, 5),
    row({ jsonrpc: "2.0", id: "ts2", result: null }, 8),
  ];
  const server = [
    row(
      {
        jsonrpc: "2.0",
        id: 1,
        result: { serverInfo: { name: "typescript-go", version: "7.0.2" } },
      },
      2,
    ),
    row({ jsonrpc: "2.0", id: 3, result: { configFilePath: config } }, 4),
    row(
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
    ),
    row({ jsonrpc: "2.0", id: 4, result: null }, 7),
    row(
      {
        jsonrpc: "2.0",
        method: "window/logMessage",
        params: { type: 3, message: `Added new watch: root files for ${config} watcher 6.0` },
      },
      10,
    ),
  ];
  const stderr = [
    { at: at(10), data: Buffer.from("\n") },
    { at: at(11), data: Buffer.from("1 paths ineligible for watching\n") },
  ];
  const terminal: Parameters<typeof observeTerminalStableStderr>[5] = {
    nativeOwner: 41,
    members: [
      { tid: 41, parent: null },
      { tid: 42, parent: 41 },
      { tid: 43, parent: 42 },
    ],
    threadExits: [
      { tid: 41, at: at(17), code: 0 },
      { tid: 42, at: at(16), code: 0 },
      { tid: 43, at: at(15), code: 0 },
    ],
    exitGroups: [{ tid: 42, at: at(12), code: 0 }],
    failedWrites: [],
    zeroCapacityReads: [],
    unknownWrites: [
      {
        at: at(13),
        op: "write",
        syscall: "write",
        iovCount: null,
        iovecs: null,
        argumentsComplete: true,
        rawArguments: "constructed unit contract",
        fd: 2,
        tid: 43,
        file: "syscalls.43",
        line: 27,
        data: Buffer.from(cancellation),
        requestedBytes: Buffer.byteLength(cancellation),
        returnedBytes: null,
        outcome: "unknown",
        rawResult: "?",
      },
    ],
    unfinishedOperations: [
      { tid: 41, file: "syscalls.41", line: 28, at: at(14), raw: `${at(14)} ???( ` },
    ],
  };
  return {
    context,
    client,
    server,
    stderr,
    terminal,
    eof: [
      { tid: 43, fd: 0, at: at(9), line: 0, completion: { returnedAt: at(9), returnedBy: null } },
    ],
  };
}
function observe(f: ReturnType<typeof fixture>, mutateIo?: (io: WatcherTimedIo[]) => void) {
  const frame = (packet: Packet) => {
    const body = Buffer.from(JSON.stringify(packet));
    return Buffer.concat([Buffer.from(`Content-Length: ${body.length}\r\n\r\n`), body]);
  };
  const io: WatcherTimedIo[] = [
    ...f.client.map((row, line) => ({
      ...row,
      op: "read",
      fd: 0,
      tid: 43,
      line,
      data: frame(row.packet),
    })),
    ...f.server.map((row, line) => ({
      ...row,
      op: "write",
      fd: 1,
      tid: 42,
      line,
      data: frame(row.packet),
    })),
    ...f.stderr.map((row, line) => ({ ...row, op: "write", fd: 2, tid: 43, line })),
  ];
  for (const row of io) row.completion = { returnedAt: row.at, returnedBy: null };
  mutateIo?.(io);
  return observeTerminalStableStderr(
    f.client.map((row) => row.packet),
    f.server.map((row) => row.packet),
    f.context,
    io,
    f.eof,
    f.terminal,
  );
}
test("unknown terminal tail remains attempted, never completed or qualifying", () => {
  const f = fixture(),
    result = observe(f);
  assert.equal(result.completedStderr.bytes, 33);
  assert.equal(result.attemptedStderr.requestedBytes, 127);
  assert.equal(result.attemptedStderr.returnedBytes, null);
  assert.equal(result.attemptedStderr.deliveredBytes, null);
  assert.equal(result.attemptedStderr.attemptedHex, Buffer.from(cancellation).toString("hex"));
  for (const key of [
    "stderrOutputComplete",
    "rawIOComplete",
    "nativeQualification",
    "candidateFunctionalPass",
    "lifecyclePass",
  ] as const)
    assert.equal(result[key], false);
  assert.equal(result.registrationPass, true);
  assert.deepEqual(result.unfinishedOperations, f.terminal.unfinishedOperations);
});
test("completed byte partition does not define the semantic observation", () => {
  const f = fixture();
  f.stderr = [{ at: at(10), data: Buffer.from("\n1 paths ineligible for watching\n") }];
  assert.equal(observe(f).completedStderr.bytes, 33);
});
test("complete vector attempt is equivalent while mismatched vector metadata refuses", () => {
  const f = fixture(),
    attempt = f.terminal.unknownWrites[0]!;
  attempt.syscall = "writev";
  attempt.iovCount = 2;
  attempt.iovecs = [
    { data: attempt.data.subarray(0, 64), requestedBytes: 64 },
    { data: attempt.data.subarray(64), requestedBytes: attempt.data.length - 64 },
  ];
  assert.equal(observe(f).attemptedStderr.deliveredBytes, null);
  attempt.iovCount = 99;
  assert.throws(() => observe(f));
  attempt.iovCount = 2;
  attempt.iovecs[1]!.requestedBytes++;
  assert.throws(() => observe(f));
});
test("wrong actor, outcome, ownership, payload, lifetime or extra write cannot qualify", () => {
  type F = ReturnType<typeof fixture>;
  const mutations: Array<[string, (f: F) => void]> = [
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
      "foreign project",
      (f) => {
        f.context.projectRoots = ["/tmp/foreign"];
      },
    ],
    [
      "owner",
      (f) => {
        f.terminal.nativeOwner = 999;
      },
    ],
    [
      "attempt owner",
      (f) => {
        f.terminal.unknownWrites[0]!.tid = 999;
      },
    ],
    [
      "attempt fd",
      (f) => {
        f.terminal.unknownWrites[0]!.fd = 1;
      },
    ],
    [
      "attempt body",
      (f) => {
        f.terminal.unknownWrites[0]!.data = Buffer.from("other\n");
      },
    ],
    [
      "attempt count",
      (f) => {
        f.terminal.unknownWrites[0]!.requestedBytes = 126;
      },
    ],
    [
      "invent zero delivery",
      (f) => {
        Object.assign(f.terminal.unknownWrites[0]!, { returnedBytes: 0 });
      },
    ],
    [
      "completed outcome",
      (f) => {
        Object.assign(f.terminal.unknownWrites[0]!, { outcome: "completed" });
      },
    ],
    [
      "before exit_group",
      (f) => {
        f.terminal.unknownWrites[0]!.at = at(11);
      },
    ],
    [
      "after thread exit",
      (f) => {
        f.terminal.unknownWrites[0]!.at = at(15);
      },
    ],
    [
      "completed stderr before EOF",
      (f) => {
        f.stderr[0]!.at = at(8);
      },
    ],
    [
      "append unknown bytes",
      (f) => {
        f.stderr.push({ at: at(11), data: Buffer.from(cancellation) });
      },
    ],
    [
      "extra unknown attempt",
      (f) => {
        f.terminal.unknownWrites.push(structuredClone(f.terminal.unknownWrites[0]!));
      },
    ],
    [
      "missing thread exit",
      (f) => {
        f.terminal.threadExits.pop();
      },
    ],
    [
      "nonzero group exit",
      (f) => {
        f.terminal.exitGroups[0]!.code = 1;
      },
    ],
    [
      "unowned EOF",
      (f) => {
        f.eof[0]!.tid = 999;
      },
    ],
    [
      "unowned unresolved operation",
      (f) => {
        f.terminal.unfinishedOperations[0]!.tid = 999;
      },
    ],
    [
      "unanswered watcher",
      (f) => {
        f.client.pop();
      },
    ],
  ];
  for (const [name, mutate] of mutations) {
    const f = fixture();
    mutate(f);
    assert.throws(() => observe(f), name);
  }
  assert.throws(
    () =>
      observe(fixture(), (io) => {
        io[0]!.tid = 999;
      }),
    "foreign owned stream",
  );
});
