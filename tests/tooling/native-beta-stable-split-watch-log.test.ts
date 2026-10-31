import assert from "node:assert/strict";
import fs from "node:fs";
import { test } from "node:test";
import {
  observeStableSplitWatchLog,
  requireCandidateStableStderr,
  type WatcherContext,
  type WatcherTimedIo,
} from "./support/native-beta-dto/stable-watcher-boundary.ts";

type Packet = Record<string, unknown>;
const record = (value: unknown) => value as Packet;
const fixtureContext = (
  JSON.parse(
    fs.readFileSync(
      new URL(
        "../_fixtures/differential/typechecker/native-beta-diagnostic-dto-10731/stable-post-shutdown-41099824102/identity.json",
        import.meta.url,
      ),
      "utf8",
    ),
  ) as { context: WatcherContext }
).context;
test("candidate stable requires zero stderr, including known historical cancellation forms", () => {
  assert.doesNotThrow(() => requireCandidateStableStderr(Buffer.alloc(0)));
  const f = splitLogFixture();
  const cancellation = Buffer.concat(f.stderr.slice(2).map((row) => row.data));
  const split = Buffer.concat(f.stderr.map((row) => row.data));
  for (const bytes of [cancellation, split])
    assert.throws(
      () => requireCandidateStableStderr(bytes),
      /Candidate stable stderr must be empty/u,
    );
});
// Constructed timing controls; actual8fb raw replay is retained independently.
function splitLogFixture() {
  const context = structuredClone(fixtureContext),
    project = context.projectRoots[1]!;
  const at = (value: number) => `1793408283.${String(value).padStart(6, "0")}`;
  const packet = (value: Packet, time: number) => ({ packet: value, at: at(time) });
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
    ),
    packet(
      {
        jsonrpc: "2.0",
        id: 3,
        method: "custom/projectInfo",
        params: { textDocument: { uri: `file://${project}/main.ts` } },
      },
      3,
    ),
    packet({ jsonrpc: "2.0", id: 4, method: "shutdown" }, 5),
    packet({ jsonrpc: "2.0", id: "ts2", result: null }, 8),
  ];
  const server = [
    packet(
      {
        jsonrpc: "2.0",
        id: 1,
        result: { serverInfo: { name: "typescript-go", version: "7.0.2" } },
      },
      2,
    ),
    packet({ jsonrpc: "2.0", id: 3, result: { configFilePath: project + "/tsconfig.json" } }, 4),
    packet(
      {
        jsonrpc: "2.0",
        id: "ts2",
        method: "client/registerCapability",
        params: {
          registrations: [
            {
              id: `root files for ${project}/tsconfig.json watcher 6.0`,
              method: "workspace/didChangeWatchedFiles",
              registerOptions: { watchers: [{ globPattern: project + "/**/*", kind: 7 }] },
            },
          ],
        },
      },
      6,
    ),
    packet({ jsonrpc: "2.0", id: 4, result: null }, 7),
    packet(
      {
        jsonrpc: "2.0",
        method: "window/logMessage",
        params: {
          type: 3,
          message: `Added new watch: root files for ${project}/tsconfig.json watcher 6.0`,
        },
      },
      12,
    ),
  ];
  const stderr = [
    `\t${project}/**/*\n`,
    "\n",
    "1 paths ineligible for watching\n",
    "errors updating watches: [failed to register file watcher: context canceled failed to register file watcher: context canceled]\n",
  ].map((text, index) => ({ at: at(10 + index), data: Buffer.from(text) }));
  return { context, client, server, stderr, eof: [at(9)] };
}
function splitLog(
  f: ReturnType<typeof splitLogFixture>,
  mutateIo?: (io: WatcherTimedIo[]) => void,
) {
  const frame = (packet: Packet) => {
    const body = Buffer.from(JSON.stringify(packet));
    return Buffer.concat([Buffer.from(`Content-Length: ${body.length}\r\n\r\n`), body]);
  };
  const io: WatcherTimedIo[] = [
    ...f.client.map((row, line) => ({
      ...row,
      data: frame(row.packet),
      op: "read",
      fd: 0,
      tid: 1,
      line,
    })),
    ...f.server.map((row, line) => ({
      ...row,
      data: frame(row.packet),
      op: "write",
      fd: 1,
      tid: 2,
      line,
    })),
    ...f.stderr.map((row, line) => ({ ...row, op: "write", fd: 2, tid: 3, line })),
  ];
  mutateIo?.(io);
  return observeStableSplitWatchLog(
    f.client.map((row) => row.packet),
    f.server.map((row) => row.packet),
    f.context,
    io,
    f.eof,
  );
}

test("a complete split watch log is only a historical baseline observation", () => {
  const f = splitLogFixture(),
    result = splitLog(f);
  assert.equal(result.nativeQualification, false);
  assert.equal(result.candidateFunctionalPass, false);
  assert.equal(result.lifecyclePass, false);
  assert.equal(result.registrationPass, true, "actual root watcher was acknowledged");
  assert.deepEqual(
    result.clientPackets,
    f.client.map((row) => row.packet),
  );
  assert.deepEqual(
    result.serverPackets,
    f.server.map((row) => row.packet),
  );
  assert.deepEqual(
    result.stderrWrites.map((row) => Buffer.from(row.hex, "hex")),
    f.stderr.map((row) => row.data),
  );
});

test("split logs reject wrong actors, payloads, timing, incomplete streams and unmatched requests", () => {
  type Split = ReturnType<typeof splitLogFixture>;
  const mutations: Array<[string, (f: Split) => void]> = [
    [
      "after",
      (f) => {
        f.context.role = "after";
      },
    ],
    [
      "other source",
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
      "other image",
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
      "wrong SDK executable",
      (f) => {
        f.context.sdkExecutable = "/foreign/tsc";
      },
    ],
    [
      "wrong native actor",
      (f) => {
        record(record(f.server[0]!.packet.result).serverInfo).version = "7.1.0-beta";
      },
    ],
    [
      "wrong header",
      (f) => {
        record(f.server[4]!.packet.params).message = "other watch";
      },
    ],
    [
      "header extra field",
      (f) => {
        record(f.server[4]!.packet.params).other = true;
      },
    ],
    [
      "missing header",
      (f) => {
        f.server.pop();
      },
    ],
    [
      "duplicate header",
      (f) => {
        f.server.push(structuredClone(f.server[4]!));
      },
    ],
    [
      "early header",
      (f) => {
        f.server[4]!.at = f.server[3]!.at;
      },
    ],
    [
      "foreign config",
      (f) => {
        record(f.server[1]!.packet.result).configFilePath = "/tmp/foreign/tsconfig.json";
      },
    ],
    [
      "bad watcher payload",
      (f) => {
        record(f.server[2]!.packet.params).registrations = [];
      },
    ],
    [
      "unanswered watcher",
      (f) => {
        f.client.pop();
      },
    ],
    [
      "watcher error",
      (f) => {
        f.client[3]!.packet.error = { code: -1 };
      },
    ],
    [
      "missing EOF",
      (f) => {
        f.eof = [];
      },
    ],
    [
      "duplicate EOF",
      (f) => {
        f.eof.push(f.eof[0]!);
      },
    ],
    [
      "EOF before ACK",
      (f) => {
        f.eof[0] = f.server[2]!.at;
      },
    ],
    [
      "wrong glob",
      (f) => {
        f.stderr[0]!.data = Buffer.from("\t/tmp/foreign/**/*\n");
      },
    ],
    [
      "extra warning",
      (f) => {
        f.stderr[3]!.data = Buffer.concat([f.stderr[3]!.data, Buffer.from("warning\n")]);
      },
    ],
    [
      "missing stderr write",
      (f) => {
        f.stderr.splice(1, 1);
      },
    ],
    [
      "extra request",
      (f) => {
        f.client.push({ packet: { jsonrpc: "2.0", id: 99, method: "other" }, at: f.eof[0]! });
      },
    ],
    [
      "standard exit notification",
      (f) => {
        f.client.push({ packet: { jsonrpc: "2.0", method: "exit" }, at: f.eof[0]! });
      },
    ],
  ];
  for (let index = 0; index < 4; index++)
    mutations.push([
      `stderr write ${index} before EOF`,
      (f) => {
        f.stderr[index]!.at = f.server[3]!.at;
      },
    ]);
  for (const [name, mutate] of mutations) {
    const f = splitLogFixture();
    mutate(f);
    assert.throws(() => splitLog(f), name);
  }
  assert.throws(
    () =>
      splitLog(splitLogFixture(), (io) => {
        io.find((row) => row.fd === 1)!.data = Buffer.from("Content-Length: 99\r\n\r\n{}");
      }),
    "incomplete frame",
  );
  assert.throws(
    () =>
      splitLog(splitLogFixture(), (io) => {
        io.pop();
      }),
    "missing owned stderr write",
  );
  assert.throws(
    () =>
      splitLog(splitLogFixture(), (io) => {
        io.splice(io.map((row) => row.fd).lastIndexOf(1), 1);
      }),
    "header from another owner cannot replace owned stdout",
  );
});
