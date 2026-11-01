/** Normal direct CLI command with passive /proc RSS windows; no heavy PSS reader. */
import assert from "node:assert/strict";
import { spawn, execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { prepareColdScale, verifyColdDiagnostics } from "./cold-typechecker-scale.ts";
import { coldScaleCommand } from "./cold-typechecker-scale-command.ts";
import { linuxPeakRss, linuxCommandCpu } from "./cold-typechecker-scale-paired-stats.ts";
import { seal } from "./cold-typechecker-scale-native-inputs.ts";

type ProcessRow = {
  pid: number;
  parent: number;
  birthTicks: string;
  rssBytes: number;
  executable: string | null;
};
export function selectObservedTree(rows: ProcessRow[], pid: number, birthTicks: string) {
  const all = new Map(rows.map((row) => [row.pid, row]));
  const root = all.get(pid);
  if (!root) return [];
  assert.equal(root.birthTicks, birthTicks, "CLI PID reuse");
  return rows.filter((row) => {
    let current: ProcessRow | undefined = row;
    const visited = new Set<number>();
    while (current && !visited.has(current.pid)) {
      if (current.pid === pid) return true;
      visited.add(current.pid);
      current = all.get(current.parent);
    }
    return false;
  });
}

function procRows(pageBytes: number) {
  const rows: ProcessRow[] = [],
    unavailable: string[] = [];
  for (const name of fs.readdirSync("/proc").filter((name) => /^\d+$/u.test(name))) {
    try {
      const stat = fs.readFileSync(`/proc/${name}/stat`, "utf8");
      const fields = stat
        .slice(stat.lastIndexOf(") ") + 2)
        .trim()
        .split(/\s+/u);
      let executable: string | null = null;
      try {
        executable = fs.realpathSync(`/proc/${name}/exe`);
      } catch {
        /* Observed stat may outlive exe. */
      }
      rows.push({
        pid: Number(name),
        parent: Number(fields[1]),
        birthTicks: fields[19],
        rssBytes: Math.max(0, Number(fields[21])) * pageBytes,
        executable,
      });
    } catch {
      unavailable.push(name);
    }
  }
  return { rows, unavailable };
}
const sha = (body: Buffer) => createHash("sha256").update(body).digest("hex");
const write = (file: string, value: unknown) =>
  fs.writeFileSync(file, JSON.stringify(value, null, 2) + "\n", { flag: "wx" });

export async function captureReadBufferCommand(options: {
  provider: string;
  runRoot: string;
  directory: string;
  files: number;
  binary: string;
}) {
  const { provider, runRoot, directory, files, binary } = options;
  assert.equal(process.platform, "linux");
  assert(!fs.existsSync(directory));
  fs.mkdirSync(directory);
  for (const name of ["project", "xdg-cache", "tmp"])
    fs.rmSync(path.join(runRoot, name), { recursive: true, force: true });
  fs.mkdirSync(path.join(runRoot, "tmp"));
  const corpus = prepareColdScale(path.join(runRoot, "project"), provider, files, false);
  write(path.join(directory, "corpus.json"), corpus);
  const selected = coldScaleCommand({
    provider,
    out: runRoot,
    app: corpus.app,
    tool: "cli",
    binary,
  });
  const image = {
    cli: sha(fs.readFileSync(binary)),
    native: sha(fs.readFileSync(selected.backend)),
  };
  assert.equal(image.native, "a183b1956edbe7e7d98f2249858eedf2d8a9790809cc4909db1276e202faaa70");
  const before = seal([path.join(runRoot, "project"), provider], null);
  write(path.join(directory, "inputs-before.json"), before);
  const pageBytes = Number(execFileSync("getconf", ["PAGESIZE"], { encoding: "utf8" }));
  assert(Number.isSafeInteger(pageBytes) && pageBytes > 0);
  const timeFile = path.join(directory, "gnu-time.txt");
  const argv = ["-v", "-o", timeFile, selected.command.executable, ...selected.command.args];
  const environment: NodeJS.ProcessEnv = { ...process.env, ...selected.command.env };
  assert.equal(environment.VIZE_OWNED_READ_BUFFER_TRACE, undefined);
  assert(
    !fs.existsSync(path.join(runRoot, "xdg-cache")),
    "application cache namespace is actually absent before spawn",
  );
  const stdout: Buffer[] = [],
    stderr: Buffer[] = [];
  const samples: {
    atNs: string;
    startedNs: string;
    completedNs: string;
    root: ProcessRow | null;
    tree: ProcessRow[];
    unavailable: string[];
  }[] = [];
  let root: ProcessRow | undefined,
    samplingError: string | null = null,
    spawnError: string | null = null;
  const startedNs = process.hrtime.bigint();
  const child = spawn("/usr/bin/time", argv, {
    cwd: corpus.app,
    env: environment,
    stdio: ["ignore", "pipe", "pipe"],
    timeout: 600000,
  });
  child.stdout.on("data", (data: Buffer) => stdout.push(Buffer.from(data)));
  child.stderr.on("data", (data: Buffer) => stderr.push(Buffer.from(data)));
  child.on("error", (error) => {
    spawnError = error.message;
  });
  const sample = () => {
    const start = process.hrtime.bigint();
    try {
      const current = procRows(pageBytes);
      root ??= current.rows.find((row) => row.parent === child.pid && row.executable === binary);
      const tree = root ? selectObservedTree(current.rows, root.pid, root.birthTicks) : [];
      samples.push({
        atNs: (start - startedNs).toString(),
        startedNs: start.toString(),
        completedNs: process.hrtime.bigint().toString(),
        root: tree.find((row) => row.pid === root?.pid) ?? null,
        tree,
        unavailable: current.unavailable,
      });
    } catch (error) {
      samplingError = String(error);
    }
  };
  const interval = setInterval(sample, 50);
  sample();
  const outcome = await new Promise<{ status: number | null; signal: NodeJS.Signals | null }>(
    (resolve) => child.on("close", (status, signal) => resolve({ status, signal })),
  );
  const completedNs = process.hrtime.bigint();
  clearInterval(interval);
  const out = Buffer.concat(stdout),
    err = Buffer.concat(stderr);
  fs.writeFileSync(path.join(directory, "stdout.bin"), out, { flag: "wx" });
  fs.writeFileSync(path.join(directory, "stderr.bin"), err, { flag: "wx" });
  const observed = samples.filter((sample) => sample.root !== null);
  const rootPeak = observed.reduce((n, s) => Math.max(n, s.root!.rssBytes), 0);
  const treePeak = observed.reduce(
    (n, s) =>
      Math.max(
        n,
        s.tree.reduce((m, p) => m + p.rssBytes, 0),
      ),
    0,
  );
  const rss = {
    intervalMs: 50,
    pageBytes,
    samples,
    samplingError,
    observedCliPid: root?.pid ?? null,
    rootPeakBytes: rootPeak,
    concurrentObservedTreePeakBytes: treePeak,
    scope:
      "timestamped /proc RSS windows for actual CLI plus live descendants; excludes GNU time/sampler; sum at a window, never sum per-PID peaks; shared pages counted per process; short-lived/unavailable/orphaned processes may be missed; no PSS/unique-RAM claim",
  };
  write(path.join(directory, "rss.json"), rss);
  const processReceipt = {
    executable: "/usr/bin/time",
    argv,
    cwd: corpus.app,
    cli: selected.command,
    images: image,
    pid: child.pid ?? null,
    cliPid: root?.pid ?? null,
    ...outcome,
    error: spawnError,
    startedNs: startedNs.toString(),
    completedNs: completedNs.toString(),
    wallMs: Number(completedNs - startedNs) / 1e6,
    stdoutSha256: sha(out),
    stderrSha256: sha(err),
    cold: {
      newProcess: true,
      projectRecreated: true,
      xdgInitiallyAbsent: true,
      osPageCache: "uncontrolled/not flushed",
      nativeCacheNamespaceAcceptance: false,
    },
  };
  write(path.join(directory, "process.json"), processReceipt);
  // All raw/status/sample windows precede assertions and post-command input inspection.
  const after = seal([path.join(runRoot, "project"), provider], null);
  write(path.join(directory, "inputs-after.json"), after);
  assert.deepEqual(after, before, "whole source/provider inputs changed");
  assert.equal(sha(fs.readFileSync(binary)), image.cli);
  assert.equal(sha(fs.readFileSync(selected.backend)), image.native);
  assert.equal(spawnError, null);
  assert.equal(samplingError, null);
  assert.equal(outcome.signal, null);
  assert.equal(outcome.status, 0);
  assert(observed.length > 0, "no actual CLI RSS window observed");
  verifyColdDiagnostics(out.toString("utf8"), "cli", files, false);
  const time = fs.readFileSync(timeFile, "utf8");
  const report = {
    ...processReceipt,
    peakRssBytes: linuxPeakRss(time),
    cpu: linuxCommandCpu(time),
    rootPeakBytes: rootPeak,
    concurrentObservedTreePeakBytes: treePeak,
    corpusSha256: corpus.corpusSha256,
    rssScope: rss.scope,
  };
  write(path.join(directory, "measurement.json"), report);
  return { report, stdout: out, stderr: err };
}
