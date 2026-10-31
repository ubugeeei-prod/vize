import {
  observeHistoricalRootWatchWarning,
  historicalRootWarning,
  type HistoricalProbeApi,
} from "./native-beta-dto/historical-root-watch-warning.ts";
import {
  historicalSource,
  stableImageSha256,
  type WatcherContext,
  type WatcherTimedIo,
} from "./native-beta-dto/stable-watcher-boundary.ts";
import { type NativeTimedObservation } from "./native-beta-dto/native-io.ts";
export type Packet = Record<string, unknown>;
export const at = (n: number) => `1793460256.${String(n).padStart(6, "0")}`;
// Constructed refusal contracts. Authenticated A/B replay uses the retained real traces separately.
export function fixture(kind: "A" | "B") {
  const root = "/tmp/.tmpUnit01",
    config = root + "/tsconfig.json";
  const context: WatcherContext = {
    source: historicalSource,
    role: "before",
    sdkVersion: "7.0.2",
    sdkSha256: stableImageSha256,
    sdkExecutable: "/sdk/node_modules/@typescript/typescript-linux-x64/lib/tsc",
    projectRoots: [root],
  };
  const timed = (packet: Packet, n: number) => ({ packet, at: at(n) });
  const client = [
    timed(
      {
        jsonrpc: "2.0",
        id: 1,
        method: "initialize",
        params: {
          rootPath: root + "/node_modules/.vize/corsa-overlay",
          rootUri: "file://" + root + "/node_modules/.vize/corsa-overlay",
        },
      },
      1,
    ),
    timed({ jsonrpc: "2.0", id: 2, method: "custom/initializeAPISession", params: null }, 3),
    timed(
      {
        jsonrpc: "2.0",
        id: 3,
        method: "custom/projectInfo",
        params: { textDocument: { uri: "file://" + root + "/main.ts" } },
      },
      5,
    ),
    timed({ jsonrpc: "2.0", id: 4, method: "shutdown" }, 7),
    timed({ jsonrpc: "2.0", id: "ts2", result: null }, 10),
  ];
  const registration = (name: string, glob: string) => ({
    registrations: [
      {
        id: name,
        method: "workspace/didChangeWatchedFiles",
        registerOptions: { watchers: [{ globPattern: glob, kind: 7 }] },
      },
    ],
  });
  const server = [
    timed(
      {
        jsonrpc: "2.0",
        id: 1,
        result: { serverInfo: { name: "typescript-go", version: "7.0.2" } },
      },
      2,
    ),
    timed({ jsonrpc: "2.0", id: 2, result: {} }, 4),
    timed({ jsonrpc: "2.0", id: 3, result: { configFilePath: config } }, 6),
    timed(
      {
        jsonrpc: "2.0",
        id: "ts2",
        method: "client/registerCapability",
        params: registration(`root files for ${config} watcher 6.0`, root + "/**/*"),
      },
      8,
    ),
    timed({ jsonrpc: "2.0", id: 4, result: null }, 9),
  ];
  if (kind === "A") {
    server.push(
      ...[`Added new watch: root files for ${config} watcher 6.0`, `\t${root}/**/*`, ""].map(
        (message, i) =>
          timed(
            { jsonrpc: "2.0", method: "window/logMessage", params: { type: 3, message } },
            12 + i,
          ),
      ),
    );
    server.push(
      timed(
        {
          jsonrpc: "2.0",
          id: "ts3",
          method: "client/registerCapability",
          params: registration(
            `program files for ${config} watcher 7.0`,
            "/sdk/node_modules/@typescript/typescript-linux-x64/lib/**/*",
          ),
        },
        15,
      ),
    );
  }
  const header = `Added new watch: root files for ${config} watcher 6.0\n\t${root}/**/*\n\n`;
  const stderr =
    kind === "A"
      ? [{ at: at(16), data: Buffer.from(historicalRootWarning) }]
      : [
          { at: at(12), data: Buffer.from(header.slice(0, header.indexOf("\n") + 1)) },
          { at: at(13), data: Buffer.from(`\t${root}/**/*\n`) },
          { at: at(14), data: Buffer.from("\n") },
        ];
  const owner: Parameters<typeof observeHistoricalRootWatchWarning>[5] = {
    nativeOwner: 41,
    members: [
      { tid: 41, parent: null },
      { tid: 42, parent: 41 },
      { tid: 43, parent: 41 },
      { tid: 44, parent: 42 },
    ],
    threadExits: [
      { tid: 41, at: at(26), code: 0 },
      { tid: 42, at: at(25), code: 0 },
      { tid: 43, at: at(24), code: 0 },
      { tid: 44, at: at(23), code: 0 },
    ],
    exitGroups: [{ tid: kind === "A" ? 43 : 42, at: at(18), code: 0 }],
    unknownWrites: [],
    failedWrites: [],
    zeroCapacityReads: [],
    unfinishedOperations: [],
  };
  if (kind === "B") {
    owner.unknownWrites.push({
      at: at(19),
      op: "write",
      syscall: "write",
      fd: 2,
      data: Buffer.from(historicalRootWarning),
      requestedBytes: 32,
      returnedBytes: null,
      iovCount: null,
      iovecs: null,
      argumentsComplete: true,
      rawArguments: "constructed contract",
      rawResult: "?",
      outcome: "unknown",
      tid: 44,
      file: "syscalls.44",
      line: 303,
      startedLine: 303,
    });
    owner.unfinishedOperations.push({
      tid: 41,
      file: "syscalls.41",
      line: 400,
      at: at(20),
      raw: `${at(20)} ???( `,
    });
  }
  const eof: Array<NativeTimedObservation & { fd: number }> = [
    {
      tid: 43,
      fd: 0,
      at: at(11),
      line: 100,
      startedLine: 100,
      completion: {
        returnedAt: null,
        returnedBy: { at: at(kind === "A" ? 18 : 24), line: 101, basis: "next-same-thread-event" },
      },
    },
  ];
  const api: HistoricalProbeApi = {
    client: [
      { jsonrpc: "2.0", id: 1, method: "initialize", params: null },
      {
        jsonrpc: "2.0",
        id: 2,
        method: "parseConfigFile",
        params: { file: root + "/node_modules/.vize/corsa-overlay/tsconfig.json" },
      },
    ],
    server: [
      { jsonrpc: "2.0", id: 1, result: {} },
      { jsonrpc: "2.0", id: 2, result: {} },
    ],
    precedingNonProtocolBytes: 5,
  };
  return { kind, context, client, server, stderr, owner, eof, api };
}
export function observe(f: ReturnType<typeof fixture>, mutate?: (io: WatcherTimedIo[]) => void) {
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
    ...f.stderr.map((row, line) => ({ ...row, op: "write", fd: 2, tid: 44, line: 300 + line })),
  ];
  for (const row of io) row.completion = { returnedAt: row.at, returnedBy: null };
  mutate?.(io);
  return observeHistoricalRootWatchWarning(
    f.client.map((row) => row.packet),
    f.server.map((row) => row.packet),
    f.context,
    io,
    f.eof,
    f.owner,
    f.api,
  );
}
