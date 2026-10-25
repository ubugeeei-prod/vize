import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";

const root = process.env.VIZE_NATIVE_DTO_CAPTURE;
const load = (name) => JSON.parse(fs.readFileSync(path.join(root, name), "utf8"));
const save = (name, value) =>
  fs.writeFileSync(path.join(root, name), JSON.stringify(value, null, 2) + "\n");
const hash = (value) => createHash("sha256").update(value).digest("hex");
const provider = load("provider.json");
const source = load("complete-source-packets.json");
assert.equal(source.sdkVersion, provider.version);
assert.equal(source.sdkSha256, provider.executableSha256);
assert.equal(source.source, process.env.VIZE_NATIVE_DTO_SOURCE_SHA);
assert.equal(source.rows.length, 3);
const files = fs
  .readdirSync(root)
  .filter((file) => /^syscalls\.\d+$/.test(file))
  .sort();
assert(files.length, "Actual native syscalls required");
const threads = new Map();
const leaders = [];
const exits = new Map();
const records = [];
const unhex = (value) => Buffer.from(value.replaceAll("\\x", ""), "hex");
for (const file of files) {
  const tid = Number(file.split(".").at(-1));
  let pending;
  const lines = fs.readFileSync(path.join(root, file), "utf8").split("\n");
  for (const [index, original] of lines.entries()) {
    let line = original;
    if (line.includes("<unfinished ...>")) {
      assert.equal(pending, undefined, "Unobserved nested syscall");
      pending = line.replace("<unfinished ...>", "");
      continue;
    }
    if (line.includes("resumed>")) {
      assert(pending, "Resumed syscall has no observed start");
      line = pending + line.slice(line.indexOf("resumed>") + "resumed>".length);
      pending = undefined;
    }
    const exit = /\+\+\+ exited with (\d+) \+\+\+/.exec(line);
    if (exit) exits.set(tid, Number(exit[1]));
    if (line.includes("execve(") && line.endsWith("= 0")) {
      const decoded = line.replace(/(?:\\x[0-9a-f]{2})+/g, (value) =>
        unhex(value).toString("utf8"),
      );
      if (decoded.includes(provider.executable))
        leaders.push({ pid: tid, argvObservation: decoded, lsp: decoded.includes("--lsp") });
    }
    const child = /clone(?:3)?\(.*\)\s+=\s+(\d+)$/.exec(line);
    if (child) threads.set(Number(child[1]), tid);
    const call =
      /^([0-9]+\.[0-9]+) (read|write)\((\d+), "((?:\\x[0-9a-f]{2})*)", [0-9]+\)\s+=\s+(\d+)/.exec(
        line,
      );
    const vector = /^([0-9]+\.[0-9]+) (readv|writev)\((\d+), (.*)\)\s+=\s+(\d+)/.exec(line);
    if (!call && !vector) continue;
    const [, at, operation, fd, encoded, count] = call ?? vector;
    const data = call
      ? unhex(encoded)
      : Buffer.concat(
          [...encoded.matchAll(/iov_base="((?:\\x[0-9a-f]{2})*)"/g)].map((match) =>
            unhex(match[1]),
          ),
        );
    assert(data.length >= Number(count), `${file}:${index + 1}: truncated successful syscall`);
    if (Number(count))
      records.push({
        at,
        op: operation.replace(/v$/, ""),
        fd: Number(fd),
        tid,
        data: data.subarray(0, Number(count)),
        file,
        line: index + 1,
      });
  }
  // An unfinished blocking read at process exit carries no returned bytes.
}
const native = leaders.filter((entry) => entry.lsp);
assert.equal(native.length, 1, "One actual native LSP owner required");
const leader = native[0].pid;
assert.equal(exits.get(leader), 0, "Actual native owner must exit cleanly");
function owner(value) {
  const seen = new Set();
  while (threads.has(value)) {
    assert(!seen.has(value));
    seen.add(value);
    value = threads.get(value);
  }
  return value;
}
function frames(data) {
  let offset = 0;
  const packets = [];
  while (offset < data.length) {
    const marker = data.indexOf("\r\n\r\n", offset);
    assert(marker >= 0, "Unclosed native header");
    const header = data.subarray(offset, marker).toString("ascii");
    const lengths = [...header.matchAll(/^Content-Length: ([0-9]+)$/gim)];
    assert.equal(lengths.length, 1, header);
    const size = Number(lengths[0][1]);
    assert(size <= 16 * 1024 * 1024);
    const end = marker + 4 + size;
    assert(end <= data.length, "Truncated native body");
    const packet = JSON.parse(data.subarray(marker + 4, end).toString("utf8"));
    assert.equal(packet.jsonrpc, "2.0");
    packets.push(packet);
    offset = end;
  }
  assert.equal(offset, data.length);
  return packets;
}
const key = (value) => `${typeof value}:${value}`;
function relations(client, server) {
  for (const [requests, responses] of [
    [client, server],
    [server, client],
  ]) {
    const ids = requests.filter((row) => "id" in row && "method" in row).map((row) => key(row.id));
    const answers = responses
      .filter((row) => "id" in row && !("method" in row))
      .map((row) => key(row.id));
    assert.equal(new Set(ids).size, ids.length, "Duplicate native request ID");
    assert.deepEqual(ids.sort(), answers.sort(), "Unanswered or unmatched native request");
  }
}
const streams = new Map();
for (const row of records) {
  if (owner(row.tid) !== leader) continue;
  const id = `${row.fd}:${row.op}`;
  if (!streams.has(id)) streams.set(id, []);
  streams.get(id).push(row);
}
const observations = [];
fs.mkdirSync(path.join(root, "raw-streams"), { recursive: true });
for (const fd of [
  ...new Set([...streams.values()].flatMap((rows) => rows.map((row) => row.fd))),
].sort((a, b) => a - b)) {
  const pair = { fd };
  for (const [operation, direction] of [
    ["read", "client"],
    ["write", "server"],
  ]) {
    const rows = (streams.get(`${fd}:${operation}`) ?? []).sort(
      (a, b) => a.at.localeCompare(b.at) || a.tid - b.tid || a.line - b.line,
    );
    const raw = Buffer.concat(rows.map((row) => row.data));
    const start = raw.indexOf("Content-Length:");
    if (start < 0) continue;
    const payload = raw.subarray(start);
    const packets = frames(payload);
    const file = `raw-streams/owner-${leader}-fd-${fd}.${direction}.bin`;
    fs.writeFileSync(path.join(root, file), payload);
    pair[direction] = {
      file,
      bytes: payload.length,
      sha256: hash(payload),
      precedingNonProtocolBytes: start,
      packets,
    };
  }
  if (pair.client || pair.server) observations.push(pair);
}
const byFd = new Map(observations.map((entry) => [entry.fd, entry]));
assert(byFd.get(0)?.client && byFd.get(1)?.server);
relations(byFd.get(0).client.packets, byFd.get(1).server.packets);
const apiStreams = observations.filter((entry) => entry.fd >= 3 && entry.client && entry.server);
assert.equal(apiStreams.length, 1, "Exactly one attached native API stream required");
const api = apiStreams[0];
relations(api.client.packets, api.server.packets);
const requests = api.client.packets.filter((row) => "id" in row && "method" in row);
const responses = new Map(
  api.server.packets
    .filter((row) => "id" in row && !("method" in row))
    .map((row) => [key(row.id), row]),
);
const kinds = new Set([
  "getSyntacticDiagnostics",
  "getSemanticDiagnostics",
  "getSuggestionDiagnostics",
  "getDeclarationDiagnostics",
]);
const kindRows = [];
for (const request of requests) {
  if (!kinds.has(request.method)) continue;
  const params = request.params;
  if (source.role === "after" && provider.version === "7.1.0-beta") {
    assert(!("file" in params));
    assert.equal(params.files.length, 1);
  } else {
    assert(!("files" in params));
    assert.equal(typeof params.file, "string");
  }
  kindRows.push({ request, response: responses.get(key(request.id)) });
}
assert(kindRows.length >= 9);
for (const phase of source.rows) {
  const binary = fs.readFileSync(path.join(root, `${phase.phase}.source-file.bin`));
  assert.equal(hash(binary), phase.encodedSourceSha256);
  assert(
    [...responses.values()].some(
      (response) =>
        typeof response.result?.data === "string" &&
        Buffer.from(response.result.data, "base64").equals(binary),
    ),
    "Retained source bytes must occur in a complete API frame",
  );
}
save("closed-full-wire.json", {
  source: source.source,
  role: source.role,
  providerVersion: provider.version,
  providerSha256: provider.executableSha256,
  nativeOwner: leader,
  nativeExit: exits.get(leader),
  leaders,
  scope:
    "Complete actual native frames and request/reply identity; no response fields filtered. Earlier non-protocol filesystem bytes are counted explicitly.",
  streams: observations,
  compilerKindPackets: kindRows,
});
console.log(
  JSON.stringify({
    source: source.source,
    role: source.role,
    sdk: provider.version,
    nativeOwner: leader,
    apiCompilerKindPackets: kindRows.length,
    allFramesClosed: true,
  }),
);
