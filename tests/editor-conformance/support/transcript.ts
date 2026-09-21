// Reading the tap's JSONL transcript while an editor is running. Drivers use
// it only to know *when* to press the next key (a request was answered, a
// publish settled); the verdict is always `conformance.ts` on the final file.
import fs from "node:fs";
import { setTimeout as sleep } from "node:timers/promises";

import { type Entry, normalizeUris } from "../conformance.ts";

export function readTranscript(file: string): Entry[] {
  if (!fs.existsSync(file)) return [];
  return fs
    .readFileSync(file, "utf8")
    .split("\n")
    .filter((line) => line.trim() !== "")
    .flatMap((line) => {
      try {
        return [JSON.parse(line) as Entry];
      } catch {
        return []; // the tap is mid-write; the next poll sees the whole line
      }
    });
}

export async function waitForTranscript<T>(
  file: string,
  label: string,
  probe: (entries: Entry[]) => T | undefined,
  timeoutMs = 120_000,
): Promise<T> {
  const deadline = Date.now() + timeoutMs;
  for (;;) {
    const found = probe(readTranscript(file));
    if (found !== undefined) return found;
    if (Date.now() > deadline)
      throw new Error(`timed out after ${timeoutMs}ms waiting for ${label}`);
    await sleep(100);
  }
}

/** The response to the first `method` request sent after `after` entries, once it exists. */
export function answeredRequest(entries: Entry[], method: string, after: number) {
  const request = entries.findIndex(
    (entry, index) =>
      index >= after && entry.dir === "c2s" && entry.msg?.method === method && entry.msg.id != null,
  );
  if (request < 0) return undefined;
  const { id } = entries[request].msg!;
  const session = entries[request].session;
  const response = entries.findIndex(
    (entry, index) =>
      index > request &&
      entry.session === session &&
      entry.dir === "s2c" &&
      entry.msg?.method == null &&
      entry.msg?.id === id,
  );
  return response < 0 ? undefined : { request, response, message: entries[response].msg! };
}

/** The newest publish for `uri` (normalized) after `after` entries, if any. */
export function latestPublish(entries: Entry[], uri: string, roots: string[], after = 0) {
  for (let index = entries.length - 1; index >= after; index -= 1) {
    const message = entries[index].msg;
    if (entries[index].dir !== "s2c" || message?.method !== "textDocument/publishDiagnostics")
      continue;
    if (normalizeUris(message.params.uri, roots) === uri) return { index, params: message.params };
  }
  return undefined;
}
