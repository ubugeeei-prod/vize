import fs from "node:fs";
import type { SpawnSyncReturns } from "node:child_process";
import { sha256 } from "./n8n-installed-authority.ts";
import { wholeProcessError } from "./oxlint-installed-html-authority.ts";
import type { HtmlCapture, HtmlEvent } from "./oxlint-installed-html-types.ts";

/** Save actual child bytes before any journal read or parse can fail. */
export function retainHtmlProcess(
  capture: HtmlCapture,
  save: () => void,
  metadata: Record<string, unknown>,
  result: SpawnSyncReturns<Buffer>,
  eventsPath?: string,
  journalPath?: string,
) {
  const packet: Record<string, unknown> = {
    ...metadata,
    kind: "process",
    pid: result.pid,
    status: result.status,
    signal: result.signal,
    error: result.error ? wholeProcessError(result.error) : null,
    stdoutBytes: Array.from(result.stdout ?? []),
    stderrBytes: Array.from(result.stderr ?? []),
    eventsPath: eventsPath ?? null,
    journalPath: journalPath ?? null,
  };
  capture.observations.push(packet);
  save();
  function retainedJournal(filename?: string) {
    if (!filename) return Buffer.alloc(0);
    try {
      const bytes = fs.existsSync(filename) ? fs.readFileSync(filename) : Buffer.alloc(0);
      packet[filename === eventsPath ? "eventJournal" : "nativeJournal"] = {
        path: filename,
        sha256: sha256(bytes),
        bytes: Array.from(bytes),
      };
      save();
      return bytes;
    } catch (error) {
      packet.journalReadFailure = {
        path: filename,
        error: error instanceof Error ? wholeProcessError(error) : error,
      };
      save();
      throw error;
    }
  }
  const eventBytes = retainedJournal(eventsPath);
  const nativeBytes = retainedJournal(journalPath);
  const decoder = new TextDecoder("utf-8", { fatal: true });
  const events = decoder
    .decode(eventBytes)
    .split("\n")
    .filter(Boolean)
    .map((line) => JSON.parse(line)) as HtmlEvent[];
  const rawJournal = decoder.decode(nativeBytes);
  packet.events = events;
  packet.rawNativeJournal = rawJournal;
  save();
  // Reject lossy child text only after its complete raw packet and journals are durable.
  const stdout = decoder.decode(result.stdout);
  const stderr = decoder.decode(result.stderr);
  return { events, rawJournal, stdout, stderr };
}
