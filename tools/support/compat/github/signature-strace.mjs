import assert from "node:assert/strict";

// -xx emits every string byte as \xNN; do not interpret JSON/locale escapes.
export function quotedBytes(value) {
  assert.match(value, /^(?:\\x[0-9a-f]{2})*$/i, "unbounded hex string required");
  return Buffer.from(value.replaceAll("\\x", ""), "hex");
}
const strings = (value) =>
  [...value.matchAll(/"([^"\n]*)"/g)].map((match) => quotedBytes(match[1]));

export function parseTrace(pid, text) {
  const records = [];
  let pending;
  let terminal = false;
  for (const [index, line] of text.trimEnd().split("\n").entries()) {
    const match = /^(\d+\.\d+)\s+(.*)$/.exec(line);
    assert.ok(match, `unrecognized trace line ${pid}:${index + 1}`);
    const time = Number(match[1]);
    let call = match[2];
    if (call.startsWith("+++ ")) {
      assert.match(call, /^\+\+\+ (exited with \d+|killed by SIG\w+(?: \(core dumped\))?) \+\+\+$/);
      assert.equal(index, text.trimEnd().split("\n").length - 1, "data after process exit");
      terminal = true;
      records.push({ pid, time, end: time, line: index + 1, call });
      continue;
    }
    if (call.startsWith("--- ")) continue;
    const resumed = /^<\.\.\. (\w+) resumed>(.*)$/.exec(call);
    if (resumed) {
      assert.ok(pending && pending.name === resumed[1], "unpaired resumed syscall");
      call = pending.call + resumed[2];
    } else if (call.endsWith("<unfinished ...>")) {
      assert.equal(pending, undefined, "overlapping unfinished syscall");
      pending = { name: call.split("(")[0], call: call.slice(0, -16), time };
      continue;
    }
    const record = { pid, time: pending?.time ?? time, end: time, line: index + 1, call };
    pending = undefined;
    const io =
      /^(read|write|readv|writev)\((\d+)(<[^<>]*(?:<char \d+:\d+>)?>)?,\s*(.*)\)\s+=\s+(\d+)(?:\s|$)/.exec(
        call,
      );
    if (/^(?:read|write|readv|writev)\(/.test(call) && /\)\s+=\s+\d+(?:\s|$)/.test(call)) {
      assert.ok(io, "unsupported positive I/O syntax; completeness refused");
    }
    if (io) {
      assert.ok(!io[4].includes("..."), "abbreviated I/O is incomplete capture");
      const bytes = Buffer.concat(strings(io[4]));
      const count = Number(io[5]);
      assert.ok(bytes.length >= count, "I/O payload shorter than returned byte count");
      if (io[1].startsWith("read"))
        assert.equal(bytes.length, count, "read payload length mismatch");
      record.io = {
        direction: io[1].startsWith("read") ? "read" : "write",
        fd: Number(io[2]),
        target: io[3] ?? "",
        bytes: bytes.subarray(0, count),
      };
    }
    if (/^execve(?:at)?\(/.test(call) && /\)\s+=\s+0$/.test(call))
      record.exec = strings(call).map((bytes) => bytes.toString("utf8"));
    records.push(record);
  }
  assert.equal(pending, undefined, "unfinished syscall at end of trace");
  assert.equal(terminal, true, `process ${pid} lacks terminal lifecycle`);
  return records;
}

export function framedMessages(bytes) {
  const messages = [];
  let offset = 0;
  while (offset < bytes.length) {
    const end = bytes.indexOf("\r\n\r\n", offset);
    assert.ok(end >= offset, "unfinished Content-Length header");
    const header = bytes.subarray(offset, end).toString("ascii");
    const lengths = [...header.matchAll(/^Content-Length:\s*(\d+)\s*$/gim)];
    assert.equal(lengths.length, 1, "one Content-Length required");
    const length = Number(lengths[0][1]);
    assert.ok(length > 0 && end + 4 + length <= bytes.length, "unfinished JSON-RPC body");
    const body = bytes.subarray(end + 4, end + 4 + length);
    const message = JSON.parse(new TextDecoder("utf-8", { fatal: true }).decode(body));
    assert.equal(message.jsonrpc, "2.0");
    messages.push({ offset, length, message });
    offset = end + 4 + length;
  }
  assert.ok(messages.length, "empty RPC stream");
  return messages;
}

// Keep the raw annotation in every event; -xx also hex-encodes FD names.
export function pipeInode(target) {
  const literal = /^<pipe:\[(\d+)\]>$/.exec(target);
  if (literal) return literal[1];
  const escaped = /^<((?:\\x[0-9a-f]{2})+)>$/i.exec(target);
  return escaped
    ? /^pipe:\[(\d+)\]$/.exec(quotedBytes(escaped[1]).toString("latin1"))?.[1]
    : undefined;
}

export function pipeInodes(call) {
  return [...call.matchAll(/<[^<>]*>/g)].map((match) => pipeInode(match[0])).filter(Boolean);
}

export function collectPipes(records) {
  const pipes = new Map();
  for (const record of records.filter((item) => item.io)) {
    const key = pipeInode(record.io.target);
    if (!key) continue;
    if (!pipes.has(key)) pipes.set(key, { read: [], write: [] });
    pipes.get(key)[record.io.direction].push(record);
  }
  for (const pipe of pipes.values()) {
    for (const direction of ["read", "write"]) {
      pipe[direction].sort((a, b) => a.time - b.time);
      const events = pipe[direction];
      for (let i = 1; i < events.length; i++) {
        // Cross-thread overlapping operations have no authenticated byte order.
        assert.ok(
          events[i - 1].pid === events[i].pid || events[i - 1].end < events[i].time,
          "ambiguous concurrent same-pipe byte order",
        );
      }
      pipe[`${direction}Bytes`] = Buffer.concat(events.map((event) => event.io.bytes));
    }
    assert.ok(pipe.writeBytes.length >= pipe.readBytes.length, "unobserved pipe writer");
    assert.ok(
      pipe.writeBytes.subarray(0, pipe.readBytes.length).equals(pipe.readBytes),
      "writer/reader pipe byte mismatch",
    );
  }
  return pipes;
}

export function nativeSessions(records, provider) {
  const native = records.filter((record) => record.exec?.[0] === provider);
  assert.ok(native.length > 0, "no actual pinned native executable observed");
  for (const record of native) {
    const argv = record.exec.slice(1);
    const lsp = argv.includes("--lsp");
    const api = argv.includes("--api") && argv.includes("--async");
    assert.ok(
      lsp !== api && !argv.some((arg) => arg === "--pipe" || arg.startsWith("--pipe=")),
      "unsupported observed native protocol: raw bytes retained, no completeness credit",
    );
  }
  return native;
}

export function processClosure(records, nextestPaths, binary, sessions, selectedTest) {
  const pids = new Set(records.map((record) => record.pid));
  const parents = new Map();
  for (const record of records) {
    const child = /^(?:clone|clone3|fork|vfork)\(.*\)\s+=\s+(\d+)$/.exec(record.call);
    if (!child) continue;
    const pid = Number(child[1]);
    assert.ok(pids.has(pid), "created child has no retained lifecycle trace");
    assert.ok(pid !== record.pid && !parents.has(pid), "reused/conflicting process parent");
    parents.set(pid, { parent: record.pid, time: record.time });
  }
  const roots = [...pids].filter((pid) => !parents.has(pid));
  assert.equal(roots.length, 1, "one observed process root required; orphan trace refused");
  const descends = (event, ancestor) => {
    let pid = event.pid;
    let time = event.time;
    const visited = new Set();
    while (pid !== ancestor.pid) {
      assert.ok(!visited.has(pid), "cyclic process creation closure");
      visited.add(pid);
      const edge = parents.get(pid);
      if (!edge || edge.time > time) return false;
      pid = edge.parent;
      time = edge.time;
    }
    return ancestor.time <= time;
  };
  for (const pid of pids)
    assert.ok(descends({ pid, time: Infinity }, { pid: roots[0], time: -Infinity }));
  const nextest = records.filter((record) => nextestPaths.includes(record.exec?.[0]));
  const executions = records.filter((record) => record.exec?.[0] === binary);
  const listingArgs = [
    ["--list", "--format", "terse"],
    ["--list", "--format", "terse", "--ignored"],
  ];
  const listings = executions.filter((record) => record.exec.slice(2).includes("--list"));
  if (selectedTest || listings.length) {
    assert.equal(listings.length, 2, "exact two archive listing executions required");
    for (const args of listingArgs)
      assert.equal(
        listings.filter(
          (record) => JSON.stringify(record.exec) === JSON.stringify([binary, binary, ...args]),
        ).length,
        1,
        "one exact archive listing argv required",
      );
  }
  const tests = executions.filter((record) => !listings.includes(record));
  assert.equal(nextest.length, 1, "one pinned nextest execution required");
  assert.equal(tests.length, 1, "one original archive test binary execution required");
  if (selectedTest)
    assert.deepEqual(
      tests[0].exec,
      [binary, binary, "--exact", selectedTest, "--nocapture"],
      "original selected test argv required",
    );
  for (const event of [...listings, tests[0]])
    assert.ok(descends(event, nextest[0]), "test does not descend from pinned nextest");
  for (const listing of listings)
    assert.ok(
      records.some(
        (record) => record.pid === listing.pid && record.call === "+++ exited with 0 +++",
      ),
      "archive listing did not exit successfully",
    );
  for (const session of sessions)
    assert.ok(descends(session, tests[0]), "native session does not descend from original test");
  return {
    root: roots[0],
    nextest: nextest[0].pid,
    test: tests[0].pid,
    listings: listings.map(({ pid, exec }) => ({ pid, argv: exec })),
    parents: [...parents],
  };
}
