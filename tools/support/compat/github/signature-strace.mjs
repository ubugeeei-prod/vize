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
    const io = /^(read|write|readv|writev)\((\d+)(<[^>]*>)?,\s*(.*)\)\s+=\s+(\d+)(?:\s|$)/.exec(
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

export function collectPipes(records) {
  const pipes = new Map();
  for (const record of records.filter((item) => item.io)) {
    const match = /^<pipe:\[(\d+)\]>$/.exec(record.io.target);
    if (!match) continue;
    const key = match[1];
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
          events[i - 1].pid === events[i].pid || events[i - 1].end <= events[i].time,
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
    assert.ok(
      argv.includes("--lsp") || (argv.includes("--api") && argv.includes("--async")),
      "unsupported observed native protocol: raw bytes retained, no completeness credit",
    );
  }
  return native;
}
