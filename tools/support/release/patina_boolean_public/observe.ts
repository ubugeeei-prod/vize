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
  installedExecutables: readonly string[],
) {
  assertHost();
  assert(installedExecutables.length > 0 && installedExecutables.length <= 3);
  const candidates = installedExecutables.map((path) => {
    assert.equal(realpathSync(path), path, "canonical installed checker path required");
    return { path, before: digest(path) };
  });
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
  const selected = new Map<
    string,
    {
      pid: number;
      parent: number;
      executable: string;
      lsof: { stdout: string; stderr: string; status: number | null };
    }
  >();
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
    if (!child.pid || samples >= 600) return;
    samples++;
    const ps = spawnSync("/bin/ps", ["-axo", "pid=,ppid=,comm="], {
      encoding: "utf8",
      maxBuffer: 4 * 1024 * 1024,
      timeout: 5000,
    });
    if (ps.error || ps.signal || ps.status !== 0) return;
    // Include the CLI itself: absence of a descendant alone is not backend absence.
    const owned = [
      { pid: child.pid, ppid: process.pid, command: argv[0]! },
      ...descendants(ps.stdout, child.pid),
    ];
    for (const row of owned) {
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
      for (const candidate of candidates) {
        try {
          assertTextImage(text.stdout, row.pid, candidate.path);
        } catch {
          continue;
        }
        selected.set(row.pid + ":" + candidate.path, {
          pid: row.pid,
          parent: row.ppid,
          executable: candidate.path,
          lsof: { stdout: text.stdout, stderr: text.stderr, status: text.status },
        });
      }
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
  const images = candidates.map((candidate) => ({ ...candidate, after: digest(candidate.path) }));
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
      ownedProcesses: [...seen.values()],
      selected: [...selected.values()],
      selectionStatus: selected.size ? "observed-installed-text-image" : "unknown-unobserved",
      candidates: images,
    },
    limits:
      "Separate observed command; original npx is unobserved. lsof can attest an owned PID's installed text-image path, not raw protocol or SDK internals. Installed candidates do not prove selection; no observed candidate leaves selection unknown and does not prove backend absence. No runtime path override or product instrumentation.",
  };
  record.json(label + ".process.json", packet);
  assert.equal(failure, null, "actual CLI capture failed");
  assert.equal(outcome.signal, null);
  assert.equal(typeof child.pid, "number", "actual CLI PID required");
  for (const image of images)
    assert.deepEqual(image.after, image.before, "installed candidate executable bytes changed");
  return { ...packet, pid: child.pid!, stdout: out, stderr: err };
}
