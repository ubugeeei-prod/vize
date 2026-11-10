import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import type { Packet } from "./native-fault-support.ts";

function errorName(error: unknown): string {
  return error instanceof Error ? error.name : typeof error;
}

function readText(filename: string): string | null {
  try {
    return fs.readFileSync(filename, "utf8").trim();
  } catch {
    return null;
  }
}

function readBoundedBuffer(filename: string, limit = 262_144): Buffer {
  const descriptor = fs.openSync(filename, "r");
  try {
    const buffer = Buffer.alloc(limit + 1);
    let bytes = 0;
    for (;;) {
      const size = fs.readSync(descriptor, buffer, bytes, buffer.length - bytes, null);
      bytes += size;
      if (bytes > limit) throw new Error("Process observation text limit");
      if (!size) return buffer.subarray(0, bytes);
    }
  } finally {
    fs.closeSync(descriptor);
  }
}

function redactedArgv(raw: Buffer): string[] {
  const values = raw.toString("utf8").split("\0").filter(Boolean);
  let redactNext = false;
  return values.map((value) => {
    if (redactNext) {
      redactNext = false;
      return "<redacted>";
    }
    if (["-e", "-p", "--eval", "--print", "--config"].includes(value)) {
      redactNext = true;
      return value;
    }
    const credential = /^(--?)(token|password|secret|authorization|auth|header|otp)(=|$)/iu.exec(
      value,
    );
    if (credential) {
      redactNext = credential[3] !== "=";
      return credential[3] === "=" ? `${credential[1]}${credential[2]}=<redacted>` : value;
    }
    if (value === "-H") {
      redactNext = true;
      return value;
    }
    if (
      /authorization\s*[:=]|_authToken\s*=|password\s*=|token\s*=/iu.test(value) ||
      /^[^=]*(TOKEN|PASSWORD|SECRET|AUTH)[^=]*=/u.test(value)
    )
      return value.includes("=") ? `${value.split("=", 1)[0]}=<redacted>` : "<redacted>";
    const publicValue = value.replace(/(https?:\/\/)[^/@\s]+@/giu, "$1<redacted>@");
    return publicValue.length > 4096
      ? `<long-argument sha256=${createHash("sha256").update(publicValue).digest("hex")}>`
      : publicValue;
  });
}

function processText(filename: string, limit: number): string {
  return readBoundedBuffer(filename, limit).toString("utf8");
}

function parsedMaxProcesses(limits: string): Packet {
  const prefix = "Max processes";
  const line = limits
    .split("\n")
    .find(
      (candidate) => candidate.startsWith(prefix) && /^\s/u.test(candidate[prefix.length] ?? ""),
    );
  const fields = line?.slice(prefix.length).trim().split(/\s+/u) ?? [];
  if (fields.length < 2) return { unavailable: "MaxProcessesRowUnavailable" };
  return {
    soft: fields[0],
    hard: fields[1],
    units: fields.slice(2).join(" ") || null,
  };
}

function cgroupPidsFacts(cgroup: string, cgroupRoot: string): Packet {
  const rows = cgroup
    .split("\n")
    .filter(Boolean)
    .flatMap((row) => {
      const match = /^([^:]*):([^:]*):(.*)$/u.exec(row);
      return match ? [[match[1], match[2], match[3]]] : [];
    });
  const unified = rows.find(([hierarchy, controllers]) => hierarchy === "0" && !controllers);
  const legacy = rows.find(([, controllers]) => controllers?.split(",").includes("pids"));
  const relative = unified?.[2] ?? legacy?.[2];
  if (!relative?.startsWith("/")) return { unavailable: "PidsControllerUnavailable" };
  const controllerRoot = unified ? cgroupRoot : path.join(cgroupRoot, "pids");
  const directory = path.resolve(controllerRoot, `.${relative}`);
  const escaped = path.relative(path.resolve(controllerRoot), directory);
  if (escaped.startsWith("..") || path.isAbsolute(escaped))
    return { unavailable: "CgroupPathOutsideController" };
  const counters = Object.fromEntries(
    ["pids.current", "pids.max", "pids.peak", "pids.events", "pids.events.local"].map((name) => [
      name,
      readText(path.join(directory, name)),
    ]),
  );
  return {
    hierarchy: unified ? "v2" : "v1",
    path: relative,
    ...counters,
  };
}

/**
 * Capture the resource ownership needed to interpret a failed process spawn.
 * Every field is bounded and independently optional: a disappearing process or
 * unavailable controller is evidence of an observation gap, not proof that a
 * limit was absent.
 */
export function processResourceFacts(
  processDirectory: string,
  cgroupRoot = "/sys/fs/cgroup",
): Packet {
  const result: Packet = {};
  try {
    const raw = readBoundedBuffer(path.join(processDirectory, "cmdline"));
    result.argv = redactedArgv(raw);
    result.argvSHA256 = createHash("sha256").update(raw).digest("hex");
    result.argvBytes = raw.length;
  } catch (error) {
    result.argvUnavailable = errorName(error);
  }
  try {
    result.rlimitNproc = parsedMaxProcesses(
      processText(path.join(processDirectory, "limits"), 64 * 1024),
    );
  } catch (error) {
    result.rlimitNproc = { unavailable: errorName(error) };
  }
  try {
    const cgroup = processText(path.join(processDirectory, "cgroup"), 64 * 1024);
    result.cgroupSHA256 = createHash("sha256").update(cgroup).digest("hex");
    result.cgroupPids = cgroupPidsFacts(cgroup, cgroupRoot);
  } catch (error) {
    result.cgroupPids = { unavailable: errorName(error) };
  }
  return result;
}

export function runnerResourceFacts(
  pid = process.pid,
  procRoot = "/proc",
  cgroupRoot = "/sys/fs/cgroup",
): Packet {
  return {
    pid,
    process: processResourceFacts(path.join(procRoot, String(pid)), cgroupRoot),
    kernel: {
      pidMax: readText(path.join(procRoot, "sys/kernel/pid_max")),
      threadsMax: readText(path.join(procRoot, "sys/kernel/threads-max")),
      loadAverage: readText(path.join(procRoot, "loadavg")),
    },
    observedAt: Date.now() / 1000,
  };
}
