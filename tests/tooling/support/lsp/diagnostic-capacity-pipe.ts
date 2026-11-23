import type { LspSession } from "./session.ts";
import { waitUntil } from "./diagnostic-capacity.ts";
import type { NativeDiagnosticsGate } from "./responsive-lint-native-gate.ts";
import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { getSystemErrorMessage } from "node:util";
import { readNativeProcess, type NativeLife } from "./responsive-lint-native-gate.ts";

export type SocketDescriptor = {
  pid: number;
  fd: number;
  inode: string;
  birthTicks: string;
  executable: string;
};
export type SocketEvidence = SocketDescriptor & {
  queuedBytes: number;
  localPath: string;
  filesystemInode: string | null;
  raw: string;
};
export type IpcEvidence = {
  source: "Linux AF_UNIX SOCK_DIAG";
  raw: string[];
  processes: NativeLife[];
  descriptors: SocketDescriptor[];
  channels: SocketEvidence[];
};

/** Join all three kernel identities; LISTEN queues count connections, not request bytes. */
export function decodeNativeSockets(
  raw: string[],
  descriptors: SocketDescriptor[],
): SocketEvidence[] {
  const result: SocketEvidence[] = [];
  for (const line of raw) {
    const fields = line.trim().split(/\s+/u);
    const state = fields.indexOf("ESTAB");
    if (state < 0) continue;
    const localPath = fields[state + 3];
    const queue = fields[state + 1];
    // The address column is the kernel socket inode; extended ino: identifies its filesystem path.
    const inode = fields[state + 4];
    const filesystemInode = /\bino:(\d+)/u.exec(line)?.[1] ?? null;
    if (!localPath?.startsWith("/tmp/tsgo-api-") || !queue || !/^\d+$/u.test(queue)) continue;
    for (const match of line.matchAll(/\bpid=(\d+),fd=(\d+)/gu)) {
      const descriptor = descriptors.find(
        (row) => row.pid === Number(match[1]) && row.fd === Number(match[2]) && row.inode === inode,
      );
      if (descriptor)
        result.push({
          ...descriptor,
          queuedBytes: Number(queue),
          localPath,
          filesystemInode,
          raw: line,
        });
    }
  }
  return result;
}

/** Read kernel socket queue lengths without opening, peeking at, or consuming the RPC stream. */
export function nativeIpcEvidence(lives: NativeLife[]): IpcEvidence {
  const descriptors: SocketDescriptor[] = [];
  const owned = new Set(lives.map(({ pid }) => pid));
  const processes = lives.map((expected) => {
    const { life } = readNativeProcess(expected.pid);
    assert.equal(life.birthTicks, expected.birthTicks);
    assert.equal(life.executable, expected.executable);
    assert.equal(life.state, "T");
    const directory = `/proc/${life.pid}/fd`;
    for (const fd of fs.readdirSync(directory)) {
      let link;
      try {
        link = fs.readlinkSync(path.join(directory, fd));
      } catch (error) {
        if ((error as NodeJS.ErrnoException).code === "ENOENT") continue;
        throw error;
      }
      const inode = /^socket:\[(\d+)\]$/u.exec(link)?.[1];
      if (inode)
        descriptors.push({
          pid: life.pid,
          fd: Number(fd),
          inode,
          birthTicks: life.birthTicks,
          executable: life.executable,
        });
    }
    return { ...expected, state: life.state, rawStat: life.rawStat };
  });
  const output = execFileSync(
    "ss",
    ["--unix", "--all", "--numeric", "--processes", "--extended", "--no-header"],
    { encoding: "utf8", timeout: 10_000 },
  );
  const raw = output
    .split(/\r?\n/u)
    .filter((line) =>
      Array.from(line.matchAll(/pid=(\d+),fd=/gu)).some((match) => owned.has(Number(match[1]))),
    );
  return {
    source: "Linux AF_UNIX SOCK_DIAG",
    raw,
    descriptors,
    processes,
    channels: decodeNativeSockets(raw, descriptors),
  };
}
export function nativeIpcGrew(before: IpcEvidence, after: IpcEvidence): boolean {
  return after.channels.some((row) =>
    before.channels.some(
      (old) =>
        old.pid === row.pid &&
        old.birthTicks === row.birthTicks &&
        old.fd === row.fd &&
        old.inode === row.inode &&
        old.executable === row.executable &&
        old.localPath === row.localPath &&
        row.queuedBytes > old.queuedBytes,
    ),
  );
}

/** Retain identities before the parent exits; later orphan cleanup never signals a reused PID. */
export function ownedProcessSnapshot(owner: number): NativeLife[] {
  const all = new Map<number, NativeLife & { parent: number }>();
  for (const entry of fs.readdirSync("/proc")) {
    if (!/^\d+$/u.test(entry)) continue;
    try {
      const { life, parent } = readNativeProcess(Number(entry), true);
      all.set(life.pid, { ...life, parent });
    } catch (error) {
      if (!(error as NodeJS.ErrnoException).code) throw error;
    }
  }
  return [...all.values()].filter((life) => {
    const seen = new Set<number>();
    let pid = life.pid;
    while (all.has(pid) && !seen.has(pid)) {
      if (pid === owner) return true;
      seen.add(pid);
      pid = all.get(pid)!.parent;
    }
    return false;
  });
}

let signalWorker: string | undefined;
function signalWorkerPath(): string {
  if (signalWorker) return signalWorker;
  const temporary = fs.mkdtempSync(path.join(os.tmpdir(), "vize-pidfd-"));
  const binary = path.join(temporary, "inspect");
  const cleanup = () => fs.rmSync(temporary, { recursive: true, force: true });
  try {
    execFileSync(
      "cc",
      [
        "-std=c11",
        "-O2",
        "-Wall",
        "-Werror",
        path.join(import.meta.dirname, "diagnostic-capacity-pidfd.c"),
        "-o",
        binary,
      ],
      { timeout: 10_000 },
    );
  } catch (error) {
    cleanup();
    throw error;
  }
  process.once("exit", cleanup);
  signalWorker = binary;
  return binary;
}

export function signalBoundProcesses(
  lives: NativeLife[],
  action: "SIGCONT" | "SIGTERM" | "SIGKILL" | "observe",
) {
  if (lives.length === 0) return [];
  const args = lives.flatMap(({ pid, birthTicks, executable }) => [
    String(pid),
    birthTicks,
    executable,
  ]);
  const lines = execFileSync(signalWorkerPath(), [action, ...args], {
    encoding: "utf8",
    timeout: 10_000,
  })
    .trimEnd()
    .split("\n");
  assert.equal(lines.length, lives.length);
  return lives.map(({ pid, birthTicks }, index) => {
    const { errorNumber, ...status } = JSON.parse(lines[index]!) as {
      errorNumber?: number;
      matched?: boolean;
      pidfdBound?: boolean;
      state?: string;
      alive?: boolean;
      sent?: boolean;
    };
    return {
      pid,
      birthTicks,
      signal: action,
      alive: false,
      sent: false,
      ...status,
      ...(errorNumber === undefined ? {} : { error: getSystemErrorMessage(-errorNumber) }),
    };
  });
}

export async function cleanupCapacitySession(session: LspSession, gate?: NativeDiagnosticsGate) {
  const report: Record<string, unknown> = {};
  const errors: string[] = [];
  let saved = gate?.lives ?? [];
  try {
    saved = [...ownedProcessSnapshot(session.processId), ...saved];
  } catch (error) {
    errors.push(String(error));
  }
  try {
    report.resume = signalBoundProcesses(saved, "SIGCONT");
  } catch (error) {
    errors.push(String(error));
  }
  try {
    gate?.resume();
  } catch (error) {
    errors.push(String(error));
  }
  try {
    await session.shutdown();
  } catch (error) {
    errors.push(String(error));
  }
  let complete = false;
  try {
    const live = () => signalBoundProcesses(saved, "observe").some((row) => row.alive || row.error);
    if (live()) {
      report.term = signalBoundProcesses(saved, "SIGTERM");
      if (!(await waitUntil(() => !live(), 2_000)))
        report.kill = signalBoundProcesses(saved, "SIGKILL");
    }
    complete = await waitUntil(() => !live(), 2_000);
  } catch (error) {
    errors.push(String(error));
    try {
      await session.kill();
    } catch (killError) {
      errors.push(String(killError));
    }
  }
  try {
    await session.waitForClosed();
  } catch (error) {
    complete = false;
    errors.push(String(error));
  }
  return { complete, errors, saved, ...report };
}
