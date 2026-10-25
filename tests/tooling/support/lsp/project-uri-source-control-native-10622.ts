import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { createInterface } from "node:readline";
import { fileURLToPath } from "node:url";
import { decodeFrames } from "../../../differential/lsp-wire.ts";
import { beforeAuthority, object, read, sha256 } from "../project-uri-membership-10622.ts";
import { nativeImportRelocation } from "../project-uri-membership-position-10622.ts";

const MAX_FRAME_BYTES = 16 * 1024 * 1024;
type RecordValue = Record<string, unknown>;
type Roots = { workspace: string; nativeProject: string };
export type SourceControlNativeProducer = {
  role: "control" | "candidate";
  sourceRevision: string;
  cliReceiptRoot: string;
};
export type SourceControlNativeDocument = RecordValue & {
  uri: string;
  languageId: string;
  version: number;
  text: string;
};
export type SourceControlNativeCompletionParams = RecordValue & {
  textDocument: RecordValue & { uri: string };
  position: RecordValue & { line: number; character: number };
};
type CompletionRequest = RecordValue & {
  id: number;
  params: SourceControlNativeCompletionParams;
};
type SourceRow = {
  pid: number;
  owner: number;
  sequence: number;
  frameSha256: string;
  path: string;
  uri: string;
  languageId: string;
  version: number;
  sourceSha256: string;
  normalizedSourceSha256: string;
  normalizedUtf8Bytes: number;
  normalizedUtf16Length: number;
};
export type SourceControlNativeExchange = {
  pid: number;
  owner: number;
  roots: Roots;
  document: SourceControlNativeDocument;
  openedAt: number;
  acknowledgedAt: number;
  requestedAt: number;
  respondedAt: number;
  request: CompletionRequest;
  response: RecordValue;
};
type OpenDocument = {
  value: SourceControlNativeDocument;
  openedAt: number;
  acknowledgedAt?: number;
};
type Owner = {
  index: number;
  pid: number;
  role: "lsp" | "api" | "probe";
  spawn: RecordValue;
  spawnedAt: number;
  exit?: RecordValue;
  exitedAt?: number;
  root?: string;
  initialization?: RecordValue;
  initializationResponse?: RecordValue;
  initialized: boolean;
  documents: Map<string, OpenDocument>;
  acknowledgements: Map<number, { uri: string; document: OpenDocument }>;
  completions: Map<number, Omit<SourceControlNativeExchange, "response" | "respondedAt">>;
};
type Stream = {
  pid: number;
  direction: string;
  bytes: number;
  hash: ReturnType<typeof createHash>;
  pending: Buffer;
  frames: Array<{ sha256: string; packet: RecordValue }>;
  decoded: number;
  reported: number;
};
export type SourceControlNativeAudit = {
  roots: Roots;
  main: SourceControlNativeDocument;
  completions: SourceControlNativeCompletionParams[];
  completionExchanges: SourceControlNativeExchange[];
  sources: SourceRow[];
  summary: {
    producer: SourceControlNativeProducer;
    nativeTranscript: { file: string; bytes: number; sha256: string; events: number };
    lifecycle: RecordValue[];
    owners: RecordValue[];
    streams: RecordValue[];
  };
};
function string(value: unknown): string {
  assert.ok(typeof value === "string");
  return value;
}
function integer(value: unknown): number {
  assert.ok(typeof value === "number" && Number.isSafeInteger(value) && value >= 0);
  return value;
}
function bytes(value: unknown): Buffer {
  const encoded = string(value);
  assert.ok(
    encoded.length <= 4 * Math.ceil(MAX_FRAME_BYTES / 3),
    "native frame/chunk exceeds existing wire bound",
  );
  const decoded = Buffer.from(encoded, "base64");
  assert.equal(decoded.toString("base64"), encoded, "native bytes require canonical base64");
  assert.ok(decoded.length <= MAX_FRAME_BYTES, "native frame/chunk exceeds existing wire bound");
  return decoded;
}
function document(value: unknown): SourceControlNativeDocument {
  const d = object(value);
  return {
    ...d,
    uri: string(d.uri),
    languageId: string(d.languageId),
    version: integer(d.version),
    text: string(d.text),
  };
}
function completion(value: RecordValue): CompletionRequest {
  const params = object(value.params),
    target = object(params.textDocument),
    position = object(params.position);
  return {
    ...value,
    id: integer(value.id),
    params: {
      ...params,
      textDocument: { ...target, uri: string(target.uri) },
      position: {
        ...position,
        line: integer(position.line),
        character: integer(position.character),
      },
    },
  };
}
/** Independent source experiment: owner cardinality is observed, never an admission policy. */
export async function auditSourceControlNative(
  file: string,
  workspace: string,
  runtime: string,
  producer: SourceControlNativeProducer,
): Promise<SourceControlNativeAudit> {
  beforeAuthority();
  assert.ok(["control", "candidate"].includes(producer.role));
  assert.match(producer.sourceRevision, /^[a-f0-9]{40}$/u);
  assert.ok(path.isAbsolute(producer.cliReceiptRoot) && path.isAbsolute(workspace));
  const physicalRuntime = fs.realpathSync(runtime),
    runtimeHash = sha256(fs.readFileSync(physicalRuntime));
  const expectedValue: unknown = read("public-0.567.0.native-sources.json");
  assert.ok(Array.isArray(expectedValue));
  const expected = new Map(
    expectedValue.map((row: unknown) => {
      const value = object(row);
      return [string(value.path), value] as const;
    }),
  );
  assert.equal(expected.size, 534);
  const owners = new Map<number, Owner>(),
    streams = new Map<string, Stream>();
  const sources: SourceRow[] = [],
    lifecycle: RecordValue[] = [],
    exchanges: SourceControlNativeExchange[] = [];
  let sequence = 0,
    observedBytes = 0;
  const transcriptHash = createHash("sha256"),
    initialStat = fs.statSync(file, { bigint: true });
  const packet = (owner: Owner, direction: string, value: RecordValue, frameHash: string) => {
    const method = value.method;
    if (direction === "client" && method === "initialize") {
      assert.equal(owner.initialization, undefined, "one initialization per observed native owner");
      const params = object(value.params),
        root = string(params.rootPath);
      assert.ok(path.isAbsolute(root));
      assert.equal(fileURLToPath(string(params.rootUri)), root);
      assert.ok(Array.isArray(params.workspaceFolders) && params.workspaceFolders.length === 1);
      assert.equal(fileURLToPath(string(object(params.workspaceFolders[0]).uri)), root);
      owner.root = root;
      owner.initialization = value;
    }
    if (direction === "server" && owner.initialization && value.id === owner.initialization.id) {
      assert.equal(owner.initializationResponse, undefined);
      assert.equal(value.error, undefined);
      assert.equal(object(object(value.result).capabilities).positionEncoding, "utf-16");
      owner.initializationResponse = value;
    }
    if (direction === "client" && method === "initialized") {
      assert.ok(owner.initializationResponse && !owner.initialized);
      owner.initialized = true;
    }
    if (direction === "client" && method === "textDocument/didOpen") {
      assert.ok(
        owner.initialized && owner.root,
        "native initialization precedes source publication",
      );
      const d = document(object(value.params).textDocument);
      assert.equal(owner.documents.has(d.uri), false, "duplicate native source within one owner");
      const physical = fileURLToPath(d.uri),
        relative = path.relative(owner.root, physical).split(path.sep).join("/");
      const prior = expected.get(relative);
      assert.ok(prior, `unadmitted native source ${relative}`);
      const normalized = nativeImportRelocation(d.text, {
        workspace,
        nativeProject: owner.root,
      }).text;
      assert.equal(d.languageId, prior.languageId);
      assert.equal(d.version, prior.version);
      assert.equal(
        sha256(normalized),
        prior.normalizedSourceSha256,
        `${relative} complete native code`,
      );
      assert.equal(normalized.length, prior.normalizedUtf16Length);
      assert.equal(Buffer.byteLength(normalized), prior.normalizedUtf8Bytes);
      owner.documents.set(d.uri, { value: d, openedAt: sequence });
      sources.push({
        pid: owner.pid,
        owner: owner.index,
        sequence,
        frameSha256: frameHash,
        path: relative,
        uri: d.uri,
        languageId: d.languageId,
        version: d.version,
        sourceSha256: sha256(d.text),
        normalizedSourceSha256: sha256(normalized),
        normalizedUtf8Bytes: Buffer.byteLength(normalized),
        normalizedUtf16Length: normalized.length,
      });
    }
    if (
      direction === "client" &&
      ["textDocument/didChange", "textDocument/didClose"].includes(string(method ?? ""))
    ) {
      assert.fail("the immutable source-control input has no native document edits or closes");
    }
    if (direction === "client" && method === "textDocument/documentSymbol") {
      const uri = string(object(object(value.params).textDocument).uri),
        d = owner.documents.get(uri);
      assert.ok(d, "readiness request belongs to an opened native source");
      const id = integer(value.id);
      assert.equal(owner.acknowledgements.has(id), false);
      owner.acknowledgements.set(id, { uri, document: d });
    }
    if (direction === "client" && method === "textDocument/completion") {
      assert.ok(owner.root);
      const request = completion(value),
        d = owner.documents.get(request.params.textDocument.uri);
      assert.ok(
        d?.acknowledgedAt,
        "completion belongs to this owner's acknowledged full native source",
      );
      assert.ok(d.value.uri.endsWith("/src/c/Comp0.vue.ts"));
      assert.equal(owner.completions.has(request.id), false);
      owner.completions.set(request.id, {
        pid: owner.pid,
        owner: owner.index,
        roots: { workspace, nativeProject: owner.root },
        document: d.value,
        openedAt: d.openedAt,
        acknowledgedAt: d.acknowledgedAt,
        requestedAt: sequence,
        request,
      });
    }
    if (direction === "server" && typeof value.id === "number") {
      const acknowledgement = owner.acknowledgements.get(value.id);
      if (acknowledgement) {
        assert.equal(value.error, undefined);
        assert.ok(Object.hasOwn(value, "result"));
        assert.equal(owner.documents.get(acknowledgement.uri), acknowledgement.document);
        acknowledgement.document.acknowledgedAt = sequence;
        owner.acknowledgements.delete(value.id);
      }
      const query = owner.completions.get(value.id);
      if (query) {
        assert.equal(value.error, undefined);
        assert.ok(Object.hasOwn(value, "result"));
        exchanges.push({ ...query, response: value, respondedAt: sequence });
        owner.completions.delete(value.id);
      }
    }
  };
  const consume = (stream: Stream, owner: Owner, raw: Buffer) => {
    let offset = 0;
    while (offset < raw.length) {
      const room = MAX_FRAME_BYTES - stream.pending.length;
      assert.ok(room > 0, "incomplete native frame exceeds existing wire bound");
      const end = Math.min(raw.length, offset + room);
      stream.pending = Buffer.concat([stream.pending, raw.subarray(offset, end)]);
      offset = end;
      for (;;) {
        const separator = stream.pending.indexOf("\r\n\r\n");
        if (separator < 0) break;
        const lengths = [
          ...stream.pending
            .subarray(0, separator)
            .toString("ascii")
            .matchAll(/^Content-Length: (\d+)$/gim),
        ];
        assert.equal(lengths.length, 1, "one native Content-Length is required");
        const size = Number(lengths[0][1]),
          frameEnd = separator + 4 + size;
        assert.ok(
          Number.isSafeInteger(size) && frameEnd <= MAX_FRAME_BYTES,
          "native frame exceeds existing wire bound",
        );
        if (frameEnd > stream.pending.length) break;
        const frame = stream.pending.subarray(0, frameEnd),
          decoded = decodeFrames(frame).messages;
        assert.equal(decoded.length, 1);
        const value = object(decoded[0]),
          frameHash = sha256(frame);
        stream.frames.push({ sha256: frameHash, packet: value });
        stream.decoded++;
        packet(owner, stream.direction, value, frameHash);
        stream.pending = stream.pending.subarray(frameEnd);
      }
    }
  };
  const input = fs.createReadStream(file);
  input.on("data", (chunk: Buffer) => {
    transcriptHash.update(chunk);
    observedBytes += chunk.length;
  });
  try {
    for await (const line of createInterface({ input, crlfDelay: Infinity })) {
      sequence++;
      const event = object(JSON.parse(line)),
        pid = integer(event.pid),
        kind = string(event.kind);
      if (kind === "spawn") {
        assert.equal(owners.has(pid), false, "native owner IDs cannot be reused");
        assert.equal(
          fs.realpathSync(string(event.binary)),
          physicalRuntime,
          "actual native physical binary",
        );
        assert.equal(event.binarySha256, runtimeHash, "actual native binary SHA");
        assert.ok(
          Array.isArray(event.argv) && event.argv.every((arg: unknown) => typeof arg === "string"),
        );
        assert.ok(!(event.argv.includes("--lsp") && event.argv.includes("--api")));
        const role = event.argv.includes("--lsp")
          ? "lsp"
          : event.argv.includes("--api")
            ? "api"
            : "probe";
        owners.set(pid, {
          index: owners.size,
          pid,
          role,
          spawn: event,
          spawnedAt: sequence,
          initialized: false,
          documents: new Map(),
          acknowledgements: new Map(),
          completions: new Map(),
        });
        lifecycle.push({ sequence, event });
        continue;
      }
      const owner = owners.get(pid);
      assert.ok(owner, "native traffic requires its original observed owner");
      assert.equal(owner.exit, undefined, "native traffic cannot follow owner exit");
      if (kind === "exit") {
        owner.exit = event;
        owner.exitedAt = sequence;
        lifecycle.push({ sequence, event });
        continue;
      }
      assert.ok(["chunk", "frame", "stderr"].includes(kind), `unexpected native event ${kind}`);
      const direction = kind === "stderr" ? "stderr" : string(event.direction);
      assert.ok(["client", "server", "stderr"].includes(direction));
      const key = `${pid}/${direction}`;
      let stream = streams.get(key);
      if (!stream) {
        stream = {
          pid,
          direction,
          bytes: 0,
          hash: createHash("sha256"),
          pending: Buffer.alloc(0),
          frames: [],
          decoded: 0,
          reported: 0,
        };
        streams.set(key, stream);
      }
      if (kind !== "frame") {
        const raw = bytes(event.chunkBase64);
        stream.bytes += raw.length;
        stream.hash.update(raw);
        if (owner.role === "lsp" && direction !== "stderr") consume(stream, owner, raw);
      } else {
        assert.equal(owner.role, "lsp", "framed observations require the native LSP owner");
        const raw = bytes(event.frameBase64),
          indexed = stream.frames.shift();
        assert.ok(indexed, "observer frame requires its preceding raw chunk index");
        assert.equal(
          sha256(raw),
          indexed.sha256,
          "observer frame exactly matches raw native bytes",
        );
        const decoded = decodeFrames(raw).messages;
        assert.equal(decoded.length, 1);
        assert.deepEqual(decoded[0], event.packet, "reported native packet retains every field");
        assert.deepEqual(indexed.packet, event.packet, "raw native packet retains every field");
        stream.reported++;
      }
    }
  } finally {
    input.destroy();
  }
  const finalStat = fs.statSync(file, { bigint: true });
  assert.deepEqual(
    [finalStat.dev, finalStat.ino, finalStat.size, finalStat.mtimeNs],
    [initialStat.dev, initialStat.ino, initialStat.size, initialStat.mtimeNs],
    "closed native capture cannot change during audit",
  );
  assert.equal(BigInt(observedBytes), finalStat.size);
  const lspOwners = [...owners.values()].filter((owner) => owner.role === "lsp");
  assert.ok(lspOwners.length > 0, "actual native LSP traffic is required");
  for (const owner of lspOwners) {
    assert.ok(owner.initialized && owner.initializationResponse && owner.exit);
    assert.equal(owner.exit.code, 0);
    assert.equal(owner.exit.signal, null);
    assert.equal(
      owner.documents.size,
      expected.size,
      "each observed LSP owner receives all original 534 inputs",
    );
    assert.equal(owner.acknowledgements.size, 0);
    assert.equal(owner.completions.size, 0);
    assert.ok([...owner.documents.values()].every((d) => d.acknowledgedAt !== undefined));
    const paths = sources
      .filter((row) => row.pid === owner.pid)
      .map((row) => row.path)
      .sort();
    assert.deepEqual(paths, [...expected.keys()].sort());
  }
  const framed = [...streams.values()].map((stream) => {
    const owner = owners.get(stream.pid);
    assert.ok(owner);
    if (owner.role === "lsp" && stream.direction !== "stderr") {
      assert.equal(stream.pending.length, 0, "complete closed native framing");
      assert.equal(stream.frames.length, 0, "complete native passive frame index");
      assert.equal(stream.decoded, stream.reported);
    }
    return {
      pid: stream.pid,
      role: owner.role,
      direction: stream.direction,
      bytes: stream.bytes,
      sha256: stream.hash.digest("hex"),
      decodedFrames: stream.decoded,
      observerFrames: stream.reported,
    };
  });
  assert.equal(
    exchanges.length,
    2,
    "the original campaign's two actual native completion exchanges",
  );
  exchanges.sort((a, b) => a.requestedAt - b.requestedAt);
  const first = exchanges[0];
  assert.ok(first);
  return {
    roots: first.roots,
    main: first.document,
    completions: exchanges.map((row) => row.request.params),
    completionExchanges: exchanges,
    sources,
    summary: {
      producer,
      nativeTranscript: {
        file,
        bytes: observedBytes,
        sha256: transcriptHash.digest("hex"),
        events: sequence,
      },
      lifecycle,
      owners: [...owners.values()].map((owner) => ({
        owner: owner.index,
        pid: owner.pid,
        role: owner.role,
        spawn: owner.spawn,
        spawnedAt: owner.spawnedAt,
        exit: owner.exit ?? null,
        exitedAt: owner.exitedAt ?? null,
        nativeProject: owner.root ?? null,
        initialization: owner.initialization ?? null,
        initializationResponse: owner.initializationResponse ?? null,
        uniqueSources: owner.documents.size,
      })),
      streams: framed,
    },
  };
}
