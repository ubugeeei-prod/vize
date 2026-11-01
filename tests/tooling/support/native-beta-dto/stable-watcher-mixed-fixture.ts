import { observeHistoricalSplitWatchLog } from "./stable-watcher-mixed.ts";
import {
  historicalSource,
  stableImageSha256,
  type WatcherContext,
  type WatcherTimedIo,
} from "./stable-watcher-boundary.ts";
import type { NativeTimedObservation, nativeOwnerTerminalState } from "./native-io.ts";
type Packet = Record<string, unknown>;
export const at = (value: number) => `1793423752.${String(value).padStart(6, "0")}`;
export function fixture() {
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
export function inputs(f: ReturnType<typeof fixture>) {
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
export function observe(f: ReturnType<typeof fixture>, change?: (io: WatcherTimedIo[]) => void) {
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
export function exitBoundFixture() {
  const f = fixture();
  // Constructed counterpart of the saved bdfb EOF->exit_group entry bracket.
  f.server[4]!.tid = 52;
  f.server[4]!.line = 51;
  f.eof[0]!.completion!.returnedBy = { at: at(16), line: 17, basis: "next-same-thread-event" };
  f.owner.exitGroups = [{ tid: 38, at: at(16), code: 0, line: 17, rawResult: "?" }];
  return f;
}
