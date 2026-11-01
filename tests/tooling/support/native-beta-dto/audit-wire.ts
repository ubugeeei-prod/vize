import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { requireCandidateStableStderr } from "./stable-watcher-boundary.ts";
import { verifyHistoricalWatcherRelations } from "./stable-watcher-late-root.ts";
import {
  attachedOwners,
  verifyObserverDependencies,
  verifyCompiledInput,
} from "./beta-attached-route.ts";
import { verifyAfterLogicalRevision } from "./logical-revision.ts";
import { verifyBetaLogicalRoute } from "./logical-attached-route.ts";
import { loadAfter4CleanupSource } from "./after4-cleanup-source.ts";
import type { Packet, Syscall, StreamPair, Provider, Source, Receipt } from "./audit-wire-types.ts";
import {
  readNativeTrace,
  nativeOwnerTerminalState,
  requireNoUnknownNativeWrites,
  requireSingleTerminalStderrRole,
} from "./native-io.ts";
import { observeHistoricalTerminalStderr } from "./terminal-stderr-unavailable.ts";
import { observeHistoricalSplitWatchLog } from "./stable-watcher-mixed.ts";
import {
  historicalRootWarning,
  matchesHistoricalRootWarning,
  observeRootWatchWarning as observeHistoricalRootWatchWarning,
} from "./historical-acknowledged-root-warning.ts";
import { buildRoleDisposition } from "./role-disposition.ts";

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
const prepared = load("source-role.json") as Record<string, unknown>;
verifyObserverDependencies(
  source.observerDependencies,
  prepared.harnessDependencies,
  source.observerSha256,
  prepared.harness,
);
verifyCompiledInput(source.attached.observerInputDependencies, prepared.observerInputDependencies);
const watcherContext = () => ({
  source: source.source,
  role: source.role,
  sdkVersion: provider.version,
  sdkSha256: provider.executableSha256,
  sdkExecutable: provider.executable,
  projectRoots: [source.root, source.attached.root],
});
const trace = await readNativeTrace(root, provider.executable);
const { threads, leaders, exits, records, eofReads } = trace;
save("native-execve-handoffs.json", trace.execveHandoffs);
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
const key = (value: unknown) => {
  assert(
    typeof value === "string" || (typeof value === "number" && Number.isSafeInteger(value)),
    "Native request ID must be a string or safe integer",
  );
  return `${typeof value}:${value}`;
};
const watcherBoundaryObservations: NonNullable<
  ReturnType<typeof verifyHistoricalWatcherRelations>
>[] = [];
const splitWatchLogObservations: Array<{
  nativeOwner: number;
  observation: ReturnType<typeof observeHistoricalSplitWatchLog>;
}> = [];
const terminalStderrObservations: Array<{
  nativeOwner: number;
  observation: ReturnType<typeof observeHistoricalTerminalStderr>;
}> = [];
const rootWatchWarningObservations: Array<{
  nativeOwner: number;
  observation: ReturnType<typeof observeHistoricalRootWatchWarning>;
}> = [];
function rootWarning(receipt: Receipt, terminal: ReturnType<typeof nativeOwnerTerminalState>) {
  const api = receipt.streams.find((row) => row.fd >= 3 && row.client && row.server)!;
  rootWatchWarningObservations.push({
    nativeOwner: receipt.nativeOwner,
    observation: observeHistoricalRootWatchWarning(
      receipt.streams.find((row) => row.fd === 0)!.client!.packets,
      receipt.streams.find((row) => row.fd === 1)!.server!.packets,
      watcherContext(),
      records.filter((row) => owner(row.tid) === receipt.nativeOwner),
      eofReads.filter((row) => owner(row.tid) === receipt.nativeOwner && row.fd === 0),
      terminal,
      {
        client: api.client!.packets,
        server: api.server!.packets,
        precedingNonProtocolBytes: api.client!.precedingNonProtocolBytes,
      },
    ),
  });
}
function relations(
  client: Packet[],
  server: Packet[],
  allowStableWatcherRegistrationRefusal: boolean,
) {
  const observation = verifyHistoricalWatcherRelations(client, server, watcherContext());
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
if (source.role === "after")
  save(
    "attached-logical-revision.json",
    verifyAfterLogicalRevision(
      attachedOwners(receipts),
      watcherContext(),
      process.env.VIZE_NATIVE_DTO_SOURCE_SHA!,
      source.attached,
    ),
  );
if (provider.version === "7.1.0-beta") {
  if (source.role === "before")
    assert(
      receipts.slice(1).some((receipt) => !receipt.apiAttached),
      "The beta attached-editor route must retain an LSP-only native owner",
    );
  else
    save(
      "beta-attached-semantic-route.json",
      verifyBetaLogicalRoute(
        attachedOwners(receipts),
        watcherContext(),
        process.env.VIZE_NATIVE_DTO_SOURCE_SHA!,
        source.attached.root,
        source.attached.rows,
        source.attached,
        loadAfter4CleanupSource(root, source),
      ),
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
save(
  "native-io-terminal-states.json",
  native.map((row) => nativeOwnerTerminalState(trace, row.pid)),
);
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
if (!stableRegistrationBaseline)
  for (const row of native)
    requireNoUnknownNativeWrites(nativeOwnerTerminalState(trace, row.pid).unknownWrites);
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
    const terminal = nativeOwnerTerminalState(trace, receipt.nativeOwner);
    if (terminal.unknownWrites.length) {
      assert.ok(stableRegistrationBaseline, "Unknown native writes refuse current qualification");
      if (terminal.unknownWrites[0]?.requestedBytes === historicalRootWarning.length) {
        rootWarning(receipt, terminal);
        continue;
      }
      assert.equal(receipt.registrationRefusals, 0);
      terminalStderrObservations.push({
        nativeOwner: receipt.nativeOwner,
        observation: observeHistoricalTerminalStderr(
          receipt.streams.find((row) => row.fd === 0)!.client!.packets,
          receipt.streams.find((row) => row.fd === 1)!.server!.packets,
          watcherContext(),
          records.filter((row) => owner(row.tid) === receipt.nativeOwner),
          eofReads.filter((row) => owner(row.tid) === receipt.nativeOwner && row.fd === 0),
          terminal,
        ),
      });
      continue;
    }
    if (provider.version === "7.0.2") {
      if (source.role === "after") requireCandidateStableStderr(bytes);
      else if (stableRegistrationBaseline && matchesHistoricalRootWarning(bytes))
        rootWarning(receipt, terminal);
      else if (stableRegistrationBaseline && bytes[0] === 0x09) {
        assert.equal(receipt.registrationRefusals, 0);
        splitWatchLogObservations.push({
          nativeOwner: receipt.nativeOwner,
          observation: observeHistoricalSplitWatchLog(
            receipt.streams.find((row) => row.fd === 0)!.client!.packets,
            receipt.streams.find((row) => row.fd === 1)!.server!.packets,
            watcherContext(),
            records.filter((row) => owner(row.tid) === receipt.nativeOwner),
            eofReads.filter((row) => owner(row.tid) === receipt.nativeOwner && row.fd === 0),
            terminal,
          ),
        });
      } else if (bytes.length) requireStableWatcherShutdown(bytes);
    } else {
      assert.equal(bytes.length, 0, "Nonempty native beta stderr refuses qualification");
    }
  }
}
assert.ok(splitWatchLogObservations.length <= 1, "one historical split watch log observation only");
save("stable-split-watch-log-observation.json", splitWatchLogObservations);
const terminalOwners = [
  ...terminalStderrObservations.map((row) => row.nativeOwner),
  ...rootWatchWarningObservations
    .filter((row) => !row.observation.stderrOutputComplete)
    .map((row) => row.nativeOwner),
];
if (terminalOwners.length)
  requireSingleTerminalStderrRole(
    terminalOwners,
    native.map((row) => nativeOwnerTerminalState(trace, row.pid)),
  );
save("terminal-stderr-observation.json", terminalStderrObservations);
save("historical-root-watch-warning-observation.json", rootWatchWarningObservations);
const disposition = buildRoleDisposition({
  source: source.source,
  role: source.role,
  sdk: provider.version,
  baseline,
  stableRegistrationBaseline,
  registrationRefusals,
  nativeOwners: receipts.length,
  watcher: watcherBoundaryObservations,
  split: splitWatchLogObservations,
  terminal: terminalStderrObservations,
  rootWarnings: rootWatchWarningObservations,
});
save("role-disposition.json", disposition);
console.log(JSON.stringify(disposition));
