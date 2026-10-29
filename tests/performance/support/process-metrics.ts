import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";

import { repoRoot } from "../../_helpers/realworld-patch.ts";

export type ProcessRss = { pid: number; parentPid: number; rssKiB: number; executable: string };

export type StableProcessIdentity = {
  birthTicks: string;
  resolvedExecutable: string;
  executableSha256: string;
  argv: string[];
  argvSha256: string;
};

export type AttributedProcessRss = ProcessRss & {
  role: "root" | "descendant";
  /** Linux `/proc` identity. Other POSIX hosts retain RSS attribution without it. */
  identity: StableProcessIdentity | null;
};

export type ProcessTreeRssSample = {
  totalKiB: number;
  processes: number;
  members: ProcessRss[];
};

export type AttributedProcessTreeRssSample = {
  totalKiB: number;
  processes: number;
  members: AttributedProcessRss[];
};

export type PeakProcessTreeRssSample = AttributedProcessTreeRssSample & {
  label: string;
  memberRssSumKiB: number;
};

const executableHashes = new Map<string, string>();

export function processRssKiB(processId: number): number | null {
  if (process.platform === "win32") return null;
  const result = spawnSync("ps", ["-o", "rss=", "-p", String(processId)], { encoding: "utf8" });
  if (result.status !== 0) return null;
  const value = Number.parseInt(result.stdout.trim(), 10);
  return Number.isFinite(value) && value > 0 ? value : null;
}

/**
 * Resident-set total and process count for a process and every live descendant,
 * so leaked Corsa/tsgo worker sessions show up even when the `vize lsp` parent
 * remains within its own RSS budget. Unavailable on Windows, where CI relies on
 * the Linux vue-parity lane for process-tree enforcement.
 */
export function processTreeRss(rootPid: number): ProcessTreeRssSample | null;
export function processTreeRss(
  rootPid: number,
  options: { stableIdentity: true },
): AttributedProcessTreeRssSample | null;
export function processTreeRss(
  rootPid: number,
  options?: { stableIdentity?: boolean },
): ProcessTreeRssSample | AttributedProcessTreeRssSample | null {
  if (process.platform === "win32") return null;
  const result = spawnSync("ps", ["-Ao", "pid=,ppid=,rss=,comm="], { encoding: "utf8" });
  if (result.status !== 0) return null;
  const children = new Map<number, number[]>();
  const processByPid = new Map<number, ProcessRss>();
  for (const line of result.stdout.trim().split("\n")) {
    const match = /^\s*(\d+)\s+(\d+)\s+(\d+)\s+(.*)$/.exec(line);
    if (!match) continue;
    const [pid, ppid, rss] = match.slice(1, 4).map(Number);
    if (!Number.isSafeInteger(pid) || !Number.isSafeInteger(ppid)) continue;
    // When the measured root is this harness process, `ps` briefly appears as
    // its child. It is measurement machinery, not a product-tree member.
    if (options?.stableIdentity && pid === result.pid) continue;
    processByPid.set(pid, { pid, parentPid: ppid, rssKiB: rss, executable: match[4] });
    const siblings = children.get(ppid);
    if (siblings == null) {
      children.set(ppid, [pid]);
    } else {
      siblings.push(pid);
    }
  }
  if (!processByPid.has(rootPid)) return null;
  let totalKiB = 0;
  let processes = 0;
  const members: ProcessRss[] = [];
  const stack = [rootPid];
  while (stack.length > 0) {
    const pid = stack.pop()!;
    const member = processByPid.get(pid);
    if (member) {
      totalKiB += member.rssKiB;
      processes += 1;
      members.push(member);
    }
    stack.push(...(children.get(pid) ?? []));
  }
  if (!options?.stableIdentity) return { totalKiB, processes, members };
  const attributedMembers = members.map((member) => ({
    ...member,
    role: member.pid === rootPid ? ("root" as const) : ("descendant" as const),
    identity: process.platform === "linux" ? linuxProcessIdentity(member) : null,
  }));
  if (process.platform === "linux" && attributedMembers.some((member) => member.identity == null)) {
    // A member exited, was reparented, or reused its PID during custody. Drop
    // the ambiguous sample instead of attributing its RSS to the wrong owner.
    return null;
  }
  return { totalKiB, processes, members: attributedMembers };
}

/**
 * Validates one attributed snapshot and records Linux `(pid, birthTicks)`
 * identities across snapshots. A reused PID makes the run ambiguous and is a
 * hard failure rather than a silently relabelled process.
 */
export function assertStableProcessTreeSample(
  rootPid: number,
  sample: AttributedProcessTreeRssSample,
  observedBirthTicks: Map<number, string>,
): void {
  if (sample.processes !== sample.members.length) {
    throw new Error(
      `process-tree ${rootPid} reports ${sample.processes} processes but retains ` +
        `${sample.members.length} members`,
    );
  }
  const memberSum = sample.members.reduce((sum, member) => sum + member.rssKiB, 0);
  if (sample.totalKiB !== memberSum) {
    throw new Error(
      `process-tree ${rootPid} reports ${sample.totalKiB} KiB but its members sum to ` +
        `${memberSum} KiB`,
    );
  }
  const membersByPid = new Map(sample.members.map((member) => [member.pid, member]));
  if (membersByPid.size !== sample.members.length) {
    throw new Error(`process-tree ${rootPid} contains a duplicate PID`);
  }
  const roots = sample.members.filter((member) => member.role === "root");
  if (roots.length !== 1 || roots[0].pid !== rootPid) {
    throw new Error(`process-tree ${rootPid} must retain exactly one matching root identity`);
  }
  const nextBirthTicks = new Map<number, string>();
  for (const member of sample.members) {
    if (member.pid !== rootPid && !membersByPid.has(member.parentPid)) {
      throw new Error(
        `process-tree ${rootPid} member ${member.pid} lost parent ${member.parentPid}`,
      );
    }
    const identity = member.identity;
    if (identity == null) {
      if (process.platform === "linux") {
        throw new Error(`process-tree ${rootPid} member ${member.pid} lacks Linux identity`);
      }
      continue;
    }
    if (!/^\d+$/.test(identity.birthTicks)) {
      throw new Error(`process-tree ${rootPid} member ${member.pid} has invalid birth ticks`);
    }
    if (!/^[/]/.test(identity.resolvedExecutable)) {
      throw new Error(`process-tree ${rootPid} member ${member.pid} has unresolved executable`);
    }
    if (!/^[0-9a-f]{64}$/.test(identity.executableSha256)) {
      throw new Error(`process-tree ${rootPid} member ${member.pid} has invalid executable hash`);
    }
    if (identity.argv.length === 0 || !/^[0-9a-f]{64}$/.test(identity.argvSha256)) {
      throw new Error(`process-tree ${rootPid} member ${member.pid} has invalid argv identity`);
    }
    const previous = observedBirthTicks.get(member.pid);
    if (previous != null && previous !== identity.birthTicks) {
      throw new Error(
        `process-tree ${rootPid} PID ${member.pid} was reused: birth ticks changed from ` +
          `${previous} to ${identity.birthTicks}`,
      );
    }
    nextBirthTicks.set(member.pid, identity.birthTicks);
  }
  for (const [pid, birthTicks] of nextBirthTicks) observedBirthTicks.set(pid, birthTicks);
}

function linuxProcessIdentity(member: ProcessRss): StableProcessIdentity | null {
  try {
    const before = procIdentity(member.pid);
    if (before.parentPid !== member.parentPid) return null;
    const executableLink = `/proc/${member.pid}/exe`;
    const resolvedExecutable = fs.realpathSync(executableLink);
    const executableStat = fs.statSync(executableLink);
    const executableKey = [
      resolvedExecutable,
      executableStat.dev,
      executableStat.ino,
      executableStat.size,
      executableStat.mtimeMs,
      executableStat.ctimeMs,
    ].join(":");
    let executableSha256 = executableHashes.get(executableKey);
    if (executableSha256 == null) {
      executableSha256 = createHash("sha256").update(fs.readFileSync(executableLink)).digest("hex");
      executableHashes.set(executableKey, executableSha256);
    }
    const argvBytes = fs.readFileSync(`/proc/${member.pid}/cmdline`);
    const argv = argvBytes
      .toString("utf8")
      .split("\0")
      .filter((entry) => entry !== "");
    if (argv.length === 0) return null;
    const after = procIdentity(member.pid);
    const resolvedExecutableAfter = fs.realpathSync(executableLink);
    const executableStatAfter = fs.statSync(executableLink);
    const argvBytesAfter = fs.readFileSync(`/proc/${member.pid}/cmdline`);
    if (
      after.parentPid !== before.parentPid ||
      after.birthTicks !== before.birthTicks ||
      resolvedExecutableAfter !== resolvedExecutable ||
      executableStatAfter.dev !== executableStat.dev ||
      executableStatAfter.ino !== executableStat.ino ||
      !argvBytesAfter.equals(argvBytes)
    ) {
      return null;
    }
    return {
      birthTicks: before.birthTicks,
      resolvedExecutable,
      executableSha256,
      argv,
      argvSha256: createHash("sha256").update(argvBytes).digest("hex"),
    };
  } catch {
    return null;
  }
}

function procIdentity(pid: number): { parentPid: number; birthTicks: string } {
  const stat = fs.readFileSync(`/proc/${pid}/stat`, "utf8");
  const fields = stat
    .slice(stat.lastIndexOf(") ") + 2)
    .trim()
    .split(/\s+/u);
  if (!/^\d+$/.test(fields[1] ?? "") || !/^\d+$/.test(fields[19] ?? "")) {
    throw new Error(`process ${pid} has an invalid /proc stat identity`);
  }
  return { parentPid: Number(fields[1]), birthTicks: fields[19] };
}

export function gitHead(): string {
  const result = spawnSync("git", ["rev-parse", "HEAD"], { cwd: repoRoot, encoding: "utf8" });
  return result.status === 0 ? result.stdout.trim() : "unknown";
}
