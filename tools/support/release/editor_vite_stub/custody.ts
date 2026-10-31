import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { join } from "node:path";
import { fileURLToPath } from "node:url";
import { asNumber, asRecord, asString, parseJson, type JsonRecord } from "../../json-value.ts";
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
  exit: number | null,
  nativeRequired = true,
): void {
  const raw = readFileSync(journal, "utf8");
  assert.ok(raw.endsWith("\n"), "native journal must contain complete records");
  const events = raw
    .slice(0, -1)
    .split("\n")
    .map((line) => asRecord(parseJson(line)));
  assert.ok(events.length > 0);
  const pid = asNumber(events[0].pid);
  assert.ok(Number.isSafeInteger(pid) && pid > 0);
  for (const event of events) {
    assert.equal(event.schema, "vize-public-native-custody-event-v1");
    assert.equal(event.pid, pid);
    assert.ok(["initialized", "attempt", "returned", "exit"].includes(asString(event.event)));
  }
  assert.equal(events[0].event, "initialized");
  assert.equal(events.filter((event) => event.event === "initialized").length, 1);
  const binary = asRecord(identity.binary);
  assert.equal(events[0].nativePath, binary.path);
  assert.equal(events[0].sha256, binary.sha256);
  const exits = events.filter((event) => event.event === "exit");
  assert.equal(exits.length, exit === null ? 0 : 1);
  if (exit !== null) {
    assert.equal(events.at(-1)?.event, "exit");
    assert.equal(events.at(-1)?.code, exit);
  }
  let attempts = 0,
    returns = 0;
  for (const event of events) {
    if (event.expectedNative !== true) continue;
    assert.equal(event.actualPath, binary.path);
    assert.equal(event.sha256, binary.sha256);
    if (event.event === "attempt") attempts++;
    if (event.event === "returned") {
      assert.equal(
        attempts,
        returns + 1,
        "native return must follow its own successful load attempt",
      );
      returns++;
    }
  }
  const returned = events.filter(
    (event) => event.event === "returned" && event.expectedNative === true,
  );
  assert.ok(returned.length <= 1);
  assert.equal(attempts, returned.length);
  if (nativeRequired) assert.equal(returned.length, 1);
  for (const event of returned) {
    assert.equal(event.actualPath, binary.path);
    assert.equal(event.sha256, binary.sha256);
  }
}
