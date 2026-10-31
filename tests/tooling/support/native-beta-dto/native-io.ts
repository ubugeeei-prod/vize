import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { createInterface } from "node:readline";
import type { Syscall } from "./audit-wire-types.ts";
import { ExecveHandoffs } from "./execve-handoff.ts";

interface NativeIoPayload {
  at: string;
  op: "read" | "write";
  syscall: "read" | "write" | "readv" | "writev";
  fd: number;
  data: Buffer | null;
  requestedBytes: number | null;
  iovCount: number | null;
  iovecs: Array<{ data: Buffer; requestedBytes: number }> | null;
  argumentsComplete: boolean;
  rawArguments: string;
  rawResult: string;
}
export type ParsedNativeIo = NativeIoPayload &
  (
    | {
        outcome: "completed";
        returnedBytes: number;
        data: Buffer;
        completedData: Buffer;
        requestedBytes: number;
        argumentsComplete: true;
      }
    | {
        outcome: "unknown";
        returnedBytes: null;
        data: Buffer;
        requestedBytes: number;
        argumentsComplete: true;
      }
    | { outcome: "failed"; returnedBytes: number }
  );
export type NativeCompletion = {
  returnedAt: string | null;
  returnedBy: {
    at: string;
    line: number;
    basis: "explicit-resume" | "next-same-thread-event";
  } | null;
};
export type NativeTimedObservation = {
  at: string;
  tid: number;
  line: number;
  startedLine?: number;
  completion?: NativeCompletion;
};
export function requireCompletedBefore(
  earlier: NativeTimedObservation,
  later: NativeTimedObservation,
) {
  assert.ok(earlier.completion, "known completed syscall required");
  if (earlier.tid === later.tid && earlier.line < (later.startedLine ?? later.line)) return;
  const bound = earlier.completion?.returnedAt ?? earlier.completion?.returnedBy?.at;
  assert.ok(bound, "observed completion order required");
  const time = (at: string) => {
    assert.match(at, /^[0-9]+\.[0-9]{6}$/u);
    return BigInt(at.replace(".", ""));
  };
  assert.ok(time(bound) < time(later.at), "completion must precede later syscall entry");
}
type NativeIoProvenance = NativeTimedObservation & { file: string };
export type UnknownNativeWrite = Extract<ParsedNativeIo, { outcome: "unknown" }> &
  NativeIoProvenance;
export type FailedNativeWrite = Extract<ParsedNativeIo, { outcome: "failed" }> & NativeIoProvenance;
export function requireNoUnknownNativeWrites(writes: readonly UnknownNativeWrite[]) {
  assert.equal(writes.length, 0, "Unobserved native writes refuse current qualification");
}
export function requireSingleTerminalStderrRole(
  observedOwners: readonly number[],
  states: readonly { nativeOwner: number; unknownWrites: readonly UnknownNativeWrite[] }[],
) {
  assert.equal(observedOwners.length, 1, "one terminal stderr owner per role required");
  const unknown = states.flatMap((state) =>
    state.unknownWrites.map((write) => ({ nativeOwner: state.nativeOwner, write })),
  );
  assert.equal(unknown.length, 1, "exactly one native unknown write per role required");
  assert.equal(unknown[0]!.nativeOwner, observedOwners[0]);
}
export interface NativeThreadExit {
  tid: number;
  at: string;
  code: number;
}
export interface NativeExitGroup {
  tid: number;
  at: string;
  code: number;
  line?: number;
  rawResult?: string | null;
}
export interface NativeUnfinishedOperation {
  tid: number;
  file: string;
  line: number;
  at: string;
  raw: string;
}

const unhex = (value: string) => Buffer.from(value.replaceAll("\\x", ""), "hex");
/** Returned bytes and unobserved attempts are deliberately separate. */
export function parseNativeIo(line: string): ParsedNativeIo | null {
  const failed =
    /^([0-9]+\.[0-9]+) (read|write|readv|writev)\((-?[0-9]+), (.*)\)\s+=\s+(-[0-9]+)(.*)$/u.exec(
      line,
    );
  const opaqueFailure = (): ParsedNativeIo => {
    assert.ok(failed);
    const operation = failed[2]!,
      read = operation.startsWith("read");
    return {
      at: failed[1]!,
      op: read ? "read" : "write",
      syscall: read
        ? operation.endsWith("v")
          ? "readv"
          : "read"
        : operation.endsWith("v")
          ? "writev"
          : "write",
      fd: Number(failed[3]),
      data: null,
      requestedBytes: null,
      iovCount: null,
      iovecs: null,
      argumentsComplete: false,
      rawArguments: failed[4]!,
      rawResult: failed[5]! + failed[6]!,
      outcome: "failed",
      returnedBytes: Number(failed[5]),
    };
  };
  try {
    const call =
      /^([0-9]+\.[0-9]+) (read|write)\(([0-9]+),\s+"((?:\\x[0-9a-f]{2})*)",\s+([0-9]+)\s*\)\s+=\s+(-?[0-9]+|\?)(.*)$/u.exec(
        line,
      );
    const vector =
      /^([0-9]+\.[0-9]+) (readv|writev)\(([0-9]+),\s+\[(.*)\],\s+([0-9]+)\s*\)\s+=\s+(-?[0-9]+|\?)(.*)$/u.exec(
        line,
      );
    if (!call && !vector) {
      if (failed) return opaqueFailure();
      assert.ok(!/^[0-9]+\.[0-9]+ writev?\(/u.test(line), "unsupported native write attempt");
      return null;
    }
    const at = (call ?? vector)![1]!,
      operation = (call ?? vector)![2]!,
      fd = Number((call ?? vector)![3]);
    const encoded = (call ?? vector)![4]!;
    const iovecs: Array<{ data: Buffer; requestedBytes: number }> | null = call ? null : [];
    if (iovecs) {
      let offset = 0;
      while (offset < encoded.length) {
        const entry = /^\s*\{iov_base="((?:\\x[0-9a-f]{2})*)",\s*iov_len=([0-9]+)\}\s*/u.exec(
          encoded.slice(offset),
        );
        assert.ok(entry, "complete captured iovec required");
        const data = unhex(entry[1]!),
          requestedBytes = Number(entry[2]);
        assert.ok(Number.isSafeInteger(requestedBytes) && requestedBytes >= data.length);
        iovecs.push({ data, requestedBytes });
        offset += entry[0].length;
        if (offset === encoded.length) break;
        assert.equal(encoded[offset++], ",", "complete iovec separator required");
        assert.ok(offset < encoded.length, "truncated iovec list");
      }
      assert.equal(
        iovecs.length,
        Number(vector![5]),
        "iovcnt must match the complete captured vector",
      );
    }
    const iovCount = call ? null : Number(vector![5]);
    assert.ok(iovCount === null || Number.isSafeInteger(iovCount));
    const data = call ? unhex(encoded) : Buffer.concat(iovecs!.map((row) => row.data));
    const requestedBytes = call
      ? Number(call[5])
      : iovecs!.reduce((sum, row) => sum + row.requestedBytes, 0);
    const result = (call ?? vector)![6]!,
      rawResult = result + (call ?? vector)![7]!;
    const returnedBytes = result === "?" ? null : Number(result);
    assert.ok(Number.isSafeInteger(requestedBytes) && requestedBytes >= 0);
    let completedData: Buffer | null = null;
    if (returnedBytes !== null && returnedBytes >= 0) {
      assert.ok(
        returnedBytes <= requestedBytes && data.length >= returnedBytes,
        "truncated successful native syscall",
      );
      if (iovecs) {
        let remaining = returnedBytes;
        const prefixes: Buffer[] = [];
        for (const row of iovecs) {
          const needed = Math.min(row.requestedBytes, remaining);
          assert.ok(
            row.data.length >= needed,
            "returned bytes must be captured in their own iovec",
          );
          prefixes.push(row.data.subarray(0, needed));
          remaining -= needed;
        }
        assert.equal(remaining, 0);
        completedData = Buffer.concat(prefixes);
      } else completedData = data.subarray(0, returnedBytes);
    } else if (operation.startsWith("write"))
      assert.equal(data.length, requestedBytes, "whole attempted native write payload required");
    const end = /\)\s+=/u.exec(line)!.index;
    const payload = {
      at,
      op: operation.startsWith("read") ? ("read" as const) : ("write" as const),
      syscall: call
        ? operation === "read"
          ? ("read" as const)
          : ("write" as const)
        : operation === "readv"
          ? ("readv" as const)
          : ("writev" as const),
      fd,
      data,
      requestedBytes,
      iovCount,
      iovecs,
      argumentsComplete: true as const,
      rawArguments: line.slice(line.indexOf("(") + 1, end),
      rawResult,
    };
    if (returnedBytes === null) return { ...payload, returnedBytes, outcome: "unknown" };
    if (returnedBytes < 0) return { ...payload, returnedBytes, outcome: "failed" };
    assert.ok(completedData);
    return { ...payload, returnedBytes, completedData, outcome: "completed" };
  } catch (error) {
    if (failed) return opaqueFailure();
    throw error;
  }
}

export async function readNativeTrace(root: string, executable: string) {
  const handoffs = new ExecveHandoffs(),
    lastLines = new Map<number, number>();
  const files = fs
    .readdirSync(root)
    .filter((file) => /^syscalls\.[0-9]+$/u.test(file))
    .sort();
  assert.ok(files.length, "Actual native syscalls required");
  const threads = new Map<number, number>(),
    exits = new Map<number, number>();
  const threadExits: NativeThreadExit[] = [],
    exitGroups: NativeExitGroup[] = [];
  const leaders: Array<{ pid: number; argvObservation: string; lsp: boolean }> = [];
  const records: Array<Syscall & { startedLine: number; completion: NativeCompletion }> = [],
    unknownWrites: UnknownNativeWrite[] = [],
    failedWrites: FailedNativeWrite[] = [];
  const eofReads: Array<NativeTimedObservation & { fd: number; completion: NativeCompletion }> = [];
  const zeroCapacityReads: Array<
    Extract<ParsedNativeIo, { outcome: "completed" }> & NativeIoProvenance
  > = [];
  const unfinishedOperations: NativeUnfinishedOperation[] = [];
  for (const file of files) {
    const tid = Number(file.split(".").at(-1));
    let pending: { text: string; line: number } | undefined;
    let unbounded: NativeCompletion | undefined;
    let lineNumber = 0;
    for await (const original of createInterface({
      input: fs.createReadStream(path.join(root, file)),
      crlfDelay: Infinity,
    })) {
      lineNumber++;
      const observedAt = /^([0-9]+\.[0-9]+) /u.exec(original)?.[1];
      if (unbounded && observedAt) {
        unbounded.returnedBy = {
          at: observedAt,
          line: lineNumber,
          basis: "next-same-thread-event",
        };
        unbounded = undefined;
      }
      let line = original;
      let startedLine = lineNumber;
      let resumed = false;
      if (handoffs.observe(tid, file, lineNumber, line, pending !== undefined)) continue;
      if (line.includes("<unfinished ...>")) {
        assert.equal(pending, undefined, "Unobserved nested syscall");
        pending = { text: line.replace("<unfinished ...>", ""), line: lineNumber };
        continue;
      }
      if (line.includes("resumed>")) {
        assert.ok(pending, `${file}:${lineNumber}: Resumed syscall has no observed start`);
        startedLine = pending.line;
        resumed = true;
        line = pending.text + line.slice(line.indexOf("resumed>") + "resumed>".length);
        pending = undefined;
      }
      const exit = /^([0-9]+\.[0-9]+) \+\+\+ exited with ([0-9]+) \+\+\+/u.exec(line);
      if (exit) {
        exits.set(tid, Number(exit[2]));
        threadExits.push({ tid, at: exit[1]!, code: Number(exit[2]) });
      }
      const group = /^([0-9]+\.[0-9]+) exit_group\(([0-9]+)\)/u.exec(line);
      if (group)
        exitGroups.push({
          tid,
          at: group[1]!,
          code: Number(group[2]),
          line: lineNumber,
          rawResult: /\)\s+=\s+(.+)$/u.exec(line)?.[1] ?? null,
        });
      if (line.includes("execve(") && line.endsWith("= 0")) {
        const decoded = line.replace(/(?:\\x[0-9a-f]{2})+/gu, (value) =>
          unhex(value).toString("utf8"),
        );
        if (decoded.includes(executable))
          leaders.push({ pid: tid, argvObservation: decoded, lsp: decoded.includes("--lsp") });
      }
      const child = /clone(?:3)?\(.*\)\s+=\s+([0-9]+)$/u.exec(line);
      if (child && line.includes("CLONE_THREAD")) threads.set(Number(child[1]), tid);
      const call = parseNativeIo(line);
      if (!call) continue;
      assert.ok(observedAt);
      const completion: NativeCompletion = {
        returnedAt: resumed ? observedAt : null,
        returnedBy: resumed ? { at: observedAt, line: lineNumber, basis: "explicit-resume" } : null,
      };
      if (call.outcome !== "unknown" && !resumed) unbounded = completion;
      if (call.outcome !== "completed") {
        if (call.op === "write") {
          if (call.outcome === "unknown")
            unknownWrites.push({ ...call, tid, file, line: lineNumber, startedLine });
          else {
            failedWrites.push({ ...call, completion, tid, file, line: lineNumber, startedLine });
          }
        }
        continue;
      }
      if (call.returnedBytes === 0 && call.op === "read" && call.requestedBytes > 0)
        eofReads.push({ tid, fd: call.fd, at: call.at, line: lineNumber, startedLine, completion });
      if (call.op === "read" && call.requestedBytes === 0)
        zeroCapacityReads.push({ ...call, completion, tid, file, line: lineNumber, startedLine });
      if (call.returnedBytes)
        records.push({
          at: call.at,
          completion,
          startedLine,
          op: call.op,
          fd: call.fd,
          tid,
          data: call.completedData,
          file,
          line: lineNumber,
        });
    }
    lastLines.set(tid, lineNumber);
    if (pending) {
      const normalized = pending.text.trimEnd().replace(/\)?$/u, ") = ?");
      const write = /^([0-9]+\.[0-9]+) writev?\(/u.test(pending.text)
        ? parseNativeIo(normalized)
        : null;
      if (write) {
        assert.equal(write.outcome, "unknown");
        unknownWrites.push({ ...write, rawResult: "unfinished", tid, file, line: pending.line });
      } else
        unfinishedOperations.push({
          tid,
          file,
          line: pending.line,
          at: pending.text.split(" ")[0]!,
          raw: pending.text,
        });
    }
  }
  for (const tid of threads.keys())
    assert.ok(files.includes(`syscalls.${tid}`), "whole cloned thread trace required");
  const execveHandoffs = handoffs.resolve(threads, lastLines);
  for (const handoff of execveHandoffs)
    assert.notEqual(
      handoff.executable,
      executable,
      "SDK execve handoff requires separate process-generation authority",
    );
  return {
    files,
    threads,
    exits,
    leaders,
    records,
    eofReads,
    zeroCapacityReads,
    unknownWrites,
    failedWrites,
    threadExits,
    exitGroups,
    unfinishedOperations,
    execveHandoffs,
  };
}

export function nativeThreadOwner(threads: ReadonlyMap<number, number>, tid: number): number {
  const seen = new Set<number>();
  while (threads.has(tid)) {
    assert.ok(!seen.has(tid), "cyclic native thread ownership");
    seen.add(tid);
    tid = threads.get(tid)!;
  }
  return tid;
}
export function nativeOwnerTerminalState(
  trace: Awaited<ReturnType<typeof readNativeTrace>>,
  nativeOwner: number,
) {
  const owned = (tid: number) => nativeThreadOwner(trace.threads, tid) === nativeOwner;
  return {
    nativeOwner,
    members: [
      { tid: nativeOwner, parent: null as number | null },
      ...[...trace.threads].filter(([tid]) => owned(tid)).map(([tid, parent]) => ({ tid, parent })),
    ],
    threadExits: trace.threadExits.filter((row) => owned(row.tid)),
    exitGroups: trace.exitGroups.filter((row) => owned(row.tid)),
    unknownWrites: trace.unknownWrites.filter((row) => owned(row.tid)),
    failedWrites: trace.failedWrites.filter((row) => owned(row.tid)),
    zeroCapacityReads: trace.zeroCapacityReads.filter((row) => owned(row.tid)),
    unfinishedOperations: trace.unfinishedOperations.filter((row) => owned(row.tid)),
  };
}
