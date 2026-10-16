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

export function sha256File(filename: string): string {
  const started = performance.now();
  const digest = createHash("sha256");
  const descriptor = fs.openSync(filename, "r");
  try {
    const block = Buffer.allocUnsafe(1024 * 1024);
    for (let size = fs.readSync(descriptor, block); size; size = fs.readSync(descriptor, block)) {
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

export function runCapture(argv: string[], gated = false): Packet {
  const result: Packet = { argv };
  const directory = fs.mkdtempSync(path.join(tmpdir(), "native-fault-text-"));
  const paths = ["stdout", "stderr"].map((name) => path.join(directory, name));
  const streams = paths.map((filename) => fs.openSync(filename, "w"));
  try {
    const child = spawnSync(argv[0], argv.slice(1), {
      stdio: ["inherit", streams[0], streams[1]],
      timeout: 30_000,
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

function processTable(): Map<number, { parent: number; start: string }> {
  const processes = new Map<number, { parent: number; start: string }>();
  for (const directory of fs.readdirSync("/proc")) {
    if (!/^\d+$/.test(directory)) continue;
    try {
      const stat = fs.readFileSync(`/proc/${directory}/stat`, "utf8");
      const tail = stat
        .slice(stat.lastIndexOf(")") + 2)
        .trim()
        .split(/\s+/);
      if (tail.length > 19)
        processes.set(Number(directory), { parent: Number(tail[1]), start: tail[19] });
    } catch {
      // A vanished PID is an observation gap, never absence proof.
    }
  }
  return processes;
}

export function statIdentity(stat: fs.BigIntStats): string {
  return [stat.dev, stat.ino, stat.size, stat.mtimeNs].join(":");
}

export function observeDescendants(
  leader: number,
  seen: Set<string>,
  retainedProviders: Map<string, Packet>,
  output: string,
): void {
  const processes = processTable();
  const descendants = new Set([leader]);
  for (let previousSize = 0; previousSize !== descendants.size;) {
    previousSize = descendants.size;
    for (const [pid, info] of processes) if (descendants.has(info.parent)) descendants.add(pid);
  }
  for (const pid of descendants) {
    const processInfo = processes.get(pid);
    if (!processInfo) continue;
    const directory = `/proc/${pid}`;
    try {
      const executable = fs.readlinkSync(`${directory}/exe`);
      if (path.basename(executable) !== "node" && pid !== leader) continue;
      const maps = fs.readFileSync(`${directory}/maps`, "utf8");
      const mapped = new Map<string, string>();
      for (const line of maps.split("\n")) {
        const fields = /^\S+\s+(\S+)\s+\S+\s+\S+\s+(\d+)\s+(.+)$/.exec(line);
        if (fields?.[1].includes("x") && fields[3].startsWith("/"))
          mapped.set(fields[3], fields[2]);
      }
      const sorted = [...mapped].sort(([left], [right]) => left.localeCompare(right));
      const identity = JSON.stringify([pid, processInfo.start, executable, sorted]);
      if (seen.has(identity)) continue;
      const providers = sorted.map(([filename, inode]) => {
        const mappedInode =
          BigInt(inode) <= BigInt(Number.MAX_SAFE_INTEGER) ? Number(inode) : inode;
        let item: Packet = { path: filename, mappedInode };
        try {
          const stat = fs.statSync(filename, { bigint: true });
          if (stat.ino !== BigInt(inode)) throw new Error("Mapped provider inode changed");
          const key = `${filename}:${statIdentity(stat)}`;
          if (!retainedProviders.has(key)) {
            const sha256 = sha256File(filename);
            if (statIdentity(stat) !== statIdentity(fs.statSync(filename, { bigint: true }))) {
              throw new Error("Provider changed during hashing");
            }
            retainedProviders.set(key, { ...item, sha256, bytes: Number(stat.size) });
          }
          item = retainedProviders.get(key)!;
        } catch (error) {
          item.unavailable = errorName(error);
        }
        return item;
      });
      const packet = {
        pid,
        startTicks: processInfo.start,
        executable,
        cwd: fs.readlinkSync(`${directory}/cwd`),
        maps,
        providers,
      };
      if (
        processTable().get(pid)?.start !== processInfo.start ||
        fs.readlinkSync(`${directory}/exe`) !== executable
      )
        continue;
      writeJson(path.join(output, "processes", `${pid}-${seen.size}.json`), packet);
      seen.add(identity);
    } catch {
      // Process exit and /proc permission changes are observation gaps.
    }
  }
}
