import assert from "node:assert/strict";
import { createHash, type BinaryLike } from "node:crypto";

export interface NativeFrame {
  offset: number;
  bytes: number;
  packet: any;
}
interface SyscallRow {
  tid: number;
  at: string;
  completedAt: string;
  call: string;
  duration: string | null;
  interval: { startNanoseconds: string; endNanoseconds: string };
}
interface PendingSyscall {
  tid: number;
  at: string;
  call: string;
  raw: string;
}
export interface ExecAttempt {
  tid: number;
  path: string;
  argv: string[];
  at: string;
  result: number;
  call: string;
}
interface ProcessExit {
  code: number | null;
  signal: string | null;
}
interface TerminalUnknownSyscall extends PendingSyscall {
  process: number;
  classification: string;
  exitGroup: SyscallRow;
  sameCodeThreadTerminals: number[];
  syscallBytesCaptured: boolean;
}
interface IoEvent extends SyscallRow {
  operation: string;
  fd: number;
  count: number | null;
  returnKind?: string;
  restart?: string;
  bytes?: Buffer;
}
export interface TraceOwner extends ExecAttempt {
  process: number;
  rows: SyscallRow[];
  exit: ProcessExit | null;
  terminalUnknownSyscalls?: TerminalUnknownSyscall[];
  events?: IoEvent[];
}
export interface StraceCapture {
  owners: TraceOwner[];
  attempts: ExecAttempt[];
  syscallsComplete: boolean;
  terminalUnknownSyscalls: TerminalUnknownSyscall[];
}

export const sha = (bytes: BinaryLike) => createHash("sha256").update(bytes).digest("hex");
const encoded = '"(?:\\\\x[0-9a-fA-F]{2})*"';
export function decodeStraceString(value: string): Buffer {
  assert.match(value, new RegExp(`^${encoded}$`, "u"), "complete hex-encoded syscall string");
  return Buffer.from(
    [...value.matchAll(/\\x([0-9a-f]{2})/giu)].map((match) => Number.parseInt(match[1], 16)),
  );
}
export function decodeNativeFrames(bytes: Buffer): NativeFrame[] {
  let offset = 0;
  const frames: NativeFrame[] = [];
  while (offset < bytes.length) {
    const boundary = bytes.indexOf("\r\n\r\n", offset);
    assert.ok(boundary >= 0, "complete native frame header");
    const headers = bytes.subarray(offset, boundary).toString("ascii").split("\r\n");
    const lengths = headers.filter((header) => /^content-length:/iu.test(header));
    assert.equal(lengths.length, 1, "one native Content-Length");
    assert.match(lengths[0], /^Content-Length: [0-9]+$/iu);
    const length = Number(lengths[0].slice(lengths[0].indexOf(":") + 1));
    assert.ok(
      Number.isSafeInteger(length) && length <= 16 * 1024 * 1024,
      "existing 16MiB frame bound",
    );
    const end = boundary + 4 + length;
    assert.ok(end <= bytes.length, "complete native frame body");
    const body = bytes.subarray(boundary + 4, end);
    const text = body.toString("utf8");
    assert.deepEqual(Buffer.from(text), body, "lossless native UTF8");
    frames.push({ offset, bytes: end - offset, packet: JSON.parse(text) });
    offset = end;
  }
  return frames;
}

const clock = (value: string) => {
  const match = /^(\d+)\.(\d{1,9})$/u.exec(value);
  assert.ok(match, "decimal syscall timestamp/duration preserves nanosecond precision");
  return {
    nanoseconds: BigInt(match[1]) * 1_000_000_000n + BigInt(match[2].padEnd(9, "0")),
    quantum: 10n ** BigInt(9 - match[2].length),
  };
};
type Timed = { interval: { startNanoseconds: string } };
const compareRows = (a: Timed, b: Timed) => {
  const left = BigInt(a.interval.startNanoseconds),
    right = BigInt(b.interval.startNanoseconds);
  return left < right ? -1 : left > right ? 1 : 0;
};
function syscallRow(tid: number, at: string, completedAt: string, call: string): SyscallRow {
  const start = clock(at),
    resumed = clock(completedAt);
  const duration = / <(\d+\.\d+)>$/u.exec(call)?.[1];
  const elapsed = duration ? clock(duration) : { nanoseconds: 0n, quantum: 0n };
  const end = start.nanoseconds + elapsed.nanoseconds;
  // Printed timestamps/durations have finite precision. The upper endpoint is
  // deliberately conservative; equal/overlapping cross-thread intervals refuse.
  const upper =
    (end > resumed.nanoseconds ? end : resumed.nanoseconds) +
    start.quantum +
    resumed.quantum +
    elapsed.quantum;
  return {
    tid,
    at,
    completedAt,
    call,
    duration: duration ?? null,
    interval: {
      startNanoseconds: String(start.nanoseconds),
      endNanoseconds: String(upper),
    },
  };
}

/** Preserve every exec attempt before any protocol-owner selection. */
export function parseStraceCapture(
  traces: readonly { tid: number; text: string }[],
): StraceCapture {
  const rows: SyscallRow[] = [];
  const parents = new Map<number, number>();
  const executables = new Map<number, ExecAttempt>();
  const attempts: ExecAttempt[] = [];
  const unfinished: PendingSyscall[] = [];
  for (const { tid, text } of traces) {
    let pending: PendingSyscall | undefined;
    for (const line of text.split("\n").filter(Boolean)) {
      const match = /^(\d+\.\d+)\s+(.*)$/u.exec(line);
      assert.ok(match, `timestamped complete strace line for ${tid}: ${line}`);
      const at = match[1];
      let call = match[2];
      if (call.endsWith("<unfinished ...>")) {
        assert.equal(pending, undefined, "one unfinished syscall per thread");
        pending = { tid, at, call: call.slice(0, -"<unfinished ...>".length), raw: line };
        continue;
      }
      const resumed = /^<\.\.\. (\w+) resumed>(.*)$/u.exec(call);
      if (resumed) {
        assert.ok(
          pending && pending.call.startsWith(`${resumed[1]}(`),
          "resumed syscall owns its original entry",
        );
        call = pending.call + resumed[2];
        rows.push(syscallRow(tid, pending.at, at, call));
        pending = undefined;
      } else rows.push(syscallRow(tid, at, at, call));
    }
    if (pending) {
      // A second thread can enter exit_group while the group is already exiting.
      assert.match(
        pending.call.trim(),
        /^(?:\?\?\?\(|exit_group\(\d+)$/u,
        "nonterminal unfinished syscalls remain unqualified",
      );
      unfinished.push(pending);
    }
  }
  rows.sort(compareRows);
  for (const row of rows) {
    const clone = /^(?:clone|clone3)\(.*CLONE_THREAD.*\)\s+=\s+(\d+)/u.exec(row.call);
    if (clone) {
      const child = Number(clone[1]);
      assert.equal(parents.has(child), false, "thread identity is unique within one closed trace");
      parents.set(child, row.tid);
    }
    const exec = new RegExp(
      `^execve\\((${encoded}), \\[([^\\]]*)\\], .*\\)\\s+=\\s+(-?\\d+)(?:\\s|$)`,
      "u",
    ).exec(row.call);
    if (exec) {
      const argv = [...exec[2].matchAll(new RegExp(encoded, "gu"))].map((match) =>
        decodeStraceString(match[0]).toString("utf8"),
      );
      const remaining = exec[2].replace(new RegExp(encoded, "gu"), "").replace(/[\s,]/gu, "");
      assert.equal(remaining, "", "complete exec argv without truncation");
      const attempt: ExecAttempt = {
        tid: row.tid,
        path: decodeStraceString(exec[1]).toString("utf8"),
        argv,
        at: row.at,
        result: Number(exec[3]),
        call: row.call,
      };
      attempts.push(attempt);
      if (attempt.result === 0) {
        assert.equal(
          executables.has(row.tid),
          false,
          "no unexplained re-exec of an observed owner",
        );
        executables.set(row.tid, attempt);
      }
    }
    if (/^execve(?:at)?\(/u.test(row.call))
      assert.ok(exec, "every exec attempt has its complete supported path/argv/result recipe");
  }
  const leader = (tid: number) => {
    const seen = new Set<number>();
    let parent = parents.get(tid);
    while (parent !== undefined) {
      assert.equal(seen.has(tid), false, "acyclic thread ownership");
      seen.add(tid);
      tid = parent;
      parent = parents.get(tid);
    }
    return tid;
  };
  const owners = [...executables.values()].map((exec): TraceOwner => ({
    ...exec,
    process: leader(exec.tid),
    rows: [],
    exit: null,
  }));
  const byPid = new Map<number, TraceOwner>(owners.map((owner) => [owner.process, owner]));
  for (const row of rows) {
    const owner = byPid.get(leader(row.tid));
    if (!owner) continue;
    owner.rows.push(row);
    const exited = /^\+\+\+ exited with (\d+) \+\+\+$/u.exec(row.call);
    const killed = /^\+\+\+ killed by (SIG[A-Z0-9]+)(?: \(core dumped\))? \+\+\+$/u.exec(row.call);
    if ((exited || killed) && row.tid === owner.process)
      owner.exit = { code: exited ? Number(exited[1]) : null, signal: killed?.[1] ?? null };
  }
  for (const unknown of unfinished) {
    const process = leader(unknown.tid),
      owner = byPid.get(process);
    assert.ok(owner, "terminal unknown syscall belongs to an actual executed image group");
    const exits = owner.rows.filter((row) => /^exit_group\(\d+\)\s+=\s+\?(?:\s|$)/u.test(row.call));
    assert.equal(exits.length, 1, "one explicit exit_group owns terminal interruption");
    const exit = exits[0],
      exitCode = /^exit_group\((\d+)\)/u.exec(exit.call);
    assert.ok(exitCode);
    const code = Number(exitCode[1]);
    const exitEntry = /^exit_group\((\d+)$/u.exec(unknown.call.trim());
    if (exitEntry)
      assert.equal(
        Number(exitEntry[1]),
        code,
        "an unfinished exit entry repeats the group exit code",
      );
    assert.ok(
      BigInt(exit.interval.endNanoseconds) < clock(unknown.at).nanoseconds,
      "unknown interval must be strictly after the same-group explicit exit_group",
    );
    const members = new Set<number>([
      process,
      ...[...parents.keys()].filter((tid) => leader(tid) === process),
    ]);
    for (const tid of members) {
      const terminals: SyscallRow[] = owner.rows.filter(
        (row) => row.tid === tid && /^\+\+\+ (?:exited|killed)/u.test(row.call),
      );
      assert.equal(
        terminals.length,
        1,
        "every observed same-group thread has its own terminal proof",
      );
      assert.equal(
        terminals[0].call,
        `+++ exited with ${code} +++`,
        "same-code group terminal proof",
      );
    }
    const later = owner.rows.filter(
      (row) =>
        row.tid === unknown.tid && clock(row.at).nanoseconds >= clock(unknown.at).nanoseconds,
    );
    assert.equal(later.length, 1, "no hidden resumed/other syscall after terminal unknown");
    assert.equal(later[0].call, `+++ exited with ${code} +++`);
    (owner.terminalUnknownSyscalls ??= []).push({
      ...unknown,
      process,
      classification: exitEntry
        ? "UNKNOWN_UNCAPTURED_NONRETURNING_EXIT_ENTRY"
        : "UNKNOWN_UNCAPTURED_TERMINAL_SYSCALL",
      exitGroup: exit,
      sameCodeThreadTerminals: [...members],
      syscallBytesCaptured: false,
    });
  }
  return {
    owners,
    attempts,
    syscallsComplete: unfinished.length === 0,
    terminalUnknownSyscalls: owners.flatMap((owner) => owner.terminalUnknownSyscalls ?? []),
  };
}

export function decodeNativeIo<T extends TraceOwner>(owner: T): T & { events: IoEvent[] } {
  const decoded = Object.assign(owner, { events: [] as IoEvent[] });
  for (const row of decoded.rows) {
    const io = /^(read|write|readv|writev)\((\d+),/u.exec(row.call);
    if (io && [0, 1, 2].includes(Number(io[2]))) {
      // Upstream strace documents ERESTARTSYS as a terminated kernel call
      // followed by re-execution, not a successful read of unavailable bytes.
      // https://github.com/strace/strace/blob/master/doc/strace.1.in
      const restart =
        /\)\s+=\s+\? (ERESTARTSYS|ERESTARTNOINTR|ERESTARTNOHAND|ERESTART_RESTARTBLOCK)(?:\s|$)/u.exec(
          row.call,
        );
      if (restart) {
        decoded.events.push({
          ...row,
          operation: io[1],
          fd: Number(io[2]),
          count: null,
          returnKind: "kernel-restart",
          restart: restart[1],
        });
        continue;
      }
      const returned = /\)\s+=\s+(-?\d+)(?:\s|$)/u.exec(row.call);
      assert.ok(returned, "native IO has an explicit return count");
      const count = Number(returned[1]);
      if (count <= 0) {
        decoded.events.push({ ...row, operation: io[1], fd: Number(io[2]), count });
        continue;
      }
      assert.ok(
        io[1] === "read" || io[1] === "write",
        "successful vectored native IO needs an explicit decoder",
      );
      const data = new RegExp(`^(?:read|write)\\(\\d+, (${encoded}), (\\d+)\\)`, "u").exec(
        row.call,
      );
      assert.ok(data, "complete successful native IO buffer, without truncation");
      const bytes = decodeStraceString(data[1]);
      assert.ok(count <= bytes.length && count <= Number(data[2]));
      decoded.events.push({
        ...row,
        operation: io[1],
        fd: Number(io[2]),
        count,
        bytes: bytes.subarray(0, count),
      });
    }
  }
  for (const [operation, fd] of [
    ["read", 0],
    ["write", 1],
    ["write", 2],
  ] as const) {
    const latestEnds = new Map<number, bigint>();
    for (const event of decoded.events
      .filter((event) => event.operation === operation && event.fd === fd && event.bytes)
      .sort(compareRows)) {
      assert.ok(event.duration, "successful native IO has its actual syscall duration");
      const start = BigInt(event.interval.startNanoseconds);
      for (const [tid, end] of latestEnds)
        if (tid !== event.tid)
          assert.ok(
            end < start,
            "cross-thread native byte ordering is unambiguous; ties/overlaps refuse",
          );
      const end = BigInt(event.interval.endNanoseconds),
        previous = latestEnds.get(event.tid);
      latestEnds.set(event.tid, previous === undefined || end > previous ? end : previous);
    }
  }
  return decoded;
}

/** Pure protocol decoder: production admission additionally validates every exec. */
export function parseStraceTexts(traces: readonly { tid: number; text: string }[]) {
  return parseStraceCapture(traces).owners.map(decodeNativeIo);
}
