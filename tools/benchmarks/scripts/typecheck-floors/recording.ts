/**
 * Recorded mapper replies for the zero-wait floors. `bodies.bin` holds each
 * transform reply minus its `{"id":…,` head, `index.jsonl` its
 * `[fileName, offset, length]`, and `session.json` the recorded `initialize`
 * and `openProject` results, so a replay serves bytes without parsing them.
 */

import { mkdirSync, openSync, readFileSync, writeFileSync, writeSync } from "node:fs";
import { join } from "node:path";

export type Session = { initialize: unknown; openProject: unknown[] };
export type Recorded = { fileName: string; result: Record<string, unknown> };

/** The reply object after its `{`, with no id: `"jsonrpc":"2.0","result":{…}}`. */
export const replyTail = (result: unknown): Buffer =>
  Buffer.from(JSON.stringify({ jsonrpc: "2.0", result }).slice(1));

export function recordingWriter(dir: string): {
  add: (fileName: string, result: unknown) => void;
  session: (session: Session) => void;
} {
  mkdirSync(dir, { recursive: true });
  const bodies = openSync(join(dir, "bodies.bin"), "w");
  const index = openSync(join(dir, "index.jsonl"), "w");
  let offset = 0;
  return {
    add(fileName, result) {
      const body = replyTail(result);
      writeSync(bodies, body);
      writeSync(index, `${JSON.stringify([fileName, offset, body.length])}\n`);
      offset += body.length;
    },
    session: (session) => writeFileSync(join(dir, "session.json"), JSON.stringify(session)),
  };
}

export function readSession(dir: string): Session {
  return JSON.parse(readFileSync(join(dir, "session.json"), "utf8")) as Session;
}

/** The last recorded reply per file, as offsets into `bodies`. */
export function readIndex(dir: string): { bodies: Buffer; index: Map<string, [number, number]> } {
  const bodies = readFileSync(join(dir, "bodies.bin"));
  const index = new Map<string, [number, number]>();
  for (const line of readFileSync(join(dir, "index.jsonl"), "utf8").split("\n")) {
    if (!line) continue;
    const [fileName, offset, length] = JSON.parse(line) as [string, number, number];
    index.set(fileName, [offset, length]);
  }
  return { bodies, index };
}

/** Rewrite every recorded result with `edit` (given the session's position encoding); sums its counts. */
export function transformRecording(
  source: string,
  target: string,
  edit: (result: Record<string, unknown>, encoding: "utf-8" | "utf-16") => Record<string, number>,
): { files: number; bytesBefore: number; bytesAfter: number; counts: Record<string, number> } {
  const session = readSession(source);
  const negotiated = (session.initialize as { positionEncoding?: string } | null)?.positionEncoding;
  const encoding = negotiated === "utf-16" ? "utf-16" : "utf-8";
  const { bodies, index } = readIndex(source);
  const writer = recordingWriter(target);
  const counts: Record<string, number> = {};
  let bytesBefore = 0;
  let bytesAfter = 0;
  for (const [fileName, [offset, length]] of index) {
    const reply = JSON.parse(`{${bodies.subarray(offset, offset + length).toString("utf8")}`) as {
      result: Record<string, unknown>;
    };
    bytesBefore += length;
    for (const [key, value] of Object.entries(edit(reply.result, encoding)))
      counts[key] = (counts[key] ?? 0) + value;
    bytesAfter += replyTail(reply.result).length;
    writer.add(fileName, reply.result);
  }
  writer.session(session);
  return { files: index.size, bytesBefore, bytesAfter, counts };
}
