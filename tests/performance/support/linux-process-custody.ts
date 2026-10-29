import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import type { BigIntStats } from "node:fs";
import path from "node:path";

export type LinuxExecutableKind = "vize" | "node" | "other";

export type StableProcessIdentity = {
  birthTicks: string;
  executableKind: LinuxExecutableKind;
  executableSha256: string;
  runtimeRole: "native-lsp" | "native-api" | "typescript-lsp" | "typescript-api" | "other";
  argvCount: number;
  argvSha256: string;
};

export type LinuxProcStat = {
  pid: number;
  parentPid: number;
  birthTicks: string;
  rssKiB: number;
};

export type CapturedLinuxProcess = {
  pid: number;
  parentPid: number;
  rssKiB: number;
  executable: LinuxExecutableKind;
  identity: StableProcessIdentity;
};

const executableHashes = new Map<string, string>();
let cachedLinuxPageSizeKiB: number | null = null;

export function linuxProcessBirthTicks(pid: number): string | null {
  if (process.platform !== "linux") return null;
  try {
    return readLinuxProcStat(pid).birthTicks;
  } catch {
    return null;
  }
}

export function parseLinuxProcStat(source: string, pageSizeKiB: number): LinuxProcStat {
  const match = /^(\d+) \((.*)\) ([A-Za-z]) (.*)$/su.exec(source.trim());
  if (match == null || !Number.isSafeInteger(pageSizeKiB) || pageSizeKiB <= 0) {
    throw new Error("invalid Linux /proc stat record");
  }
  const fields = match[4].split(/\s+/u);
  const pid = Number(match[1]);
  const parentPidSource = fields[0] ?? "";
  const rssPagesSource = fields[20] ?? "";
  const parentPid = Number(parentPidSource);
  const birthTicks = fields[18] ?? "";
  const rssPages = Number(rssPagesSource);
  if (
    !Number.isSafeInteger(pid) ||
    pid <= 0 ||
    !Number.isSafeInteger(parentPid) ||
    parentPid < 0 ||
    !/^\d+$/u.test(parentPidSource) ||
    !/^\d+$/u.test(birthTicks) ||
    !Number.isSafeInteger(rssPages) ||
    rssPages < 0 ||
    !/^\d+$/u.test(rssPagesSource)
  ) {
    throw new Error("invalid Linux /proc stat identity or RSS");
  }
  const rssKiB = rssPages * pageSizeKiB;
  if (!Number.isSafeInteger(rssKiB)) throw new Error("Linux /proc RSS exceeds safe range");
  return { pid, parentPid, birthTicks, rssKiB };
}

export function sameLinuxProcessAnchor(expected: LinuxProcStat, actual: LinuxProcStat): boolean {
  return (
    expected.pid === actual.pid &&
    expected.parentPid === actual.parentPid &&
    expected.birthTicks === actual.birthTicks
  );
}

export function sameLinuxProcessCustody(before: LinuxProcStat, after: LinuxProcStat): boolean {
  return sameLinuxProcessAnchor(before, after) && before.rssKiB === after.rssKiB;
}

export function readLinuxProcessTable(): Map<number, LinuxProcStat> {
  const result = new Map<number, LinuxProcStat>();
  for (const entry of fs.readdirSync("/proc")) {
    if (!/^\d+$/u.test(entry)) continue;
    const pid = Number(entry);
    try {
      result.set(pid, readLinuxProcStat(pid));
    } catch {
      // Exits and permission races make individual entries unavailable. A selected
      // descendant race is detected by custody and retries the whole snapshot.
    }
  }
  return result;
}

export function captureLinuxProcess(anchor: LinuxProcStat): CapturedLinuxProcess | null {
  let executableFd: number | null = null;
  try {
    const executableLink = `/proc/${anchor.pid}/exe`;
    executableFd = fs.openSync(executableLink, "r");
    const executableBefore = fs.fstatSync(executableFd, { bigint: true });
    const before = readLinuxProcStat(anchor.pid);
    if (!sameLinuxProcessAnchor(anchor, before)) return null;
    const argvBefore = fs.readFileSync(`/proc/${anchor.pid}/cmdline`);
    const argv = argvBefore
      .toString("utf8")
      .split("\0")
      .filter((entry) => entry !== "");
    if (argv.length === 0) return null;
    const executableSha256 = cachedExecutableSha256(executableFd, executableBefore);
    const executableAfter = fs.fstatSync(executableFd, { bigint: true });
    const argvAfter = fs.readFileSync(`/proc/${anchor.pid}/cmdline`);
    const after = readLinuxProcStat(anchor.pid);
    const linkedExecutableAfter = fs.statSync(executableLink, { bigint: true });
    if (
      !sameLinuxProcessCustody(before, after) ||
      !sameFileStat(executableBefore, executableAfter) ||
      !sameFileStat(executableBefore, linkedExecutableAfter) ||
      !argvAfter.equals(argvBefore)
    ) {
      return null;
    }
    const executableKind = classifyExecutableKind(fs.readlinkSync(`/proc/self/fd/${executableFd}`));
    return {
      pid: anchor.pid,
      parentPid: before.parentPid,
      rssKiB: before.rssKiB,
      executable: executableKind,
      identity: {
        birthTicks: before.birthTicks,
        executableKind,
        executableSha256,
        runtimeRole: classifyRuntimeRole(executableKind, argv),
        argvCount: argv.length,
        argvSha256: createHash("sha256").update(argvBefore).digest("hex"),
      },
    };
  } catch {
    return null;
  } finally {
    if (executableFd != null) fs.closeSync(executableFd);
  }
}

function readLinuxProcStat(pid: number): LinuxProcStat {
  return parseLinuxProcStat(fs.readFileSync(`/proc/${pid}/stat`, "utf8"), linuxPageSizeKiB());
}

function linuxPageSizeKiB(): number {
  if (cachedLinuxPageSizeKiB != null) return cachedLinuxPageSizeKiB;
  const result = spawnSync("getconf", ["PAGESIZE"], { encoding: "utf8" });
  const pageSizeBytes = Number(result.stdout.trim());
  if (
    result.status !== 0 ||
    !Number.isSafeInteger(pageSizeBytes) ||
    pageSizeBytes <= 0 ||
    pageSizeBytes % 1024 !== 0
  ) {
    throw new Error("unable to determine Linux memory page size");
  }
  cachedLinuxPageSizeKiB = pageSizeBytes / 1024;
  return cachedLinuxPageSizeKiB;
}

export function classifyExecutableKind(target: string): LinuxExecutableKind {
  const basename = path.basename(target.replace(/ \(deleted\)$/u, "")).toLowerCase();
  if (basename === "vize" || basename === "vize.exe") return "vize";
  if (basename === "node" || basename === "nodejs" || basename === "node.exe") return "node";
  return "other";
}

export function classifyRuntimeRole(
  executableKind: LinuxExecutableKind,
  argv: readonly string[],
): StableProcessIdentity["runtimeRole"] {
  if (executableKind === "vize" && argv[1] === "lsp") return "native-lsp";
  if (executableKind === "vize" && argv[1] === "api") return "native-api";
  if (executableKind === "other" && argv[1] === "--lsp") return "typescript-lsp";
  if (executableKind === "other" && argv[1] === "--api") return "typescript-api";
  return "other";
}

export function cachedExecutableSha256(
  fileDescriptor: number,
  stat: BigIntStats,
  readFile: (fileDescriptor: number) => Buffer = (descriptor) => fs.readFileSync(descriptor),
): string {
  const executableKey = fileStatKey(stat);
  const cached = executableHashes.get(executableKey);
  if (cached != null) return cached;
  const hash = createHash("sha256").update(readFile(fileDescriptor)).digest("hex");
  executableHashes.set(executableKey, hash);
  return hash;
}

function fileStatKey(stat: BigIntStats): string {
  return [stat.dev, stat.ino, stat.size, stat.mtimeNs, stat.ctimeNs].join(":");
}

function sameFileStat(left: BigIntStats, right: BigIntStats): boolean {
  return fileStatKey(left) === fileStatKey(right);
}
