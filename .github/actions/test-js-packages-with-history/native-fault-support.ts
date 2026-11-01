import { execFileSync, spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import { constants, cpus, release, tmpdir } from "node:os";
import path from "node:path";

export const hashMeasurements = { bytes: 0, seconds: 0 };
export type Packet = Record<string, any>;

export function writeJson(filename: string, value: unknown): void {
  fs.mkdirSync(path.dirname(filename), { recursive: true });
  fs.writeFileSync(filename, `${JSON.stringify(value, null, 2)}\n`);
}

export function errorName(error: unknown): string {
  return error instanceof Error ? error.name : typeof error;
}

export function readText(filename: string): string | null {
  try {
    return fs.readFileSync(filename, "utf8").trim();
  } catch {
    return null;
  }
}

export function sha256File(filename: string, deadline = Infinity): string {
  const started = performance.now();
  const digest = createHash("sha256");
  const descriptor = fs.openSync(filename, "r");
  try {
    const block = Buffer.allocUnsafe(1024 * 1024);
    for (let size = fs.readSync(descriptor, block); size; size = fs.readSync(descriptor, block)) {
      if (performance.now() >= deadline) throw new Error("Image observation time limit");
      digest.update(block.subarray(0, size));
      hashMeasurements.bytes += size;
    }
  } finally {
    fs.closeSync(descriptor);
    hashMeasurements.seconds += (performance.now() - started) / 1000;
  }
  return digest.digest("hex");
}

function* lines(filename: string): Generator<Buffer> {
  const descriptor = fs.openSync(filename, "r");
  let pending = Buffer.alloc(0);
  try {
    const block = Buffer.allocUnsafe(64 * 1024);
    for (let size = fs.readSync(descriptor, block); size; size = fs.readSync(descriptor, block)) {
      pending = Buffer.concat([pending, block.subarray(0, size)]);
      let end: number;
      while ((end = pending.indexOf(10)) >= 0) {
        yield pending.subarray(0, end + 1);
        pending = pending.subarray(end + 1);
      }
    }
    if (pending.length) yield pending;
  } finally {
    fs.closeSync(descriptor);
  }
}

export function returnCode(status: number | null, signal: string | null): number {
  const signalNumber = Object.entries(constants.signals).find(([name]) => name === signal)?.[1];
  return status ?? (typeof signalNumber === "number" ? -signalNumber : 1);
}

export function runCapture(argv: string[], gated = false, timeoutMs = 30_000): Packet {
  const result: Packet = { argv };
  const directory = fs.mkdtempSync(path.join(tmpdir(), "native-fault-text-"));
  const paths = ["stdout", "stderr"].map((name) => path.join(directory, name));
  const streams = paths.map((filename) => fs.openSync(filename, "w"));
  try {
    const child = spawnSync(argv[0], argv.slice(1), {
      stdio: ["inherit", streams[0], streams[1]],
      timeout: timeoutMs,
      killSignal: "SIGKILL",
    });
    if (child.error) result.unavailable = errorName(child.error);
    else result.status = returnCode(child.status, child.signal);
    for (const [index, name] of ["stdout", "stderr"].entries()) {
      const chunks: Buffer[] = [];
      let admitted = !gated;
      let publicBytes = 0;
      let retainedBytes = 0;
      for (const line of lines(paths[index])) {
        if (gated && line.equals(Buffer.from("VIZE_NATIVE_FAULT_BEGIN\n"))) {
          admitted = true;
          continue;
        }
        if (gated && line.equals(Buffer.from("VIZE_NATIVE_FAULT_END\n"))) {
          admitted = false;
          continue;
        }
        if (admitted && (!gated || name === "stdout")) {
          publicBytes += line.length;
          const retained = line.subarray(0, Math.max(0, 262_144 - retainedBytes));
          if (retained.length) chunks.push(retained);
          retainedBytes += retained.length;
        }
      }
      result[name] = Buffer.concat(chunks).toString("utf8");
      result[`${name}Authority`] = {
        bytes: fs.statSync(paths[index]).size,
        sha256: sha256File(paths[index]),
        publicBytes,
        selection: gated ? "diagnostic markers; startup/arguments excluded" : "whole stream",
        truncated: publicBytes > retainedBytes,
      };
    }
    return result;
  } finally {
    for (const stream of streams) fs.closeSync(stream);
    fs.rmSync(directory, { recursive: true, force: true });
  }
}

export function sourceSnapshot(physicalGit = false): Packet {
  const gitOptions = physicalGit ? ["--no-replace-objects"] : [];
  const git = runCapture(["git", ...gitOptions, "rev-parse", "HEAD", "HEAD^{tree}"]);
  const listing = execFileSync("git", [...gitOptions, "ls-files", "--stage", "-z"], {
    maxBuffer: Infinity,
  });
  const files = listing
    .toString("utf8")
    .split("\0")
    .filter(Boolean)
    .map((entry) => {
      const separator = entry.indexOf("\t");
      const [mode, blob, stage] = entry.slice(0, separator).split(" ");
      const filename = entry.slice(separator + 1);
      if (stage !== "0") throw new Error("Source has unresolved index entries");
      const item: Packet = { file: filename, mode, blob };
      if (mode === "160000") item.gitlink = blob;
      else {
        try {
          if (mode === "120000") {
            const body = Buffer.from(fs.readlinkSync(filename));
            Object.assign(item, {
              bytes: body.length,
              sha256: createHash("sha256").update(body).digest("hex"),
            });
          } else
            Object.assign(item, {
              bytes: fs.statSync(filename).size,
              sha256: sha256File(filename),
            });
        } catch (error) {
          item.unavailable = errorName(error);
        }
      }
      return item;
    });
  const diff = execFileSync("git", [...gitOptions, "diff", "--binary", "HEAD"], {
    maxBuffer: Infinity,
  });
  return { git, files, diffSha256: createHash("sha256").update(diff).digest("hex") };
}

export function privateGuest(capture = runCapture, read = readText): Packet {
  const vm = capture(["systemd-detect-virt", "--vm"]);
  const container = capture(["systemd-detect-virt", "--container"]);
  const dmi = Object.fromEntries(
    ["sys_vendor", "product_name", "bios_vendor"].map((name) => [
      name,
      read(`/sys/class/dmi/id/${name}`),
    ]),
  );
  const allowed =
    vm.status === 0 &&
    vm.stdout?.trim() === "kvm" &&
    container.status === 1 &&
    Object.values(dmi).some((value) => value?.toLowerCase().includes("firecracker"));
  return {
    privateFirecrackerGuest: allowed,
    vm,
    container,
    dmi,
    bootId: read("/proc/sys/kernel/random/boot_id"),
    cgroups: read("/proc/1/cgroup"),
    kernel: release(),
    cpuCount: cpus().length,
  };
}

const processLimits = {
  packets: 128,
  tasks: 128,
  textBytes: 262_144,
  imageBytes: 536_870_912,
  sampleMilliseconds: 1000,
};
const processGaps = new WeakMap<Set<string>, Packet[]>();

function processText(filename: string, limit = processLimits.textBytes): string {
  const descriptor = fs.openSync(filename, "r");
  try {
    const buffer = Buffer.alloc(limit + 1);
    let bytes = 0;
    for (;;) {
      const size = fs.readSync(descriptor, buffer, bytes, buffer.length - bytes, null);
      bytes += size;
      if (bytes > limit) throw new Error("Process observation text limit");
      if (!size) return buffer.subarray(0, bytes).toString("utf8");
    }
  } finally {
    fs.closeSync(descriptor);
  }
}

function processStat(filename: string): { pid: number; parent: number; start: string } {
  const stat = processText(filename, 16_384);
  const tail = stat
    .slice(stat.lastIndexOf(")") + 2)
    .trim()
    .split(/\s+/);
  const pid = Number(stat.slice(0, stat.indexOf(" ")));
  const parent = Number(tail[1]);
  const start = tail[19];
  if (
    !Number.isSafeInteger(pid) ||
    pid < 1 ||
    !Number.isSafeInteger(parent) ||
    parent < 0 ||
    typeof start !== "string" ||
    !/^\d+$/.test(start)
  )
    throw new Error("Unknown process identity");
  return { pid, parent, start };
}

function processTable(procRoot: string): Map<number, { parent: number; start: string }> {
  const processes = new Map<number, { parent: number; start: string }>();
  for (const directory of fs.readdirSync(procRoot)) {
    if (!/^\d+$/.test(directory)) continue;
    try {
      const stat = processStat(path.join(procRoot, directory, "stat"));
      if (stat.pid === Number(directory)) processes.set(stat.pid, stat);
    } catch {
      // A vanished PID is an observation gap, never absence proof.
    }
  }
  return processes;
}

export function statIdentity(stat: fs.BigIntStats): string {
  return [stat.dev, stat.ino, stat.size, stat.mtimeNs].join(":");
}

function imageIdentity(
  filename: string,
  physical: string,
  retained: Map<string, Packet>,
  deadline: number,
): Packet {
  const stat = fs.statSync(physical, { bigint: true });
  if (stat.size > BigInt(processLimits.imageBytes)) throw new Error("Image observation size limit");
  const identity = statIdentity(stat);
  const key = `${filename}:${identity}`;
  if (!retained.has(key)) {
    const sha256 = sha256File(physical, deadline);
    if (identity !== statIdentity(fs.statSync(physical, { bigint: true })))
      throw new Error("Image changed during hashing");
    retained.set(key, {
      path: filename,
      fileIdentity: identity,
      inode: String(stat.ino),
      sha256,
      bytes: Number(stat.size),
    });
  }
  return retained.get(key)!;
}

function taskSample(directory: string, pid: number, deadline: number): Packet {
  const names = fs
    .readdirSync(`${directory}/task`)
    .filter((name) => /^\d+$/.test(name))
    .sort((left, right) => Number(left) - Number(right));
  const tasks: Packet[] = [];
  const unavailable: number[] = [];
  for (const name of names.slice(0, processLimits.tasks)) {
    if (performance.now() >= deadline) break;
    try {
      const root = `${directory}/task/${name}`;
      const task = processStat(`${root}/stat`);
      const status = processText(`${root}/status`, 16_384);
      if (
        task.pid !== Number(name) ||
        Number(/^Tgid:\s*(\d+)$/m.exec(status)?.[1]) !== pid ||
        Number(/^Pid:\s*(\d+)$/m.exec(status)?.[1]) !== task.pid ||
        processStat(`${root}/stat`).start !== task.start
      )
        throw new Error("Task identity changed");
      tasks.push({ tid: task.pid, tgid: pid, startTicks: task.start });
    } catch {
      unavailable.push(Number(name));
    }
  }
  return { tasks, unavailable, limited: tasks.length + unavailable.length < names.length };
}

export function observeDescendants(
  leader: number,
  seen: Set<string>,
  retainedProviders: Map<string, Packet>,
  output: string,
  procRoot = "/proc",
): void {
  const deadline = performance.now() + processLimits.sampleMilliseconds;
  const processes = processTable(procRoot);
  const descendants = new Set([leader]);
  for (let previousSize = 0; previousSize !== descendants.size;) {
    previousSize = descendants.size;
    for (const [pid, info] of processes) if (descendants.has(info.parent)) descendants.add(pid);
  }
  const gaps = processGaps.get(seen) ?? [];
  processGaps.set(seen, gaps);
  let limited = false;
  let visited = 0;
  for (const pid of descendants) {
    if (
      visited++ >= processLimits.packets ||
      seen.size >= processLimits.packets ||
      performance.now() >= deadline
    ) {
      limited = true;
      break;
    }
    const processInfo = processes.get(pid);
    if (!processInfo) {
      if (gaps.length < processLimits.packets && !gaps.some((gap) => gap.pid === pid))
        gaps.push({ pid, identity: "UNKNOWN", unavailable: "ProcessStatUnavailable" });
      continue;
    }
    const directory = path.join(procRoot, String(pid));
    try {
      const executable = fs.readlinkSync(`${directory}/exe`);
      const maps = processText(`${directory}/maps`);
      const taskObservation = taskSample(directory, pid, deadline);
      limited ||= taskObservation.limited;
      let executableIdentity: Packet;
      try {
        executableIdentity = imageIdentity(
          executable,
          `${directory}/exe`,
          retainedProviders,
          deadline,
        );
      } catch (error) {
        executableIdentity = {
          path: executable,
          identity: "UNKNOWN",
          unavailable: errorName(error),
        };
      }
      const mapped = new Map<string, string>();
      for (const line of maps.split("\n")) {
        const fields = /^\S+\s+(\S+)\s+\S+\s+\S+\s+(\d+)\s+(.+)$/.exec(line);
        if (fields?.[1].includes("x") && fields[3].startsWith("/"))
          mapped.set(fields[3], fields[2]);
      }
      const sorted = [...mapped].sort(([left], [right]) => left.localeCompare(right));
      const providers = sorted.map(([filename, inode]) => {
        const mappedInode =
          BigInt(inode) <= BigInt(Number.MAX_SAFE_INTEGER) ? Number(inode) : inode;
        let item: Packet = { path: filename, mappedInode };
        try {
          const image = imageIdentity(filename, filename, retainedProviders, deadline);
          if (BigInt(image.inode) !== BigInt(inode))
            throw new Error("Mapped provider inode changed");
          item = { ...image, mappedInode };
        } catch (error) {
          item.unavailable = errorName(error);
        }
        return item;
      });
      const identity = JSON.stringify([
        pid,
        processInfo.parent,
        processInfo.start,
        executableIdentity,
        sorted,
        providers,
        taskObservation,
      ]);
      if (seen.has(identity)) continue;
      let parentStartTicks: string | null = null;
      try {
        const parent = processStat(path.join(procRoot, String(processInfo.parent), "stat"));
        if (
          parent.pid === processInfo.parent &&
          parent.start === processes.get(processInfo.parent)?.start
        )
          parentStartTicks = parent.start;
      } catch {
        // The parent may disappear or be reused independently of this process.
      }
      const packet = {
        pid,
        parentPid: processInfo.parent,
        parentStartTicks,
        startTicks: processInfo.start,
        executable,
        executableIdentity,
        observedAt: Date.now() / 1000,
        selection: "partial process/task sample; not a complete process or call census",
        ...taskObservation,
        cwd: fs.readlinkSync(`${directory}/cwd`),
        maps,
        providers,
      };
      const after = processStat(`${directory}/stat`);
      if (
        after.start !== processInfo.start ||
        after.pid !== pid ||
        after.parent !== processInfo.parent ||
        fs.readlinkSync(`${directory}/exe`) !== executable ||
        (executableIdentity.fileIdentity &&
          statIdentity(fs.statSync(`${directory}/exe`, { bigint: true })) !==
            executableIdentity.fileIdentity)
      )
        throw new Error("Process identity changed");
      writeJson(path.join(output, "processes", `${pid}-${seen.size}.json`), packet);
      seen.add(identity);
    } catch (error) {
      // Process exit and /proc permission changes are observation gaps.
      if (gaps.length < processLimits.packets && !gaps.some((gap) => gap.pid === pid))
        gaps.push({ pid, identity: "UNKNOWN", unavailable: errorName(error) });
    }
  }
  writeJson(path.join(output, "process-sampling.json"), {
    selection: "partial process/task sample; unreadable or unobserved actors remain UNKNOWN",
    limits: processLimits,
    limited,
    sampledMapPackets: seen.size,
    gapLimitReached: gaps.length === processLimits.packets,
    gaps,
  });
}
