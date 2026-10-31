import assert from "node:assert/strict";
import { spawn, spawnSync } from "node:child_process";
import { realpathSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import type { Recorder } from "../../config/windows_consumer/common.ts";
import { sha256 } from "../../config/windows_consumer/common.ts";
import { digest } from "../editor_vite_stub/filesystem.ts";
import { assertHost, assertTextImage, descendants, type ProcessRow } from "./contract.ts";

/** Observe a separate real CLI invocation; never replace its resolver or checker. */
export async function captureSelectedRuntime(
  record: Recorder,
  label: string,
  argv: string[],
  cwd: string,
  env: NodeJS.ProcessEnv,
  expectedExecutable: string,
) {
  assertHost();
  const expected = realpathSync(expectedExecutable);
  assert.equal(expected, expectedExecutable, "canonical installed checker path required");
  const before = digest(expected);
  const child = spawn(argv[0]!, argv.slice(1), {
    cwd,
    env,
    stdio: ["ignore", "pipe", "pipe"],
    shell: false,
  });
  const stdout: Buffer[] = [],
    stderr: Buffer[] = [];
  let received = 0,
    failure: string | null = null,
    samples = 0;
  const seen = new Map<number, ProcessRow>(),
    attempts = new Map<number, number>();
  let selected: {
    pid: number;
    parent: number;
    executable: string;
    lsof: { stdout: string; stderr: string; status: number | null };
  } | null = null;
  const collect = (target: Buffer[]) => (chunk: Buffer) => {
    received += chunk.length;
    if (received > 64 * 1024 * 1024) {
      failure ??= "capture exceeds existing 64MiB evidence cap";
      child.kill("SIGKILL");
    } else target.push(chunk);
  };
  child.stdout.on("data", collect(stdout));
  child.stderr.on("data", collect(stderr));
  const observe = () => {
    if (!child.pid || selected || samples >= 600) return;
    samples++;
    const ps = spawnSync("/bin/ps", ["-axo", "pid=,ppid=,comm="], {
      encoding: "utf8",
      maxBuffer: 4 * 1024 * 1024,
      timeout: 5000,
    });
    if (ps.error || ps.signal || ps.status !== 0) return;
    for (const row of descendants(ps.stdout, child.pid)) {
      seen.set(row.pid, row);
      assert(seen.size <= 256, "bounded owned descendant population");
      const count = attempts.get(row.pid) ?? 0;
      if (count >= 3) continue;
      attempts.set(row.pid, count + 1);
      const text = spawnSync("/usr/sbin/lsof", ["-a", "-p", String(row.pid), "-d", "txt", "-Fpn"], {
        encoding: "utf8",
        maxBuffer: 1024 * 1024,
        timeout: 5000,
      });
      if (text.error || text.signal || text.status !== 0) continue;
      try {
        assertTextImage(text.stdout, row.pid, expected);
      } catch {
        continue;
      }
      selected = {
        pid: row.pid,
        parent: row.ppid,
        executable: expected,
        lsof: { stdout: text.stdout, stderr: text.stderr, status: text.status },
      };
      break;
    }
  };
  const interval = setInterval(() => {
    try {
      observe();
    } catch (error) {
      failure ??= "process observer failed: " + String(error);
      child.kill("SIGKILL");
    }
  }, 50);
  const deadline = setTimeout(() => {
    failure ??= "existing outer 600s capture deadline exceeded";
    child.kill("SIGKILL");
  }, 600_000);
  const outcome = await new Promise<{ status: number | null; signal: NodeJS.Signals | null }>(
    (resolve) => {
      child.on("error", (error) => {
        failure ??= error.message;
      });
      child.on("close", (status, signal) => resolve({ status, signal }));
    },
  );
  clearInterval(interval);
  clearTimeout(deadline);
  const out = Buffer.concat(stdout),
    err = Buffer.concat(stderr);
  writeFileSync(join(record.root, label + ".stdout.raw"), out, { flag: "wx" });
  writeFileSync(join(record.root, label + ".stderr.raw"), err, { flag: "wx" });
  const after = digest(expected);
  const packet = {
    label,
    pid: child.pid ?? null,
    argv,
    cwd,
    ...outcome,
    error: failure,
    raw: {
      stdout: { bytes: out.length, sha256: sha256(out) },
      stderr: { bytes: err.length, sha256: sha256(err) },
    },
    observer: {
      kind: "macOS process-only ps/lsof",
      samples,
      descendants: [...seen.values()],
      selected,
      expected: { path: expected, before, after },
    },
    limits:
      "Separate observed command; original npx command is captured without this observer. lsof attests the live descendant text-image path, not raw protocol packets or SDK internals. No runtime path override, wrapper replacement or product-function instrumentation.",
  };
  record.json(label + ".process.json", packet);
  assert.equal(failure, null, "actual CLI capture failed");
  assert.equal(outcome.signal, null);
  assert(
    selected,
    "actual selected checker executable was not observed; do not infer it from package presence",
  );
  assert.equal(typeof child.pid, "number", "actual CLI PID required");
  assert.deepEqual(after, before, "installed selected executable bytes changed");
  return { ...packet, pid: child.pid!, stdout: out, stderr: err };
}
