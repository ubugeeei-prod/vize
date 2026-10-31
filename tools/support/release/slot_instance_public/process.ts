import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { writeFileSync } from "node:fs";
import { join } from "node:path";
import type { Recorder } from "../../config/windows_consumer/common.ts";
import { sha256 } from "../computed_event_custody.ts";
import { asRecord, parseJson } from "../../json-value.ts";

export function assertJournalPid(raw: string, pid: number) {
  assert.ok(Number.isSafeInteger(pid) && pid > 0);
  assert.ok(raw.endsWith("\n"));
  const rows = raw.slice(0, -1).split("\n");
  assert.ok(rows.length > 0);
  for (const row of rows)
    assert.equal(
      asRecord(parseJson(row)).pid,
      pid,
      "native journal belongs to the actual spawned PID",
    );
}

/** Persist raw bytes, PID and full outcome before checking any result. */
export function capture(
  record: Recorder,
  label: string,
  argv: string[],
  cwd: string,
  env = process.env,
) {
  const result = spawnSync(argv[0], argv.slice(1), {
    cwd,
    env,
    encoding: "buffer",
    timeout: 240_000,
    maxBuffer: 64 * 1024 * 1024,
    shell: false,
  });
  const stdout = result.stdout ?? Buffer.alloc(0),
    stderr = result.stderr ?? Buffer.alloc(0);
  for (const [name, raw] of [
    ["stdout", stdout],
    ["stderr", stderr],
  ] as const)
    writeFileSync(join(record.root, label + "." + name + ".raw"), raw, { flag: "wx" });
  const packet = {
    label,
    argv,
    cwd,
    pid: result.pid,
    status: result.status,
    signal: result.signal,
    error: result.error?.message ?? null,
    stdout: new TextDecoder("utf-8", { fatal: true }).decode(stdout),
    stderr: new TextDecoder("utf-8", { fatal: true }).decode(stderr),
    raw: {
      stdout: { bytes: stdout.length, sha256: sha256(stdout) },
      stderr: { bytes: stderr.length, sha256: sha256(stderr) },
    },
  };
  record.json(label + ".process.json", packet);
  return packet;
}
