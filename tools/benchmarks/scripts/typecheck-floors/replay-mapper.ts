/**
 * Zero-wait floor mapper: serves the recording in $FLOOR_REPLAY_DIR
 * (recording.ts) byte for byte, so every transform reply is ready the moment
 * the host asks. A file the recording lacks is a JSON-RPC error.
 */

import { bodyReader, writeFrame, writeMessage } from "./frames.ts";
import { readIndex, readSession } from "./recording.ts";

const dir = process.env.FLOOR_REPLAY_DIR;
if (!dir) {
  process.stderr.write("usage: FLOOR_REPLAY_DIR=<dir> node replay-mapper.ts\n");
  process.exit(2);
}
const session = readSession(dir);
const { bodies, index } = readIndex(dir);
let opened = 0;
const read = bodyReader();
process.stdin.on("data", (chunk: Buffer) => {
  for (const body of read(chunk)) {
    const message = JSON.parse(body.toString("utf8")) as {
      id?: unknown;
      method?: string;
      params?: { fileName?: string };
    };
    if (message.id == null) continue;
    const id = message.id;
    if (message.method === "transform") {
      const entry = index.get(message.params?.fileName ?? "");
      if (entry) {
        const [offset, length] = entry;
        const head = Buffer.from(`{"id":${JSON.stringify(id)},`);
        writeFrame(process.stdout, Buffer.concat([head, bodies.subarray(offset, offset + length)]));
      } else {
        const error = { code: -32603, message: `not recorded: ${message.params?.fileName}` };
        writeMessage(process.stdout, { error, id, jsonrpc: "2.0" });
      }
      continue;
    }
    const result =
      message.method === "initialize"
        ? session.initialize
        : message.method === "openProject"
          ? (session.openProject[Math.min(opened++, session.openProject.length - 1)] ?? null)
          : null;
    writeMessage(process.stdout, { id, jsonrpc: "2.0", result });
  }
});
process.stdin.on("end", () => process.exit(0));
