import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { join } from "node:path";
import { fileURLToPath } from "node:url";
import { asRecord, asString, parseJson, type JsonRecord } from "../../json-value.ts";
import type { Recorder } from "../../config/windows_consumer/common.ts";

export const hook = fileURLToPath(new URL("../public_install/native-custody.cjs", import.meta.url));
export function observed(root: string, identity: JsonRecord, record: Recorder, label: string) {
  const binary = asRecord(identity.binary);
  const journal = join(record.root, label + ".native.ndjson");
  const env: NodeJS.ProcessEnv = {
    ...process.env,
    VIZE_PUBLIC_NATIVE_CUSTODY: JSON.stringify({
      schema: "vize-public-native-custody-v1",
      installRoot: root,
      nativePath: binary.path,
      nativeSha256: binary.sha256,
      journalPath: journal,
    }),
  };
  return { env, journal };
}
export function verifyJournal(
  journal: string,
  identity: JsonRecord,
  exit: number,
  nativeRequired = true,
): void {
  const events = readFileSync(journal, "utf8")
    .trim()
    .split("\n")
    .map((line) => asRecord(parseJson(line)));
  assert.ok(events.length >= (nativeRequired ? 4 : 2));
  for (const event of events) assert.equal(event.schema, "vize-public-native-custody-event-v1");
  assert.equal(events[0].event, "initialized");
  const binary = asRecord(identity.binary);
  assert.equal(events[0].nativePath, binary.path);
  assert.equal(events[0].sha256, binary.sha256);
  assert.equal(events.at(-1)?.event, "exit");
  assert.equal(events.at(-1)?.code, exit);
  assert.equal(
    events.some((event) => ["failed", "rejected"].includes(asString(event.event))),
    false,
  );
  const returned = events.filter(
    (event) => event.event === "returned" && event.expectedNative === true,
  );
  assert.ok(returned.length <= 1);
  if (nativeRequired) assert.equal(returned.length, 1);
  for (const event of returned) {
    assert.equal(event.actualPath, binary.path);
    assert.equal(event.sha256, binary.sha256);
  }
}
