import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { pathToFileURL } from "node:url";
import {
  decodeNativeFrames,
  decodeNativeIo,
  parseStraceCapture,
  validateExecInventory,
} from "./lsp-strace.mjs";
import { readStraceLines } from "./lsp-strace-lines.mjs";

const max = (values) => values.reduce((a, b) => (a > b ? a : b));
const min = (values) => values.reduce((a, b) => (a < b ? a : b));
const bounds = (row) => {
  const start = BigInt(row.interval.startNanoseconds);
  const end = BigInt(row.interval.endNanoseconds);
  assert.ok(start <= end, "ordered conservative syscall interval");
  return { start, end };
};
function decimal(text) {
  const parts = /^(\d+)\.(\d{1,9})$/u.exec(text);
  assert.ok(parts, "actual decimal timestamp or duration");
  return {
    value: BigInt(parts[1]) * 1_000_000_000n + BigInt(parts[2].padEnd(9, "0")),
    quantum: 10n ** BigInt(9 - parts[2].length),
  };
}
function returnBounds(row) {
  const start = decimal(row.at),
    duration = decimal(row.duration),
    resumed = decimal(row.completedAt);
  // No exact kernel timestamp is invented. Rounded printed measurements give
  // conservative lower/upper bounds; tied or overlapping events refuse.
  const lower = max([
    start.value + duration.value - start.quantum - duration.quantum,
    resumed.value - resumed.quantum,
  ]);
  const upper = bounds(row).end;
  assert.ok(lower <= upper);
  return { lower, upper };
}

function stream(owner, operation, fd) {
  let offset = 0;
  const segments = [];
  const chunks = [];
  for (const event of owner.events) {
    if (event.operation !== operation || event.fd !== fd || !event.bytes) continue;
    assert.ok(Buffer.isBuffer(event.bytes));
    assert.equal(event.count, event.bytes.length);
    assert.ok(event.count > 0);
    chunks.push(event.bytes);
    segments.push({ offset, end: offset + event.bytes.length, interval: bounds(event) });
    offset += event.bytes.length;
  }
  const bytes = Buffer.concat(chunks);
  const frames =
    fd === 2
      ? []
      : decodeNativeFrames(bytes).map((frame) => {
          const covered = segments.filter(
            (segment) => segment.offset < frame.offset + frame.bytes && segment.end > frame.offset,
          );
          assert.ok(covered.length > 0);
          return {
            ...frame,
            interval: {
              start: min(covered.map((row) => row.interval.start)),
              end: max(covered.map((row) => row.interval.end)),
            },
          };
        });
  return { bytes, frames };
}
function one(rows, predicate, label) {
  const matches = rows.filter(predicate);
  assert.equal(matches.length, 1, label);
  return matches[0];
}
function duration(value) {
  if (value === "0s") return true;
  const ranks = { h: 6, m: 5, s: 4, ms: 3, µs: 2, ns: 1 };
  let previous = 7,
    count = 0;
  while (value !== "") {
    const part = /^(\d+(?:\.\d+)?)(ms|µs|ns|h|m|s)/u.exec(value);
    if (!part || ranks[part[2]] >= previous) return false;
    previous = ranks[part[2]];
    value = value.slice(part[0].length);
    count++;
  }
  return count > 0;
}
function completion(frame, id) {
  const packet = frame.packet;
  if (packet.method !== "window/logMessage" || packet.params?.type !== 3) return false;
  const match = /^handled method 'shutdown' \(([1-9]\d*)\) in (.+)$/u.exec(
    packet.params?.message ?? "",
  );
  if (!match || match[1] !== String(id)) return false;
  assert.ok(duration(match[2]), "complete current native completion duration");
  assert.deepEqual(Object.keys(packet).sort(), ["jsonrpc", "method", "params"]);
  assert.deepEqual(Object.keys(packet.params).sort(), ["message", "type"]);
  assert.equal(packet.jsonrpc, "2.0");
  return true;
}
function initialize(request, template, sourcePid, workspace, temporaryRoot) {
  const root = request.params?.rootPath;
  assert.equal(typeof root, "string");
  assert.ok(path.isAbsolute(root));
  const relative = path.relative(path.join(temporaryRoot, "vize-canon/editor/sessions"), root);
  assert.ok(
    root === workspace || /^session-[A-Za-z0-9]+\/projects\/[a-f0-9]{64}$/u.test(relative),
    "source-owned authored or editor mirror root",
  );
  const values = {
    "${sourcePid}": sourcePid,
    "${nativeRoot}": root,
    "${nativeRootUri}": pathToFileURL(root).href,
    "${nativeWorkspaceName}": path.basename(root),
  };
  const relocate = (value) => {
    if (typeof value === "string" && Object.hasOwn(values, value)) return values[value];
    if (Array.isArray(value)) return value.map(relocate);
    if (value && typeof value === "object")
      return Object.fromEntries(
        Object.entries(value).map(([key, child]) => [key, relocate(child)]),
      );
    return value;
  };
  assert.deepEqual(
    request,
    relocate(template),
    "entire source-owned Initialize request with only declared transport fields relocated",
  );
}

/**
 * Pure oracle input: strict decoder owners {process,argv,path,executable,exit,
 * rows:[{tid,call,interval,...}],events:[{operation,fd,count,bytes?,interval,
 * at,completedAt,duration,...}]}. Unit seeds are synthetic and grant no wire
 * credit. The production wrapper derives every owner/event from original raw
 * strace, validates physical exec custody, and compares complete outer bytes.
 */
export function assertOwnedShutdownOrder({
  sourceOwner,
  nativeOwners,
  outerShutdown,
  knownContract,
  initializeRequest,
  workspace,
  expectedExecutable,
  temporaryRoot = os.tmpdir(),
}) {
  assert.ok(nativeOwners.length > 0, "actual initialized compiler owners are required");
  assert.ok(expectedExecutable);
  const sourceClient = stream(sourceOwner, "read", 0),
    sourceServer = stream(sourceOwner, "write", 1);
  const request = one(
    sourceClient.frames,
    (frame) => frame.packet.method === "shutdown",
    "one whole outer shutdown request",
  );
  assert.deepEqual(request.packet, { jsonrpc: "2.0", id: request.packet.id, method: "shutdown" });
  assert.deepEqual(outerShutdown, { jsonrpc: "2.0", id: request.packet.id, result: null });
  const ack = one(
    sourceServer.frames,
    (frame) => !frame.packet.method && frame.packet.id === request.packet.id,
    "one whole outer shutdown ACK",
  );
  assert.deepEqual(ack.packet, outerShutdown);
  const proof = [];
  for (const owner of nativeOwners) {
    assert.equal(owner.executable.physicalPath, expectedExecutable.path);
    assert.equal(owner.executable.sha256, expectedExecutable.sha256);
    assert.deepEqual(owner.argv, [owner.path, "--lsp", "--stdio"]);
    assert.deepEqual(owner.exit, { code: 0, signal: null });
    const client = stream(owner, "read", 0),
      server = stream(owner, "write", 1);
    const initialized = one(
      client.frames,
      (frame) => frame.packet.method === "initialize",
      "one actual SDK Initialize request",
    );
    initialize(
      initialized.packet,
      initializeRequest,
      sourceOwner.process,
      workspace,
      temporaryRoot,
    );
    const initializedReply = one(
      server.frames,
      (frame) => !frame.packet.method && frame.packet.id === initialized.packet.id,
      "one complete SDK Initialize reply",
    );
    assert.deepEqual(initializedReply.packet, {
      jsonrpc: "2.0",
      id: initialized.packet.id,
      result: knownContract,
    });
    const shutdown = one(
      client.frames,
      (frame) => frame.packet.method === "shutdown",
      "every initialized SDK receives one Shutdown",
    );
    assert.ok(Number.isSafeInteger(shutdown.packet.id) && shutdown.packet.id > 0);
    assert.deepEqual(shutdown.packet, {
      jsonrpc: "2.0",
      id: shutdown.packet.id,
      method: "shutdown",
    });
    assert.ok(initialized.offset + initialized.bytes <= shutdown.offset);
    const shutdownAck = one(
      server.frames,
      (frame) => !frame.packet.method && frame.packet.id === shutdown.packet.id,
      "one native Shutdown ACK",
    );
    assert.deepEqual(shutdownAck.packet, { jsonrpc: "2.0", id: shutdown.packet.id, result: null });
    const info = one(
      server.frames,
      (frame) => completion(frame, shutdown.packet.id),
      "one current native Shutdown completion Info",
    );
    assert.ok(
      shutdownAck.offset + shutdownAck.bytes <= info.offset,
      "SDK ACK bytes precede the matching current completion Info",
    );
    assert.equal(
      client.frames.some((frame) => frame.packet.method === "exit"),
      false,
      "qualified702 closure uses EOF only",
    );
    assert.equal(
      stream(owner, "write", 2).bytes.length,
      0,
      "whole captured SDK stderr remains empty",
    );
    const eof = owner.events.filter(
      (event) => event.operation === "read" && event.fd === 0 && event.count === 0,
    );
    assert.ok(eof.length > 0, "actual successful native stdin read(0)=0 is mandatory");
    const eofReturns = eof.map(returnBounds);
    for (const returned of eofReturns)
      assert.ok(
        info.interval.end < returned.lower,
        "current Info completes before actual stdin EOF return; ambiguous intervals refuse",
      );
    const tids = new Set(owner.rows.map((row) => row.tid));
    for (const row of owner.rows) {
      const clone = /^(?:clone|clone3)\(.*CLONE_THREAD.*\)\s+=\s+(\d+)/u.exec(row.call);
      if (clone) tids.add(Number(clone[1]));
    }
    const terminals = [...tids].map((tid) =>
      one(
        owner.rows,
        (row) => row.tid === tid && /^\+\+\+ exited with 0 \+\+\+$/u.test(row.call),
        "every SDK thread has actual terminal status0",
      ),
    );
    const exitGroup = owner.rows.filter((row) => /^exit_group\(0\)\s+=/u.test(row.call));
    assert.ok(exitGroup.length > 0, "actual successful native exit_group(0) remains observed");
    const exitsAt = min(exitGroup.map((row) => bounds(row).start));
    for (const returned of eofReturns)
      assert.ok(returned.upper < exitsAt, "actual EOF completion precedes process exit");
    const lastTerminal = max(terminals.map((row) => bounds(row).end));
    assert.ok(
      lastTerminal < ack.interval.start,
      "every actual SDK thread closes BEFORE outer Shutdown ACK write starts",
    );
    proof.push({
      process: owner.process,
      sourcePid: sourceOwner.process,
      initializeRequest: initialized.packet,
      initializeReply: initializedReply.packet,
      shutdownRequest: shutdown.packet,
      shutdownAck: shutdownAck.packet,
      completionInfo: info.packet,
      stdinEof: eof.map(({ bytes: _bytes, ...row }) => row),
      threadTerminals: terminals,
      lastTerminalUpperNanoseconds: String(lastTerminal),
      outerAckStartNanoseconds: String(ack.interval.start),
      noFullSyscallCoverageClaim: true,
    });
  }
  return { outerShutdown: ack.packet, everyInitializedOwnerClosedBeforeAck: true, owners: proof };
}

export function captureOwnedShutdownOrder({
  prefix,
  directory,
  authority,
  outer,
  outerShutdown,
  knownContract,
  initializeRequest,
  workspace,
  temporaryRoot,
}) {
  const parent = path.dirname(prefix),
    basename = path.basename(prefix) + ".";
  const traces = fs
    .readdirSync(parent)
    .filter((name) => name.startsWith(basename))
    .map((name) => {
      const tid = name.slice(basename.length);
      assert.match(tid, /^\d+$/u);
      return { tid: Number(tid), lines: readStraceLines(path.join(parent, name)) };
    });
  const parsed = parseStraceCapture(traces);
  const owners = validateExecInventory(parsed, authority).map(decodeNativeIo);
  const sourceOwner = one(
    owners,
    (owner) => owner.role === "source-lsp",
    "one actual source process",
  );
  for (const [name, operation, fd] of [
    ["client", "read", 0],
    ["server", "write", 1],
    ["stderr", "write", 2],
  ])
    assert.deepEqual(
      stream(sourceOwner, operation, fd).bytes,
      fs.readFileSync(outer.streams[name].path),
      `complete original outer ${name} bytes match source syscalls`,
    );
  const proof = assertOwnedShutdownOrder({
    sourceOwner,
    nativeOwners: owners.filter((owner) => owner.role === "selected-native-lsp"),
    outerShutdown,
    knownContract,
    initializeRequest,
    workspace,
    expectedExecutable: authority.sdk,
    temporaryRoot,
  });
  proof.syscallObservation = {
    syscallsComplete: parsed.syscallsComplete,
    terminalUnknownSyscalls: parsed.terminalUnknownSyscalls,
    terminalExitEntries: parsed.terminalExitEntries,
    terminalReadEntries: parsed.terminalReadEntries,
    attribution: "Original global decoder classifications, without invented per-owner empty lists.",
  };
  fs.writeFileSync(
    path.join(directory, "owned-shutdown-order.json"),
    JSON.stringify(proof, null, 2) + "\n",
  );
  return proof;
}
