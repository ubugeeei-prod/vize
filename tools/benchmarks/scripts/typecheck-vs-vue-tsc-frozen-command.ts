/** One direct normal command; full streams/status/resources survive every validation failure. */
import assert from "node:assert/strict";
import { spawn, spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { gnuTime, identity, inventory, save } from "./typecheck-vs-vue-tsc-frozen-evidence.ts";
export type Command = { executable: string; args: string[]; cwd: string };
/** Explicit source data is test-only; the campaign always uses the real process.env default. */
export function environment(cache: string, sourceEnvironment: NodeJS.ProcessEnv = process.env) {
  for (const key of [
    "GOGC",
    "GOMEMLIMIT",
    "NODE_OPTIONS",
    "NODE_PATH",
    "CORSA_PATH",
    "TSGO_PATH",
    "TSRS_PATH",
    "VIZE_CHECKERS",
    "VIZE_THREADS",
    "GOMAXPROCS",
    "RAYON_NUM_THREADS",
    "RUSTFLAGS",
    "CARGO_ENCODED_RUSTFLAGS",
  ])
    assert(!sourceEnvironment[key], "unexpected inherited policy: " + key);
  fs.mkdirSync(cache, { recursive: true });
  const inherited = Object.fromEntries(
    ["PATH", "USER", "LOGNAME", "SHELL", "SYSTEMROOT", "LD_LIBRARY_PATH"]
      .filter((key) => sourceEnvironment[key] !== undefined)
      .map((key) => [key, sourceEnvironment[key]]),
  );
  assert(!inherited.LD_LIBRARY_PATH, "unexpected loader policy");
  const env = {
    ...inherited,
    LANG: "C",
    LC_ALL: "C",
    HOME: path.join(cache, "home"),
    XDG_CACHE_HOME: path.join(cache, "xdg"),
    TMPDIR: path.join(cache, "tmp"),
    NO_COLOR: "1",
    NODE_DISABLE_COMPILE_CACHE: "1",
  };
  for (const dir of [env.HOME, env.XDG_CACHE_HOME, env.TMPDIR])
    fs.mkdirSync(dir, { recursive: true });
  return env;
}
export function affinity() {
  const text = fs.readFileSync("/proc/self/status", "utf8"),
    match = /^Cpus_allowed_list:\s*(.*)$/mu.exec(text);
  assert(match);
  const available = match[1].split(",").flatMap((part) => {
    const [lo, hi] = part.split("-").map(Number);
    return Array.from({ length: (hi ?? lo) - lo + 1 }, (_, i) => lo + i);
  });
  assert(available.length >= 8);
  const selected = available.slice(0, 8).join(",");
  const result = spawnSync("taskset", ["-pc", selected, String(process.pid)], { encoding: "utf8" });
  assert.equal(result.status, 0, result.stderr);
  return {
    before: match[1],
    selected,
    result: result.stdout,
    actual: fs.readFileSync("/proc/self/status", "utf8"),
  };
}
export function processSample(rootPid: number, atMs: number) {
  const rows: {
      pid: number;
      ppid: number;
      startTicks: string;
      rssKiB: number;
      executable: string | null;
    }[] = [],
    gaps: { pid: number; error: string }[] = [];
  let all: string[] = [];
  try {
    all = fs.readdirSync("/proc").filter((name) => /^\d+$/u.test(name));
  } catch (error) {
    gaps.push({ pid: rootPid, error: String(error) });
  }
  const records = new Map<number, { ppid: number; start: string }>();
  for (const name of all) {
    try {
      const stat = fs.readFileSync(`/proc/${name}/stat`, "utf8"),
        fields = stat.slice(stat.lastIndexOf(")") + 2).split(" ");
      records.set(Number(name), { ppid: Number(fields[1]), start: fields[19] });
    } catch {
      /* Unrelated exited processes are not descendant gaps. */
    }
  }
  const selected = new Set([rootPid]);
  for (let changed = true; changed;) {
    changed = false;
    for (const [pid, row] of records)
      if (!selected.has(pid) && selected.has(row.ppid)) {
        selected.add(pid);
        changed = true;
      }
  }
  for (const pid of selected) {
    try {
      const status = fs.readFileSync(`/proc/${pid}/status`, "utf8"),
        rss = /^VmRSS:\s*(\d+) kB$/mu.exec(status);
      let executable: string | null = null;
      try {
        executable = fs.readlinkSync(`/proc/${pid}/exe`);
      } catch {
        /* Explicit null. */
      }
      const record = records.get(pid);
      assert(record);
      rows.push({
        pid,
        ppid: record.ppid,
        startTicks: record.start,
        rssKiB: rss ? Number(rss[1]) : 0,
        executable,
      });
    } catch (error) {
      gaps.push({ pid, error: String(error) });
    }
  }
  return {
    atMs,
    rows,
    gaps,
    simultaneousObservedTreeRssKiB: rows.reduce((n, row) => n + row.rssKiB, 0),
  };
}
export async function command(
  out: string,
  spec: Command,
  env: NodeJS.ProcessEnv,
  timeoutMs = 2_400_000,
  testTimeExecutable?: string,
) {
  fs.mkdirSync(out); // Never reuse an attempt.
  save(path.join(out, "request.json"), {
    ...spec,
    executableIdentity: identity(spec.executable),
    environment: env,
    cacheBefore: inventory(env.HOME! + "/.."),
    osPageCache: "UNCONTROLLED; no host cache-drop authority exercised",
    timeoutMs,
  });
  const stdout = fs.openSync(path.join(out, "stdout.bin"), "wx"),
    stderr = fs.openSync(path.join(out, "stderr.bin"), "wx");
  const started = performance.now(),
    samples: ReturnType<typeof processSample>[] = [];
  const child = spawn(
    testTimeExecutable ?? "/usr/bin/time",
    ["-v", "-o", path.join(out, "gnu-time.txt"), spec.executable, ...spec.args],
    { cwd: spec.cwd, env, detached: true, stdio: ["ignore", stdout, stderr] },
  );
  let spawnError: string | null = null,
    timedOut = false;
  const timer = setInterval(() => {
    if (child.pid) samples.push(processSample(child.pid, performance.now() - started));
  }, 50);
  const deadline = setTimeout(() => {
    timedOut = true;
    if (child.pid) process.kill(-child.pid, "SIGKILL");
  }, timeoutMs);
  const outcome = await new Promise<{ status: number | null; signal: NodeJS.Signals | null }>(
    (resolve) => {
      child.on("error", (error) => {
        spawnError = String(error);
      });
      child.on("close", (status, signal) => resolve({ status, signal }));
    },
  );
  const wallMs = performance.now() - started;
  clearInterval(timer);
  clearTimeout(deadline);
  fs.closeSync(stdout);
  fs.closeSync(stderr);
  // Raw and status are durable before identity/resource/cache/diagnostic guards.
  const raw = {
    ...outcome,
    error: spawnError,
    timedOut,
    pid: child.pid ?? null,
    wallMs,
    samples,
    samplingIntervalMs: 50,
    memoryScope:
      "sampled /proc root+descendants at each window; gaps/short children possible; not PSS, exact peak, or sum of independent peaks",
  };
  save(path.join(out, "process.json"), raw);
  save(path.join(out, "cache-after.json"), inventory(env.HOME! + "/.."));
  assert.equal(spawnError, null);
  assert.equal(timedOut, false);
  assert.equal(outcome.signal, null);
  assert(outcome.status !== null && [0, 1, 2].includes(outcome.status));
  const resources = gnuTime(fs.readFileSync(path.join(out, "gnu-time.txt"), "utf8"));
  const result = {
    ...raw,
    ...resources,
    executableAfter: identity(spec.executable),
    stdout: identity(path.join(out, "stdout.bin")),
    stderr: identity(path.join(out, "stderr.bin")),
  };
  save(path.join(out, "measurement.json"), result);
  return result;
}
