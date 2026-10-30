import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { createInterface } from "node:readline";
import { verifyNativeRelations } from "./stable-watcher-boundary.ts";

type Registration = {
  id: string;
  method: string;
  registerOptions: { watchers: Array<{ kind: number; globPattern: string }> };
};
type Packet = {
  jsonrpc: string;
  id: string | number;
  method?: string;
  params: {
    registrations?: Registration[];
    file?: string;
    files?: string[];
    [name: string]: unknown;
  };
  result: { data?: string } | null;
};
type Syscall = {
  at: string;
  op: string;
  fd: number;
  tid: number;
  data: Buffer;
  file: string;
  line: number;
};
type Stream = {
  file: string;
  bytes: number;
  sha256: string;
  precedingNonProtocolBytes: number;
  packets: Packet[];
};
type StreamPair = { fd: number; client?: Stream; server?: Stream };
type Provider = { version: string; executable: string; executableSha256: string };
type Source = {
  source: string;
  root: string;
  role: string;
  sdkVersion: string;
  sdkSha256: string;
  rows: Array<{ phase: string; encodedSourceSha256: string; lower: { refused: string } }>;
  attached: { root: string; rows: unknown[] };
};
type Receipt = {
  scope: string;
  nativeOwner: number;
  nativeExit: number | undefined;
  nativeStdinEof: boolean;
  exitNotifications: number;
  apiAttached: boolean;
  registrationRefusals: number;
  stderr: { file: string; bytes: number; sha256: string };
  streams: StreamPair[];
  compilerKindPackets: Array<{ request: Packet; response: Packet | undefined }>;
};

const root = process.env.VIZE_NATIVE_DTO_CAPTURE!;
const load = (name: string): unknown => JSON.parse(fs.readFileSync(path.join(root, name), "utf8"));
const save = (name: string, value: unknown) =>
  fs.writeFileSync(path.join(root, name), JSON.stringify(value, null, 2) + "\n");
const hash = (value: Uint8Array) => createHash("sha256").update(value).digest("hex");
const provider = load("provider.json") as Provider;
const source = load("complete-source-packets.json") as Source;
assert.equal(source.sdkVersion, provider.version);
assert.equal(source.sdkSha256, provider.executableSha256);
assert.equal(source.source, process.env.VIZE_NATIVE_DTO_SOURCE_SHA);
assert.equal(source.rows.length, 3);
const files = fs
  .readdirSync(root)
  .filter((file) => /^syscalls\.\d+$/.test(file))
  .sort();
assert(files.length, "Actual native syscalls required");
const threads = new Map<number, number>();
const leaders: Array<{ pid: number; argvObservation: string; lsp: boolean }> = [];
const exits = new Map<number, number>();
const records: Syscall[] = [];
const eofReads: Array<{ tid: number; fd: number; at: string }> = [];
const unhex = (value: string) => Buffer.from(value.replaceAll("\\x", ""), "hex");
for (const file of files) {
  const tid = Number(file.split(".").at(-1));
  let pending: string | undefined;
  let lineNumber = 0;
  const lines = createInterface({
    input: fs.createReadStream(path.join(root, file)),
    crlfDelay: Infinity,
  });
  for await (const original of lines) {
    lineNumber += 1;
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
    if (child && line.includes("CLONE_THREAD")) threads.set(Number(child[1]), tid);
    const call =
      /^([0-9]+\.[0-9]+) (read|write)\((\d+), "((?:\\x[0-9a-f]{2})*)", [0-9]+\)\s+=\s+(\d+)/.exec(
        line,
      );
    const vector = /^([0-9]+\.[0-9]+) (readv|writev)\((\d+), (.*)\)\s+=\s+(\d+)/.exec(line);
    if (!call && !vector) continue;
    const [, at, operation, fd, encoded, count] = (call ?? vector)!;
    const data = call
      ? unhex(encoded)
      : Buffer.concat(
          [...encoded.matchAll(/iov_base="((?:\\x[0-9a-f]{2})*)"/g)].map((match) =>
            unhex(match[1]),
          ),
        );
    assert(data.length >= Number(count), `${file}:${lineNumber}: truncated successful syscall`);
    if (Number(count) === 0 && operation === "read") eofReads.push({ tid, fd: Number(fd), at });
    if (Number(count))
      records.push({
        at,
        op: operation.replace(/v$/, ""),
        fd: Number(fd),
        tid,
        data: data.subarray(0, Number(count)),
        file,
        line: lineNumber,
      });
  }
  // An unfinished blocking read at process exit carries no returned bytes.
}
const native = leaders.filter((entry) => entry.lsp).sort((a, b) => a.pid - b.pid);
assert(native.length >= 2, "Separate owned-API and attached-editor native scopes required");
function owner(value: number): number {
  const seen = new Set();
  while (threads.has(value)) {
    assert(!seen.has(value));
    seen.add(value);
    value = threads.get(value)!;
  }
  return value;
}
function frames(data: Buffer): Packet[] {
  let offset = 0;
  const packets: Packet[] = [];
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
    const packet = JSON.parse(data.subarray(marker + 4, end).toString("utf8")) as Packet;
    assert.equal(packet.jsonrpc, "2.0");
    packets.push(packet);
    offset = end;
  }
  assert.equal(offset, data.length);
  return packets;
}
const key = (value: unknown) => `${typeof value}:${value}`;
const watcherBoundaryObservations: NonNullable<ReturnType<typeof verifyNativeRelations>>[] = [];
function relations(
  client: Packet[],
  server: Packet[],
  allowStableWatcherRegistrationRefusal: boolean,
) {
  const observation = verifyNativeRelations(client, server, {
    source: source.source,
    role: source.role,
    sdkVersion: provider.version,
    sdkSha256: provider.executableSha256,
    sdkExecutable: provider.executable,
    projectRoots: [source.root, source.attached.root],
  });
  if (observation) {
    assert.equal(allowStableWatcherRegistrationRefusal, true);
    watcherBoundaryObservations.push(observation);
  }
  return observation === null ? 0 : 1;
}
const kinds = new Set([
  "getSyntacticDiagnostics",
  "getSemanticDiagnostics",
  "getSuggestionDiagnostics",
  "getDeclarationDiagnostics",
]);
const semantic = new Set([
  ...kinds,
  "getDiagnostics",
  "getDiagnosticsForFile",
  "getDiagnosticsForProject",
  "getHoverAtPosition",
  "getDefinitionAtPosition",
  "getReferencesAtPosition",
  "getCompletionAtPosition",
  "getRenameAtPosition",
  "getTypeAtPosition",
  "getSymbolAtPosition",
  "getSourceFile",
]);
const receipts: Receipt[] = [];
const stableRegistrationRuntime = provider.version === "7.0.2";
const stableRegistrationBaseline = source.role === "before" && stableRegistrationRuntime;
let registrationRefusals = 0;
fs.mkdirSync(path.join(root, "raw-streams"), { recursive: true });
for (const [index, process] of native.entries()) {
  const leader = process.pid;
  const scope = index === 0 ? "api-owned-explicit-file-changes" : "attached-editor-overlay";
  const streams = new Map<string, Syscall[]>();
  for (const row of records) {
    if (owner(row.tid) !== leader) continue;
    const id = `${row.fd}:${row.op}`;
    if (!streams.has(id)) streams.set(id, []);
    streams.get(id)!.push(row);
  }
  const joined = (rows: Syscall[]) =>
    Buffer.concat(
      [...rows]
        .sort((a, b) => a.at.localeCompare(b.at) || a.tid - b.tid || a.line - b.line)
        .map((row) => row.data),
    );
  const stderr = joined(streams.get("2:write") ?? []);
  const stderrFile = `raw-streams/owner-${leader}.stderr.bin`;
  fs.writeFileSync(path.join(root, stderrFile), stderr);
  const observations: StreamPair[] = [];
  for (const fd of [
    ...new Set([...streams.values()].flatMap((rows) => rows.map((row) => row.fd))),
  ].sort((a, b) => a - b)) {
    const pair: StreamPair = { fd };
    for (const [operation, direction] of [
      ["read", "client"],
      ["write", "server"],
    ] as const) {
      const raw = joined(streams.get(`${fd}:${operation}`) ?? []);
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
  let ownerRegistrationRefusals = relations(
    byFd.get(0)!.client!.packets,
    byFd.get(1)!.server!.packets,
    stableRegistrationBaseline,
  );
  const apiStreams = observations.filter((entry) => entry.fd >= 3 && entry.client && entry.server);
  assert(
    apiStreams.length <= 1,
    "At most one retained API attachment may belong to one native LSP owner",
  );
  const api = apiStreams[0];
  if (api)
    ownerRegistrationRefusals += relations(
      api.client!.packets,
      api.server!.packets,
      stableRegistrationBaseline,
    );
  registrationRefusals += ownerRegistrationRefusals;
  const requests = api?.client!.packets.filter((row) => "id" in row && "method" in row) ?? [];
  const responses = new Map(
    api
      ?.server!.packets.filter((row) => "id" in row && !("method" in row))
      .map((row) => [key(row.id), row]) ?? [],
  );
  const kindRows: Array<{ request: Packet; response: Packet | undefined }> = [];
  for (const request of requests) {
    if (!kinds.has(request.method!)) continue;
    const params = request.params;
    if (source.role === "after" && provider.version === "7.1.0-beta") {
      assert(!("file" in params));
      assert.equal(params.files!.length, 1);
    } else {
      assert(!("files" in params));
      assert.equal(typeof params.file, "string");
    }
    kindRows.push({ request, response: responses.get(key(request.id)) });
  }
  if (index === 0) {
    assert(api, "The API-owned explicit FileChanges scope must retain its attachment");
    assert(kindRows.length >= 9);
    const updates = requests.filter((row) =>
      ["updateSnapshot", "deriveSnapshot"].includes(row.method!),
    );
    assert(
      updates.some((row) => JSON.stringify(row.params).includes("source.ts")),
      "Explicit owned file changes required",
    );
    for (const phase of source.rows) {
      const binary = fs.readFileSync(path.join(root, `${phase.phase}.source-file.bin`));
      assert.equal(hash(binary), phase.encodedSourceSha256);
      assert(
        [...responses.values()].some(
          (response) =>
            typeof response.result?.data === "string" &&
            Buffer.from(response.result.data, "base64").equals(binary),
        ),
        "Owned source bytes must occur in a complete API frame",
      );
    }
  } else if (source.role === "after" && provider.version === "7.1.0-beta") {
    assert.deepEqual(
      requests.filter((row) => semantic.has(row.method!)),
      [],
      "Attached DerivedSnapshots must not use semantic API queries",
    );
  }
  const lspClient = byFd.get(0)!.client!.packets;
  const lspServer = byFd.get(1)!.server!.packets;
  const shutdown = lspClient.filter((row) => row.method === "shutdown");
  assert.equal(shutdown.length, 1, "Exactly one native shutdown request required");
  assert(
    lspServer.some((row) => row.id === shutdown[0].id && row.result === null),
    "Whole shutdown ACK required",
  );
  receipts.push({
    scope,
    nativeOwner: leader,
    nativeExit: exits.get(leader),
    nativeStdinEof: eofReads.some((row) => owner(row.tid) === leader && row.fd === 0),
    exitNotifications: lspClient.filter((row) => row.method === "exit").length,
    apiAttached: api !== undefined,
    registrationRefusals: ownerRegistrationRefusals,
    stderr: { file: stderrFile, bytes: stderr.length, sha256: hash(stderr) },
    streams: observations,
    compilerKindPackets: kindRows,
  });
}
assert.equal(source.attached.rows.length, 4);
assert.equal(
  stableRegistrationBaseline ? registrationRefusals <= 1 : registrationRefusals === 0,
  true,
  "Only the exact historical stable watcher registration request may race shutdown",
);
assert.equal(receipts[0].apiAttached, true, "The explicit owned scope must retain its API wire");
if (provider.version === "7.1.0-beta") {
  assert(
    receipts.slice(1).some((receipt) => !receipt.apiAttached),
    "The beta attached-editor route must retain an LSP-only native owner",
  );
}
save("stable-watcher-post-shutdown-observation.json", {
  source: source.source,
  role: source.role,
  observations: watcherBoundaryObservations,
  scope:
    "Distinct before-only post-shutdown refusals; original whole packets retained without ID or payload normalization.",
});
save("closed-full-wire.json", {
  source: source.source,
  role: source.role,
  providerVersion: provider.version,
  providerSha256: provider.executableSha256,
  scope:
    "Independent API-owned explicit disk/FileChanges snapshots and attached editor overlay routes. Complete native frames retained without response-field filtering.",
  leaders,
  owners: receipts,
});
function requireStableWatcherShutdown(bytes: Buffer) {
  const watch =
    /^Added new watch: root files for (\/tmp\/[^\n]+)\/tsconfig\.json watcher 6\.0\n\t\1\/\*\*\/\*\n\n/u.exec(
      bytes.toString("utf8"),
    );
  if (watch) {
    const prefix = Buffer.from(watch[0]);
    assert.deepEqual(bytes.subarray(0, prefix.length), prefix);
    bytes = bytes.subarray(prefix.length);
  }
  if (bytes.length === 0 || bytes.equals(Buffer.from("\n"))) return;
  if (bytes[0] === 0x0a) {
    bytes = bytes.subarray(1);
  }
  assert.match(
    bytes.toString("utf8"),
    /^1 paths ineligible for watching\nerrors updating watches: \[failed to register file watcher: context canceled(?: failed to register file watcher: context canceled){0,3}\]\n$/u,
    "Stable control permits only its exact post-shutdown watcher cancellation",
  );
}
// The immutable beta baseline is an expected-refusal observation. This never
// grants functional or lifecycle PASS to its DTO refusal or standard-exit error.
const baseline = source.role === "before" && provider.version === "7.1.0-beta";
if (baseline) {
  assert(
    source.rows.every(
      (row) =>
        row.lower.refused ===
        "Malformed getSemanticDiagnostics response: unknown field `endPosition`, expected one of `fileName`, `pos`, `end`, `code`, `category`, `text`, `messageChain`, `relatedInformation`, `reportsUnnecessary`, `reportsDeprecated`",
    ),
    "Expected actual old-source DTO refusal required",
  );
  let refused = 0;
  for (const receipt of receipts) {
    assert.equal(
      receipt.exitNotifications,
      1,
      "Old source standard-exit boundary must remain observed",
    );
    const bytes = fs.readFileSync(path.join(root, receipt.stderr.file));
    const clean = receipt.nativeExit === 0 && bytes.length === 0;
    const known =
      (receipt.nativeExit === 0 &&
        bytes.equals(Buffer.from("error handling method 'exit': EOF\n"))) ||
      (receipt.nativeExit === 1 && bytes.equals(Buffer.from("context canceled\n")));
    assert(clean || known, "Unexpected baseline terminal state must fail observation");
    if (known) refused++;
  }
  assert(refused > 0, "Expected baseline lifecycle refusal must be observed");
} else {
  for (const receipt of receipts) {
    if (provider.version === "7.1.0-beta") {
      assert.equal(
        receipt.exitNotifications,
        0,
        "Candidate beta requires authenticated EOF-only shutdown",
      );
      assert.equal(receipt.nativeStdinEof, true, "Actual native beta stdin EOF must be observed");
    }
    assert.equal(receipt.nativeExit, 0, "Actual candidate/control native owner must exit cleanly");
    const bytes = fs.readFileSync(path.join(root, receipt.stderr.file));
    if (provider.version === "7.0.2") {
      if (bytes.length) requireStableWatcherShutdown(bytes);
    } else {
      assert.equal(bytes.length, 0, "Nonempty native beta stderr refuses qualification");
    }
  }
}
const disposition = {
  source: source.source,
  role: source.role,
  sdk: provider.version,
  roleKind: baseline
    ? "baseline-observation-known-dto-refusal"
    : stableRegistrationBaseline
      ? registrationRefusals === 1
        ? "baseline-observation-known-post-shutdown-registration-refusal"
        : "baseline-observation-registration-acknowledged"
      : "functional-source-control",
  observerCompleted: true,
  dtoFunctionalPass: !baseline && registrationRefusals === 0,
  lifecyclePass: !baseline && registrationRefusals === 0,
  registrationPass: registrationRefusals === 0,
  nativeOwners: receipts.length,
  allFramesClosed: true,
};
save("role-disposition.json", disposition);
console.log(JSON.stringify(disposition));
