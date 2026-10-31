import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { verifyNativeRelations } from "./support/native-beta-dto/stable-watcher-boundary.ts";
import {
  parseNativeIo,
  readNativeTrace,
  nativeOwnerTerminalState,
  requireNoUnknownNativeWrites,
  requireSingleTerminalStderrRole,
  requireCompletedBefore,
} from "./support/native-beta-dto/native-io.ts";

test("returned, partial, failed and unknown writes retain distinct byte outcomes", () => {
  const complete = parseNativeIo('1.000001 write(2, "\\x61\\x62", 2) = 2')!;
  assert.equal(complete.returnedBytes, 2);
  assert.equal(complete.outcome, "completed");
  const partial = parseNativeIo('1.000002 write(2, "\\x61\\x62", 2) = 1')!;
  assert.equal(partial.outcome, "completed");
  assert.equal(partial.data.toString(), "ab");
  assert.equal(partial.returnedBytes, 1);
  assert.equal(partial.completedData.toString(), "a");
  const unknown = parseNativeIo('1.000003 write(2, "\\x61\\x62", 2) = ?')!;
  assert.equal(unknown.outcome, "unknown");
  assert.equal(unknown.data.toString(), "ab");
  assert.equal(unknown.requestedBytes, 2);
  assert.equal(unknown.returnedBytes, null);
  assert.equal(unknown.outcome, "unknown");
  const failed = parseNativeIo('1.000004 write(2, "\\x61\\x62", 2) = -1 EPIPE (Broken pipe)')!;
  assert.equal(failed.returnedBytes, -1);
  assert.equal(failed.outcome, "failed");
  assert.throws(() => parseNativeIo('1.000005 write(2, "\\x61", 2) = 2'));
  assert.throws(() => parseNativeIo('1.000005 write(2, "\\x61", 2) = ?'));
  assert.throws(() => parseNativeIo("1.000005 write(2, 0x1234, 2) = ?"));
});

test("vector writes preserve attempted bytes independently of OS partition", () => {
  const value = parseNativeIo(
    '1.000001 writev(2, [{iov_base="\\x61", iov_len=1}, {iov_base="\\x62", iov_len=1}], 2) = ?',
  )!;
  assert.equal(value.outcome, "unknown");
  assert.equal(value.data.toString(), "ab");
  assert.equal(value.requestedBytes, 2);
  assert.equal(value.returnedBytes, null);
  assert.equal(value.syscall, "writev");
  assert.equal(value.iovCount, 2);
  for (const malformed of [
    '1.000001 writev(2, [{iov_base="\\x61", iov_len=1}], 99) = ?',
    '1.000001 writev(2, [{iov_base="\\x61", iov_len=1}, ...], 2) = ?',
    '1.000001 writev(2, [{iov_base="\\x61", iov_len=2}], 1) = ?',
    "1.000001 writev(2, [{iov_base=0x1234, iov_len=1}], 1) = ?",
    '1.000001 writev(2, [{iov_base="\\x61", iov_len=1}, {iov_base="\\x62", iov_len=1}], 1) = ?',
    '1.000001 readv(1, [{iov_base="\\x61", iov_len=2}, {iov_base="\\x62", iov_len=2}], 2) = 2',
    '1.000001 writev(1, [{iov_base="\\x61", iov_len=2}, {iov_base="\\x62", iov_len=2}], 2) = 2',
  ])
    assert.throws(() => parseNativeIo(malformed));
  const partialRead = parseNativeIo(
    '1.000001 readv(0, [{iov_base="\\x61", iov_len=4096}, {iov_base="", iov_len=4096}], 2) = 1',
  )!;
  assert.equal(partialRead.requestedBytes, 8192);
  assert.equal(partialRead.returnedBytes, 1);
  assert.equal(partialRead.data!.toString(), "a");
  assert.equal(partialRead.outcome, "completed");
  assert.equal(partialRead.completedData.toString(), "a");
  const crossing = parseNativeIo(
    '1.000001 readv(0, [{iov_base="\\x61\\x62", iov_len=2}, {iov_base="\\x63", iov_len=9}], 2) = 3',
  )!;
  assert.equal(crossing.outcome, "completed");
  assert.equal(crossing.completedData.toString(), "abc");
  const read = parseNativeIo('1.000001 read(0, "", 4096) = 0')!;
  assert.equal(read.returnedBytes, 0);
  assert.equal(read.requestedBytes, 4096);
});

test("role-level terminal observation accounts for every native unknown write", () => {
  const io = parseNativeIo('1.000003 write(2, "\\x61", 1) = ?')!;
  assert.equal(io.outcome, "unknown");
  const unknown = { ...io, tid: 1, file: "syscalls.1", line: 1 };
  requireSingleTerminalStderrRole([1], [{ nativeOwner: 1, unknownWrites: [unknown] }]);
  assert.throws(() =>
    requireSingleTerminalStderrRole([1, 2], [{ nativeOwner: 1, unknownWrites: [unknown] }]),
  );
  assert.throws(() =>
    requireSingleTerminalStderrRole(
      [1],
      [
        { nativeOwner: 1, unknownWrites: [unknown] },
        { nativeOwner: 2, unknownWrites: [unknown] },
      ],
    ),
  );
  assert.throws(() =>
    requireSingleTerminalStderrRole([2], [{ nativeOwner: 1, unknownWrites: [unknown] }]),
  );
});

test("known failed write and successful retry retain closed frames without unknown delivery", async () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "native-io-retry-unit-"));
  try {
    const request = { jsonrpc: "2.0", id: 1, method: "shutdown" },
      response = { jsonrpc: "2.0", id: 1, result: null };
    const frame = (value: unknown) => {
      const body = Buffer.from(JSON.stringify(value));
      return Buffer.concat([Buffer.from(`Content-Length: ${body.length}\r\n\r\n`), body]);
    };
    const input = frame(request),
      output = frame(response),
      hex = (bytes: Buffer) =>
        [...bytes].map((byte) => `\\x${byte.toString(16).padStart(2, "0")}`).join("");
    fs.writeFileSync(
      path.join(root, "syscalls.101"),
      [
        `1.000001 read(0, "${hex(input)}", ${input.length}) = ${input.length}`,
        `1.000002 write(1, "${hex(output)}", ${output.length}) = -1 EAGAIN (Resource temporarily unavailable)`,
        `1.000003 write(1, "${hex(output)}", ${output.length}) = ${output.length}`,
        '1.000004 read(0, "", 4096) = 0',
        "1.000005 +++ exited with 0 +++",
      ].join("\n"),
    );
    const trace = await readNativeTrace(root, "/sdk/tsc");
    assert.equal(trace.failedWrites.length, 1);
    assert.equal(trace.failedWrites[0]!.returnedBytes, -1);
    assert.equal(trace.unknownWrites.length, 0);
    requireNoUnknownNativeWrites(trace.unknownWrites);
    assert.deepEqual(
      Buffer.concat(trace.records.filter((row) => row.op === "write").map((row) => row.data)),
      output,
    );
    assert.equal(
      verifyNativeRelations([request], [response], {
        source: "unit",
        role: "after",
        sdkVersion: "7.0.2",
        sdkSha256: "unit",
        sdkExecutable: "/sdk/tsc",
        projectRoots: [],
      }),
      null,
    );
    const opaque = parseNativeIo(
      "1.000006 write(1, 0x1234, 4096) = -1 EAGAIN (Resource temporarily unavailable)",
    )!;
    assert.equal(opaque.outcome, "failed");
    assert.equal(opaque.argumentsComplete, false);
    assert.equal(opaque.data, null);
    assert.equal(opaque.requestedBytes, null);
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

test("collector preserves unknown/unfinished writes and thread termination without appending bytes", async () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "native-io-unit-"));
  try {
    fs.writeFileSync(
      path.join(root, "syscalls.101"),
      [
        "1.000001 clone(child_stack=NULL, flags=CLONE_THREAD|CLONE_FILES) = 102",
        "2.000001 exit_group(0) = ?",
        "2.000005 +++ exited with 0 +++",
      ].join("\n"),
    );
    fs.writeFileSync(
      path.join(root, "syscalls.102"),
      [
        '1.000002 write(2, "\\x61", 1) = 1',
        '1.000003 read(0, "", 4096) = 0',
        '2.000002 write(2, "\\x62", 1) = ?',
        '2.000003 write(2, "\\x63", 1 <unfinished ...>',
        "2.000004 +++ exited with 0 +++",
      ].join("\n"),
    );
    const trace = await readNativeTrace(root, "/sdk/tsc");
    assert.deepEqual(
      trace.records.map((row) => row.data.toString()),
      ["a"],
    );
    assert.equal(trace.unknownWrites.length, 2);
    assert.deepEqual(
      trace.unknownWrites.map((row) => row.data.toString()),
      ["b", "c"],
    );
    assert.ok(trace.unknownWrites.every((row) => row.returnedBytes === null));
    assert.equal(trace.unknownWrites[1]!.rawResult, "unfinished");
    const state = nativeOwnerTerminalState(trace, 101);
    assert.deepEqual(state.members, [
      { tid: 101, parent: null },
      { tid: 102, parent: 101 },
    ]);
    assert.equal(state.threadExits.length, 2);
    assert.equal(state.exitGroups[0]!.code, 0);
    assert.equal(trace.eofReads[0]!.tid, 102);
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

test("current qualification rejects unknown stderr even when completed output is empty", () => {
  requireNoUnknownNativeWrites([]);
  const io = parseNativeIo('1.000003 write(2, "\\x61", 1) = ?')!;
  assert.equal(io.outcome, "unknown");
  assert.throws(() =>
    requireNoUnknownNativeWrites([{ ...io, tid: 1, file: "syscalls.1", line: 1 }]),
  );
});

test("zero-capacity scalar and vector reads do not establish EOF", async () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "native-io-zero-unit-"));
  try {
    const lines = [
      '1.000001 read(0, "", 0) = 0',
      "1.000002 readv(0, [], 0) = 0",
      '1.000003 readv(0, [{iov_base="", iov_len=4096}], 1) = 0',
      "1.000004 +++ exited with 0 +++",
    ];
    for (const line of lines.slice(0, 2)) {
      const parsed = parseNativeIo(line)!;
      assert.equal(parsed.outcome, "completed");
      assert.equal(parsed.requestedBytes, 0);
    }
    fs.writeFileSync(path.join(root, "syscalls.101"), lines.join("\n"));
    const trace = await readNativeTrace(root, "/sdk/tsc");
    assert.equal(trace.eofReads.length, 1);
    assert.equal(trace.zeroCapacityReads.length, 2);
    assert.ok(
      trace.zeroCapacityReads.every(
        (row) => row.outcome === "completed" && row.returnedBytes === 0,
      ),
    );
    assert.equal(trace.eofReads[0]!.line, 3);
    assert.equal(trace.eofReads[0]!.completion.returnedAt, null);
    assert.deepEqual(trace.eofReads[0]!.completion.returnedBy, {
      at: "1.000004",
      line: 4,
      basis: "next-same-thread-event",
    });
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

test("resumed EOF uses its actual return and never promotes a blocked earlier write", async () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "native-io-resume-unit-"));
  try {
    fs.writeFileSync(
      path.join(root, "syscalls.101"),
      [
        "1.000001 read(0,  <unfinished ...>",
        '1.000003 <... read resumed>"", 4096) = 0',
        '1.000004 write(2, "\\x61", 1) = 1',
        "1.000007 +++ exited with 0 +++",
      ].join("\n"),
    );
    fs.writeFileSync(
      path.join(root, "syscalls.102"),
      [
        '1.000002 write(2, "\\x62", 1 <unfinished ...>',
        "1.000005 <... write resumed>) = 1",
        "1.000006 +++ exited with 0 +++",
      ].join("\n"),
    );
    const trace = await readNativeTrace(root, "/sdk/tsc"),
      eof = trace.eofReads[0]!;
    assert.equal(eof.at, "1.000001");
    assert.equal(eof.completion.returnedAt, "1.000003");
    assert.equal(eof.startedLine, 1);
    assert.equal(eof.line, 2);
    const blocked = trace.records.find((row) => row.tid === 102)!;
    assert.equal(blocked.at, "1.000002");
    assert.equal(blocked.completion.returnedAt, "1.000005");
    assert.throws(() => requireCompletedBefore(eof, blocked));
    requireCompletedBefore(
      eof,
      trace.records.find((row) => row.tid === 101)!,
    );
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

test("ordinary syscall entry supplies a bracket, never an exact return clock", async () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "native-io-bracket-unit-"));
  try {
    fs.writeFileSync(
      path.join(root, "syscalls.101"),
      [
        '1.000001 read(0, "", 4096) = 0',
        '1.000003 write(2, "\\x61", 1) = 1',
        "1.000005 +++ exited with 0 +++",
      ].join("\n"),
    );
    const trace = await readNativeTrace(root, "/sdk/tsc"),
      eof = trace.eofReads[0]!;
    assert.equal(eof.completion.returnedAt, null);
    assert.equal(eof.completion.returnedBy!.at, "1.000003");
    assert.throws(() => requireCompletedBefore(eof, { at: "1.000002", tid: 102, line: 1 }));
    requireCompletedBefore(eof, { at: "1.000004", tid: 102, line: 1 });
    requireCompletedBefore(eof, trace.records[0]!);
    assert.throws(() => requireCompletedBefore({ at: "1.000000", tid: 101, line: 0 }, eof));
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});
