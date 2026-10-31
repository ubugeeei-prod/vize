import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { writeFileSync } from "node:fs";
import { join } from "node:path";
import { Recorder, sha256 } from "../../config/windows_consumer/common.ts";

/** Raw buffers and complete process outcome are durable before any semantic assertions. */
export function capture(
  record: Recorder,
  label: string,
  argv: string[],
  cwd: string,
  env = process.env,
  input?: Buffer,
) {
  const result = spawnSync(argv[0], argv.slice(1), {
    cwd,
    env,
    input,
    encoding: "buffer",
    timeout: 600_000,
    maxBuffer: 64 * 1024 * 1024,
    shell: false,
  });
  const stdout = result.stdout ?? Buffer.alloc(0),
    stderr = result.stderr ?? Buffer.alloc(0);
  for (const [name, raw] of [
    ["stdout", stdout],
    ["stderr", stderr],
    ...(input ? [["stdin", input] as const] : []),
  ] as const)
    writeFileSync(join(record.root, label + "." + name + ".raw"), raw, {
      flag: "wx",
    });
  const packet = {
    label,
    pid: result.pid,
    argv,
    cwd,
    status: result.status,
    signal: result.signal,
    error: result.error?.message ?? null,
    raw: {
      stdout: { bytes: stdout.length, sha256: sha256(stdout) },
      stderr: { bytes: stderr.length, sha256: sha256(stderr) },
      ...(input ? { stdin: { bytes: input.length, sha256: sha256(input) } } : {}),
    },
  };
  record.json(label + ".process.json", packet);
  return { ...packet, stdout, stderr };
}
export function completed(row: ReturnType<typeof capture>, status: number, emptyStderr = true) {
  assert.equal(row.error, null);
  assert.equal(row.signal, null);
  assert.equal(row.status, status, row.stderr.toString("utf8"));
  if (emptyStderr) assert.equal(row.stderr.length, 0);
}
