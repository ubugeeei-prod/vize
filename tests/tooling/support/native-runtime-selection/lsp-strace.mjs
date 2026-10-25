import assert from "node:assert/strict";
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

/** Parse -ff -ttt -T -xx process/read/write traces; unsupported successful IO refuses. */
export function parseStraceTexts(traces) {
  const rows = [];
  const parents = new Map();
  const executables = new Map();
  for (const { tid, text } of traces) {
    let pending;
    for (const line of text.split("\n").filter(Boolean)) {
      const match = /^(\d+\.\d+)\s+(.*)$/u.exec(line);
      assert.ok(match, `timestamped complete strace line for ${tid}: ${line}`);
      const at = Number(match[1]);
      let call = match[2];
      if (call.endsWith("<unfinished ...>")) {
        assert.equal(pending, undefined, "one unfinished syscall per thread");
        pending = { at, call: call.slice(0, -"<unfinished ...>".length) };
        continue;
      }
      const resumed = /^<\.\.\. (\w+) resumed>(.*)$/u.exec(call);
      if (resumed) {
        assert.ok(
          pending && pending.call.startsWith(`${resumed[1]}(`),
          "resumed syscall owns its original entry",
        );
        call = pending.call + resumed[2];
        rows.push({ tid, at: pending.at, completedAt: at, call });
        pending = undefined;
      } else rows.push({ tid, at, completedAt: at, call });
    }
    assert.equal(pending, undefined, "all traced syscalls completed");
  }
  rows.sort((a, b) => a.at - b.at || a.tid - b.tid);
  for (const row of rows) {
    const clone = /^(?:clone|clone3)\(.*CLONE_THREAD.*\)\s+=\s+(\d+)/u.exec(row.call);
    if (clone) {
      const child = Number(clone[1]);
      assert.equal(parents.has(child), false, "thread identity is unique within one closed trace");
      parents.set(child, row.tid);
    }
    const exec = new RegExp(
      `^execve\\((${encoded}), \\[([^\\]]*)\\], .*\\)\\s+=\\s+0(?:\\s|$)`,
      "u",
    ).exec(row.call);
    if (exec) {
      const argv = [...exec[2].matchAll(new RegExp(encoded, "gu"))].map((match) =>
        decodeStraceString(match[0]).toString("utf8"),
      );
      const remaining = exec[2].replace(new RegExp(encoded, "gu"), "").replace(/[\s,]/gu, "");
      assert.equal(remaining, "", "complete exec argv without truncation");
      assert.equal(executables.has(row.tid), false, "no unexplained re-exec of an observed owner");
      executables.set(row.tid, {
        tid: row.tid,
        path: decodeStraceString(exec[1]).toString("utf8"),
        argv,
        at: row.at,
      });
    }
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
    events: [],
    exit: null,
  }));
  const byPid = new Map(owners.map((owner) => [owner.process, owner]));
  for (const row of rows) {
    const owner = byPid.get(leader(row.tid));
    if (!owner) continue;
    const io = /^(read|write|readv|writev)\((\d+),/u.exec(row.call);
    if (io && [0, 1, 2].includes(Number(io[2]))) {
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
    const exited = /^\+\+\+ exited with (\d+) \+\+\+$/u.exec(row.call);
    const killed = /^\+\+\+ killed by (SIG[A-Z0-9]+)(?: \(core dumped\))? \+\+\+$/u.exec(row.call);
    if ((exited || killed) && row.tid === owner.process)
      owner.exit = { code: exited ? Number(exited[1]) : null, signal: killed?.[1] ?? null };
  }
  return owners;
}

export function captureNativeOwners(tracePrefix, directory) {
  const parent = path.dirname(tracePrefix),
    prefix = path.basename(tracePrefix) + ".";
  const traces = fs
    .readdirSync(parent)
    .filter((file) => file.startsWith(prefix))
    .map((file) => {
      const suffix = file.slice(prefix.length);
      assert.match(suffix, /^\d+$/u);
      return { tid: Number(suffix), text: fs.readFileSync(path.join(parent, file), "utf8") };
    });
  assert.ok(traces.length > 0, "actual strace files are mandatory");
  const owners = parseStraceTexts(traces).filter(
    (owner) => owner.argv.includes("--lsp") || owner.argv.includes("--api"),
  );
  fs.mkdirSync(directory, { recursive: true });
  return owners.map((owner) => {
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
    };
    fs.writeFileSync(
      path.join(directory, `${owner.process}.json`),
      JSON.stringify(observation, null, 2) + "\n",
    );
    return observation;
  });
}
