import { spawnSync } from "node:child_process";
import path from "node:path";

import { repoRoot } from "../../_helpers/realworld-patch.ts";
import {
  captureLinuxProcess,
  classifyExecutableKind,
  linuxProcessBirthTicks,
  readLinuxProcessTable,
  type LinuxProcStat,
  type StableProcessIdentity,
} from "./linux-process-custody.ts";

export {
  linuxProcessBirthTicks,
  parseLinuxProcStat,
  sameLinuxProcessAnchor,
  sameLinuxProcessCustody,
} from "./linux-process-custody.ts";
export type { LinuxProcStat, StableProcessIdentity } from "./linux-process-custody.ts";

export type ProcessRss = { pid: number; parentPid: number; rssKiB: number; executable: string };

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

type StableOptions = { stableIdentity: true; expectedRootBirthTicks?: string | null };

const stableSnapshotAttempts = 3;

export function processRssKiB(processId: number): number | null {
  if (process.platform === "win32") return null;
  const result = spawnSync("ps", ["-o", "rss=", "-p", String(processId)], { encoding: "utf8" });
  if (result.status !== 0) return null;
  const value = Number.parseInt(result.stdout.trim(), 10);
  return Number.isFinite(value) && value > 0 ? value : null;
}

/**
 * Resident-set total and process count for a process and every live descendant.
 * Stable Linux samples use one `/proc` custody window for ancestry, RSS, argv,
 * and executable identity; ambiguous snapshots fail closed after a fixed retry.
 */
export function processTreeRss(rootPid: number): ProcessTreeRssSample | null;
export function processTreeRss(
  rootPid: number,
  options: StableOptions,
): AttributedProcessTreeRssSample | null;
export function processTreeRss(
  rootPid: number,
  options?: StableOptions,
): ProcessTreeRssSample | AttributedProcessTreeRssSample | null {
  if (process.platform === "win32") return null;
  if (!options?.stableIdentity) return samplePsProcessTree(rootPid, false);
  if (process.platform !== "linux") {
    return retryStableProcessTreeSample(() => samplePsProcessTree(rootPid, true));
  }
  const expectedRootBirthTicks =
    options.expectedRootBirthTicks === undefined
      ? linuxProcessBirthTicks(rootPid)
      : options.expectedRootBirthTicks;
  if (expectedRootBirthTicks == null) return null;
  return retryStableProcessTreeSample(() =>
    sampleLinuxProcessTree(rootPid, expectedRootBirthTicks),
  );
}

/** Retries a complete stable snapshot without a blocking delay. */
export function retryStableProcessTreeSample<T>(sample: () => T | null): T | null {
  for (let attempt = 0; attempt < stableSnapshotAttempts; attempt += 1) {
    const value = sample();
    if (value != null) return value;
  }
  return null;
}

function sampleLinuxProcessTree(
  rootPid: number,
  expectedRootBirthTicks: string,
): AttributedProcessTreeRssSample | null {
  const processByPid = readLinuxProcessTable();
  const anchors = linuxTreeAnchors(processByPid, rootPid, expectedRootBirthTicks);
  if (anchors == null) return null;
  const members: AttributedProcessRss[] = [];
  for (const anchor of anchors) {
    const captured = captureLinuxProcess(anchor);
    if (captured == null) return null;
    members.push({
      ...captured,
      role: anchor.pid === rootPid ? "root" : "descendant",
    });
  }
  const afterAnchors = linuxTreeAnchors(readLinuxProcessTable(), rootPid, expectedRootBirthTicks);
  if (afterAnchors == null || !sameLinuxTreeAnchors(anchors, afterAnchors)) return null;
  const totalKiB = members.reduce((sum, member) => sum + member.rssKiB, 0);
  return { totalKiB, processes: members.length, members };
}

function linuxTreeAnchors(
  processByPid: Map<number, LinuxProcStat>,
  rootPid: number,
  expectedRootBirthTicks: string,
): LinuxProcStat[] | null {
  const root = processByPid.get(rootPid);
  if (root == null || root.birthTicks !== expectedRootBirthTicks) return null;
  const children = new Map<number, number[]>();
  for (const member of processByPid.values()) {
    const siblings = children.get(member.parentPid);
    if (siblings == null) children.set(member.parentPid, [member.pid]);
    else siblings.push(member.pid);
  }
  const anchors: LinuxProcStat[] = [];
  const stack = [rootPid];
  while (stack.length > 0) {
    const pid = stack.pop()!;
    const anchor = processByPid.get(pid);
    if (anchor == null) return null;
    anchors.push(anchor);
    stack.push(...(children.get(pid) ?? []));
  }
  return anchors;
}

function sameLinuxTreeAnchors(before: LinuxProcStat[], after: LinuxProcStat[]): boolean {
  if (before.length !== after.length) return false;
  const afterByPid = new Map(after.map((member) => [member.pid, member]));
  return before.every((member) => {
    const current = afterByPid.get(member.pid);
    return (
      current != null &&
      member.parentPid === current.parentPid &&
      member.birthTicks === current.birthTicks
    );
  });
}

function samplePsProcessTree(rootPid: number, attributed: false): ProcessTreeRssSample | null;
function samplePsProcessTree(
  rootPid: number,
  attributed: true,
): AttributedProcessTreeRssSample | null;
function samplePsProcessTree(
  rootPid: number,
  attributed: boolean,
): ProcessTreeRssSample | AttributedProcessTreeRssSample | null {
  const result = spawnSync("ps", ["-Ao", "pid=,ppid=,rss=,comm="], { encoding: "utf8" });
  if (result.status !== 0) return null;
  const children = new Map<number, number[]>();
  const processByPid = new Map<number, ProcessRss>();
  for (const line of result.stdout.trim().split("\n")) {
    const match = /^\s*(\d+)\s+(\d+)\s+(\d+)\s+(.*)$/u.exec(line);
    if (match == null) continue;
    const [pid, parentPid, rssKiB] = match.slice(1, 4).map(Number);
    if (!Number.isSafeInteger(pid) || !Number.isSafeInteger(parentPid) || pid === result.pid)
      continue;
    processByPid.set(pid, {
      pid,
      parentPid,
      rssKiB,
      executable: path.basename(match[4]),
    });
    const siblings = children.get(parentPid);
    if (siblings == null) children.set(parentPid, [pid]);
    else siblings.push(pid);
  }
  if (!processByPid.has(rootPid)) return null;
  const members: ProcessRss[] = [];
  const stack = [rootPid];
  while (stack.length > 0) {
    const pid = stack.pop()!;
    const member = processByPid.get(pid);
    if (member != null) members.push(member);
    stack.push(...(children.get(pid) ?? []));
  }
  const totalKiB = members.reduce((sum, member) => sum + member.rssKiB, 0);
  if (!attributed) return { totalKiB, processes: members.length, members };
  return {
    totalKiB,
    processes: members.length,
    members: members.map((member) => ({
      ...member,
      executable: classifyExecutableKind(member.executable),
      role: member.pid === rootPid ? "root" : "descendant",
      identity: null,
    })),
  };
}

/** Validates member sums and Linux `(pid, birthTicks)` continuity across snapshots. */
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
    if (!/^\d+$/u.test(identity.birthTicks)) {
      throw new Error(`process-tree ${rootPid} member ${member.pid} has invalid birth ticks`);
    }
    if (
      !["vize", "node", "other"].includes(identity.executableKind) ||
      member.executable !== identity.executableKind
    ) {
      throw new Error(`process-tree ${rootPid} member ${member.pid} has invalid executable kind`);
    }
    if (!/^[0-9a-f]{64}$/u.test(identity.executableSha256)) {
      throw new Error(`process-tree ${rootPid} member ${member.pid} has invalid executable hash`);
    }
    if (
      !["native-lsp", "native-api", "typescript-lsp", "typescript-api", "other"].includes(
        identity.runtimeRole,
      ) ||
      !Number.isSafeInteger(identity.argvCount) ||
      identity.argvCount <= 0 ||
      !/^[0-9a-f]{64}$/u.test(identity.argvSha256)
    ) {
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

export function gitHead(): string {
  const result = spawnSync("git", ["rev-parse", "HEAD"], { cwd: repoRoot, encoding: "utf8" });
  return result.status === 0 ? result.stdout.trim() : "unknown";
}
