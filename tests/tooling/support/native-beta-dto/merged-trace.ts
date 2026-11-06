import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { TextDecoder } from "node:util";

const MAX_CAPTURE_BYTES = 1024 * 1024 * 1024;
// `strace -s 16777216 -xx` can expand one captured byte to four ASCII bytes.
const MAX_LINE_BYTES = 80 * 1024 * 1024;
const MAX_LINES = 8_000_000;
const MAX_THREADS = 65_536;

export type MergedTraceAuthority = Readonly<{
  file: "syscalls.merged";
  bytes: number;
  sha256: string;
}>;

export type MergedTracePoint = Readonly<{
  captureSha256: string;
  line: number;
}>;

type DerivedThread = {
  tid: number;
  file: string;
  lineMapFile: string;
  descriptor: number;
  lineMapDescriptor: number;
  hash: ReturnType<typeof createHash>;
  lineMapHash: ReturnType<typeof createHash>;
  bytes: number;
  lineMapBytes: number;
  lines: number;
  mergedLines: number[];
};

export type MergedTraceReceipt = Readonly<{
  schema: "vize.nativeMergedTrace.v1";
  source: MergedTraceAuthority & { lines: number };
  limits: {
    maxCaptureBytes: number;
    maxLineBytes: number;
    maxLines: number;
    maxThreads: number;
  };
  derived: ReadonlyArray<{
    tid: number;
    file: string;
    bytes: number;
    lines: number;
    sha256: string;
    lineMapFile: string;
    lineMapBytes: number;
    lineMapSha256: string;
  }>;
  claim: "serialized tracer output proves only observed completion before a later observed entry";
}>;

export type DerivedMergedTrace = Readonly<{
  directory: string;
  receipt: MergedTraceReceipt;
  point(tid: number, derivedLine: number): MergedTracePoint;
}>;

function validAuthority(authority: MergedTraceAuthority): void {
  assert.deepEqual(Object.keys(authority).sort(), ["bytes", "file", "sha256"]);
  assert.equal(authority.file, "syscalls.merged");
  assert.ok(Number.isSafeInteger(authority.bytes) && authority.bytes > 0);
  assert.ok(authority.bytes <= MAX_CAPTURE_BYTES, "bounded merged trace required");
  assert.match(authority.sha256, /^[a-f0-9]{64}$/u);
}

/**
 * Split one authenticated `strace -f -o` stream into the legacy per-thread
 * views. The original file is never rewritten. Each derived byte and its
 * merged-line map receives an independent digest before the audit consumes it.
 */
export async function deriveMergedTrace(
  root: string,
  authority: MergedTraceAuthority,
): Promise<DerivedMergedTrace> {
  validAuthority(authority);
  const source = path.join(root, authority.file);
  const stat = fs.statSync(source);
  assert.ok(stat.isFile(), "merged trace must be a regular file");
  assert.equal(stat.size, authority.bytes, "merged trace byte count differs");

  const directory = path.join(root, "native-trace-derived");
  fs.mkdirSync(directory);
  const threads = new Map<number, DerivedThread>();
  const captureHash = createHash("sha256");
  const decoder = new TextDecoder("utf-8", { fatal: true });
  let pending = Buffer.alloc(0);
  let totalBytes = 0;
  let mergedLine = 0;

  const threadFor = (tid: number): DerivedThread => {
    let thread = threads.get(tid);
    if (thread) return thread;
    assert.ok(threads.size < MAX_THREADS, "bounded traced thread count required");
    const file = `syscalls.${tid}`;
    const lineMapFile = `${file}.merged-lines`;
    thread = {
      tid,
      file,
      lineMapFile,
      descriptor: fs.openSync(path.join(directory, file), "wx"),
      lineMapDescriptor: fs.openSync(path.join(directory, lineMapFile), "wx"),
      hash: createHash("sha256"),
      lineMapHash: createHash("sha256"),
      bytes: 0,
      lineMapBytes: 0,
      lines: 0,
      mergedLines: [],
    };
    threads.set(tid, thread);
    return thread;
  };

  const consume = (raw: Buffer): void => {
    mergedLine++;
    assert.ok(mergedLine <= MAX_LINES, "bounded merged trace line count required");
    assert.ok(raw.length > 0 && raw.length <= MAX_LINE_BYTES, "bounded trace record required");
    assert.equal(raw.includes(0x0d), false, "LF-only strace output required");
    const text = decoder.decode(raw);
    const match = /^([1-9][0-9]*) +([0-9]+\.[0-9]{6} .+)$/u.exec(text);
    assert.ok(match, `closed merged trace record required at line ${mergedLine}`);
    const tid = Number(match[1]);
    assert.ok(Number.isSafeInteger(tid));
    const payload = Buffer.from(`${match[2]}\n`);
    const thread = threadFor(tid);
    assert.equal(fs.writeSync(thread.descriptor, payload), payload.length);
    thread.hash.update(payload);
    thread.bytes += payload.length;
    thread.lines++;
    thread.mergedLines.push(mergedLine);
    const lineMap = Buffer.from(`${mergedLine}\n`);
    assert.equal(fs.writeSync(thread.lineMapDescriptor, lineMap), lineMap.length);
    thread.lineMapHash.update(lineMap);
    thread.lineMapBytes += lineMap.length;
  };

  try {
    for await (const value of fs.createReadStream(source, { highWaterMark: 1024 * 1024 })) {
      const chunk = Buffer.from(value);
      totalBytes += chunk.length;
      assert.ok(totalBytes <= MAX_CAPTURE_BYTES, "bounded merged trace required");
      captureHash.update(chunk);
      pending = pending.length === 0 ? chunk : Buffer.concat([pending, chunk]);
      let start = 0;
      for (;;) {
        const end = pending.indexOf(0x0a, start);
        if (end < 0) break;
        consume(pending.subarray(start, end));
        start = end + 1;
      }
      pending = pending.subarray(start);
      assert.ok(pending.length <= MAX_LINE_BYTES, "bounded unterminated trace record required");
    }
  } finally {
    for (const thread of threads.values()) {
      fs.closeSync(thread.descriptor);
      fs.closeSync(thread.lineMapDescriptor);
    }
  }

  assert.equal(pending.length, 0, "newline-terminated merged trace required");
  assert.equal(totalBytes, authority.bytes);
  assert.equal(captureHash.digest("hex"), authority.sha256, "original merged trace digest differs");
  assert.ok(threads.size > 0, "actual merged native syscalls required");

  const rows = [...threads.values()]
    .sort((left, right) => left.tid - right.tid)
    .map((thread) => ({
      tid: thread.tid,
      file: thread.file,
      bytes: thread.bytes,
      lines: thread.lines,
      sha256: thread.hash.digest("hex"),
      lineMapFile: thread.lineMapFile,
      lineMapBytes: thread.lineMapBytes,
      lineMapSha256: thread.lineMapHash.digest("hex"),
    }));
  const receipt: MergedTraceReceipt = Object.freeze({
    schema: "vize.nativeMergedTrace.v1",
    source: { ...authority, lines: mergedLine },
    limits: {
      maxCaptureBytes: MAX_CAPTURE_BYTES,
      maxLineBytes: MAX_LINE_BYTES,
      maxLines: MAX_LINES,
      maxThreads: MAX_THREADS,
    },
    derived: rows,
    claim: "serialized tracer output proves only observed completion before a later observed entry",
  });
  return Object.freeze({
    directory,
    receipt,
    point(tid: number, derivedLine: number): MergedTracePoint {
      const merged = threads.get(tid)?.mergedLines[derivedLine - 1];
      assert.ok(merged !== undefined, "authenticated derived trace line required");
      return Object.freeze({ captureSha256: authority.sha256, line: merged });
    },
  });
}
