/**
 * Recording content-mapper proxy: `node record-proxy.ts <exec...>` starts the
 * real mapper, forwards every byte both ways unchanged, and records each
 * transform reply and the session results to $FLOOR_RECORD_DIR (recording.ts).
 */

import { spawn } from "node:child_process";

import { bodyReader } from "./frames.ts";
import { type Session, recordingWriter } from "./recording.ts";

const [command, ...args] = process.argv.slice(2);
const dir = process.env.FLOOR_RECORD_DIR;
if (!command || !dir) {
  process.stderr.write("usage: FLOOR_RECORD_DIR=<dir> node record-proxy.ts <exec...>\n");
  process.exit(2);
}
const writer = recordingWriter(dir);
const pending = new Map<string, { method: string; fileName?: string }>();
const session: Session = { initialize: null, openProject: [] };
const mapper = spawn(command, args, { stdio: ["pipe", "pipe", "inherit"] });
const fromHost = bodyReader();
const fromMapper = bodyReader();

process.stdin.on("data", (chunk: Buffer) => {
  for (const body of fromHost(chunk)) {
    const message = JSON.parse(body.toString("utf8")) as {
      id?: unknown;
      method?: string;
      params?: { fileName?: string };
    };
    if (message.id != null && message.method) {
      pending.set(JSON.stringify(message.id), {
        method: message.method,
        fileName: message.params?.fileName,
      });
    }
  }
  mapper.stdin.write(chunk);
});
process.stdin.on("end", () => mapper.stdin.end());
mapper.stdin.on("error", (error: NodeJS.ErrnoException) => {
  if (error.code !== "EPIPE") throw error;
});
mapper.stdout.on("data", (chunk: Buffer) => {
  for (const body of fromMapper(chunk)) {
    const reply = JSON.parse(body.toString("utf8")) as { id?: unknown; result?: unknown };
    const request = pending.get(JSON.stringify(reply.id));
    if (!request || reply.result === undefined) continue;
    pending.delete(JSON.stringify(reply.id));
    if (request.method === "transform" && request.fileName) {
      writer.add(request.fileName, reply.result);
    } else if (request.method === "initialize" || request.method === "openProject") {
      if (request.method === "initialize") session.initialize = reply.result;
      else session.openProject.push(reply.result);
      // Written as it arrives: the host may stop the proxy without waiting for it.
      writer.session(session);
    }
  }
  process.stdout.write(chunk);
});
mapper.on("close", (code, signal) => process.exit(code ?? (signal ? 1 : 0)));
