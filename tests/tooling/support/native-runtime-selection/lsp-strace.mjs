import assert from "node:assert/strict";
import { classifyTerminalSyscalls } from "./lsp-strace-terminal.mjs";
import { readStraceLines } from "./lsp-strace-lines.mjs";
import fs from "node:fs";
import path from "node:path";
import { createHash } from "node:crypto";

const sha = (bytes) => createHash("sha256").update(bytes).digest("hex");
const encoded = '"(?:\\\\x[0-9a-fA-F]{2})*"';
export function decodeStraceString(value) {
  assert.match(value, new RegExp(`^${encoded}$`, "u"), "complete hex-encoded syscall string");
  return Buffer.from(
    [...value.matchAll(/\\x([0-9a-f]{2})/giu)].map((match) => Number.parseInt(match[1], 16)),
  );
}
export function decodeNativeFrames(bytes) {
  let offset = 0;
  const frames = [];
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

const clock = (value) => {
  const match = /^(\d+)\.(\d{1,9})$/u.exec(value);
  assert.ok(match, "decimal syscall timestamp/duration preserves nanosecond precision");
  return {
    nanoseconds: BigInt(match[1]) * 1_000_000_000n + BigInt(match[2].padEnd(9, "0")),
    quantum: 10n ** BigInt(9 - match[2].length),
  };
};
const compareRows = (a, b) => {
  const left = BigInt(a.interval.startNanoseconds),
    right = BigInt(b.interval.startNanoseconds);
  return left < right ? -1 : left > right ? 1 : 0;
};
function syscallRow(tid, at, completedAt, call) {
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
export function parseStraceCapture(traces) {
  const rows = [];
  const parents = new Map();
  const executables = new Map();
  const attempts = [];
  const unfinished = [];
  const exitEntries = [];
  const readEntries = [];
  for (const { tid, text, lines } of traces) {
    let pending;
    for (const line of lines ?? text.split("\n")) {
      if (line === "") continue;
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
      if (pending.call.trim() === "???(") unfinished.push(pending);
      else if (/^read\(\d+,\s*$/u.test(pending.call)) readEntries.push(pending);
      else {
        assert.match(
          pending.call,
          /^exit_group\(\d+\s*$/u,
          "nonterminal unfinished syscalls remain unqualified",
        );
        exitEntries.push(pending);
      }
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
      const attempt = {
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
  const leader = (tid) => {
    const seen = new Set();
    while (parents.has(tid)) {
      assert.equal(seen.has(tid), false, "acyclic thread ownership");
      seen.add(tid);
      tid = parents.get(tid);
    }
    return tid;
  };
  const owners = [...executables.values()].map((exec) => ({
    ...exec,
    process: leader(exec.tid),
    rows: [],
    exit: null,
  }));
  const byPid = new Map(owners.map((owner) => [owner.process, owner]));
  for (const row of rows) {
    const owner = byPid.get(leader(row.tid));
    if (!owner) continue;
    owner.rows.push(row);
    const exited = /^\+\+\+ exited with (\d+) \+\+\+$/u.exec(row.call);
    const killed = /^\+\+\+ killed by (SIG[A-Z0-9]+)(?: \(core dumped\))? \+\+\+$/u.exec(row.call);
    if ((exited || killed) && row.tid === owner.process)
      owner.exit = { code: exited ? Number(exited[1]) : null, signal: killed?.[1] ?? null };
  }
  const terminal = classifyTerminalSyscalls({
    owners,
    unfinished,
    exitEntries,
    readEntries,
    parents,
    leader,
    clock,
  });
  return {
    owners,
    attempts,
    ...terminal,
  };
}

export function decodeNativeIo(owner) {
  owner.events = [];
  for (const row of owner.rows) {
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
        owner.events.push({
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
        owner.events.push({ ...row, operation: io[1], fd: Number(io[2]), count });
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
      owner.events.push({
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
  ]) {
    const latestEnds = new Map();
    for (const event of owner.events
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
  return owner;
}

/** Pure protocol decoder: production admission additionally validates every exec. */
export function parseStraceTexts(traces) {
  return parseStraceCapture(traces).owners.map(decodeNativeIo);
}

export function validateExecInventory(capture, authority) {
  for (const attempt of capture.attempts)
    assert.equal(attempt.result, 0, "failed execution/discovery attempts remain explicit failures");
  let sourceCount = 0;
  const owners = capture.owners.map((owner) => {
    const physicalPath = fs.realpathSync(owner.path);
    const executable = {
      path: owner.path,
      physicalPath,
      sha256: sha(fs.readFileSync(physicalPath)),
    };
    let role;
    if (owner.path === authority.source.path) {
      assert.equal(executable.sha256, authority.source.sha256);
      assert.deepEqual(owner.argv, [authority.source.path, "lsp", "--stdio"]);
      sourceCount += 1;
      assert.equal(sourceCount, 1, "one original source CLI image");
      role = "source-lsp";
    } else if (owner.path === authority.node.path) {
      assert.equal(physicalPath, authority.node.physicalPath);
      assert.equal(executable.sha256, authority.node.sha256);
      assert.deepEqual(owner.argv.slice(0, 5), [
        "node",
        "--input-type=module",
        "-e",
        authority.configScript,
        authority.configFile,
      ]);
      assert.equal(owner.argv.length, 7, "exact source config observation recipe");
      assert.match(owner.argv[5], authority.configCachePattern);
      assert.equal(owner.argv[6], "--observe-config-inputs");
      role = "source-config-node";
    } else {
      assert.ok(authority.sdk, "negative profile permits no compiler/other helper image");
      assert.equal(
        physicalPath,
        authority.sdk.path,
        "only the selected physical provider may execute",
      );
      assert.equal(executable.sha256, authority.sdk.sha256);
      assert.deepEqual(
        owner.argv,
        [owner.path, "--lsp", "--stdio"],
        "only the exact observed native LSP recipe",
      );
      role = "selected-native-lsp";
    }
    // Selected-provider teardown has its separate exact image-specific law.
    if (role !== "selected-native-lsp")
      assert.deepEqual(
        owner.exit,
        { code: 0, signal: null },
        "every permitted source/config image closes successfully",
      );
    return { ...owner, executable, role };
  });
  assert.equal(sourceCount, 1);
  return owners;
}

export function captureNativeOwners(tracePrefix, directory, authority) {
  const parent = path.dirname(tracePrefix),
    prefix = path.basename(tracePrefix) + ".";
  const traces = fs
    .readdirSync(parent)
    .filter((file) => file.startsWith(prefix))
    .map((file) => {
      const suffix = file.slice(prefix.length);
      assert.match(suffix, /^\d+$/u);
      return { tid: Number(suffix), lines: readStraceLines(path.join(parent, file)) };
    });
  assert.ok(traces.length > 0, "actual strace files are mandatory");
  fs.mkdirSync(directory, { recursive: true });
  const capture = parseStraceCapture(traces);
  const inventory = {
    attempts: capture.attempts,
    executions: capture.owners.map(({ rows: _rows, ...owner }) => owner),
    syscallsComplete: capture.syscallsComplete,
    terminalUnknownSyscalls: capture.terminalUnknownSyscalls,
    terminalExitEntries: capture.terminalExitEntries,
    terminalReadEntries: capture.terminalReadEntries,
  };
  fs.writeFileSync(
    path.join(directory, "exec-inventory.json"),
    JSON.stringify(inventory, null, 2) + "\n",
  );
  const allOwners = validateExecInventory(capture, authority);
  fs.writeFileSync(
    path.join(directory, "qualified-exec-inventory.json"),
    JSON.stringify(
      allOwners.map(({ rows: _rows, ...owner }) => owner),
      null,
      2,
    ) + "\n",
  );
  const owners = allOwners
    .filter((owner) => owner.role === "selected-native-lsp")
    .map(decodeNativeIo);
  const nativeOwners = owners.map((owner) => {
    assert.ok(owner.exit, "each native process has its own actual terminal status");
    const physicalPath = fs.realpathSync(owner.path);
    const streams = {};
    for (const [name, operation, fd] of [
      ["client", "read", 0],
      ["server", "write", 1],
      ["stderr", "write", 2],
    ]) {
      const bytes = Buffer.concat(
        owner.events
          .filter((event) => event.operation === operation && event.fd === fd && event.bytes)
          .map((event) => event.bytes),
      );
      const file = path.join(directory, `${owner.process}.${name}.bin`);
      fs.writeFileSync(file, bytes);
      const frames = name === "stderr" ? null : decodeNativeFrames(bytes);
      streams[name] = { path: file, bytes: bytes.length, sha256: sha(bytes), frames };
    }
    const observation = {
      process: owner.process,
      executable: { path: owner.path, physicalPath, sha256: sha(fs.readFileSync(physicalPath)) },
      argv: owner.argv,
      exit: owner.exit,
      streams,
      ioFailures: owner.events.filter((event) => event.count < 0),
      ioRestarts: owner.events.filter((event) => event.returnKind === "kernel-restart"),
      syscallsComplete:
        !owner.terminalUnknownSyscalls?.length &&
        !owner.terminalExitEntries?.length &&
        !owner.terminalReadEntries?.length,
      terminalUnknownSyscalls: owner.terminalUnknownSyscalls ?? [],
      terminalExitEntries: owner.terminalExitEntries ?? [],
      terminalReadEntries: owner.terminalReadEntries ?? [],
    };
    fs.writeFileSync(
      path.join(directory, `${owner.process}.json`),
      JSON.stringify(observation, null, 2) + "\n",
    );
    return observation;
  });
  return {
    nativeOwners,
    syscallsComplete: capture.syscallsComplete,
    terminalUnknownSyscalls: capture.terminalUnknownSyscalls,
    terminalExitEntries: capture.terminalExitEntries,
    terminalReadEntries: capture.terminalReadEntries,
    execInventory: path.join(directory, "exec-inventory.json"),
    qualifiedExecInventory: path.join(directory, "qualified-exec-inventory.json"),
  };
}
