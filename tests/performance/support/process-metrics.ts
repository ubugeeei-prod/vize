import { spawnSync } from "node:child_process";

import { repoRoot } from "../../_helpers/realworld-patch.ts";

export type ProcessRss = { pid: number; parentPid: number; rssKiB: number; executable: string };

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
export function processTreeRss(rootPid: number): {
  totalKiB: number;
  processes: number;
  members: ProcessRss[];
} | null {
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
  return { totalKiB, processes, members };
}

export function gitHead(): string {
  const result = spawnSync("git", ["rev-parse", "HEAD"], { cwd: repoRoot, encoding: "utf8" });
  return result.status === 0 ? result.stdout.trim() : "unknown";
}
