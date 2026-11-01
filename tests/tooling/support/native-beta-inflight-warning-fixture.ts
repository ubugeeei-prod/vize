import { fixture as original, at, type Packet } from "./native-beta-root-warning-fixture.ts";
import type { WatcherTimedIo } from "./native-beta-dto/stable-watcher-timing.ts";
import { observeRootWatchWarning } from "./native-beta-dto/historical-inflight-root-warning.ts";

// Constructed causal controls only; genuine saved-7d replay is a separate partial-trace receipt.
export function inflightFixture() {
  const f = original("B");
  f.owner.unknownWrites[0]!.at = at(17);
  f.owner.unfinishedOperations = [];
  f.owner.exitGroups[0]!.rawResult = "?";
  f.owner.exitGroups[0]!.line = 500;
  for (const exit of f.owner.threadExits) exit.line = 600;
  f.eof[0]!.completion!.returnedBy!.line = 600;
  f.server.splice(3, 0, {
    at: at(4),
    packet: {
      jsonrpc: "2.0",
      id: "ts1",
      method: "client/registerCapability",
      params: {
        registrations: [
          {
            id: "typescript-config-watch-id",
            method: "workspace/didChangeConfiguration",
            registerOptions: { section: ["js/ts", "typescript", "javascript", "editor"] },
          },
        ],
      },
    },
  });
  f.client.push({ at: at(4), packet: { jsonrpc: "2.0", id: "ts1", result: null } });
  f.client.sort((a, b) => a.at.localeCompare(b.at));
  f.server.sort((a, b) => a.at.localeCompare(b.at));
  return f;
}

export function inflightArgs(
  f: ReturnType<typeof inflightFixture>,
): Parameters<typeof observeRootWatchWarning> {
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
  for (const row of io)
    row.completion = {
      returnedAt: null,
      returnedBy: {
        at: at(Number(row.at.split(".")[1]) + 1),
        line: row.line + 1,
        basis: "next-same-thread-event",
      },
    };
  return [
    f.client.map((row) => row.packet),
    f.server.map((row) => row.packet),
    f.context,
    io,
    f.eof,
    f.owner,
    f.api,
  ];
}
