import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { createInterface } from "node:readline";
import { pipeline } from "node:stream/promises";
import { fileURLToPath } from "node:url";
import { stripVTControlCharacters } from "node:util";
import { createGunzip, createGzip, gunzipSync } from "node:zlib";
import { decodeFrames } from "../../differential/lsp-wire.ts";
import { root } from "./lsp/paths.ts";
import {
  nativeImportRelocation,
  normalizePacket,
  normalizeMemberPacket,
} from "./project-uri-membership-position-10622.ts";

export const fixture = path.join(
  root,
  "tests/_fixtures/differential/lsp/project-uri-membership-10622",
);
export const sha256 = (bytes: string | Uint8Array) =>
  createHash("sha256").update(bytes).digest("hex");
export function object(value: unknown): Record<string, unknown> {
  assert.ok(value && typeof value === "object" && !Array.isArray(value));
  return value as Record<string, unknown>;
}
export const read = (file: string) => JSON.parse(fs.readFileSync(path.join(fixture, file), "utf8"));
export type Roots = { workspace: string; nativeProject: string };
export const normalizeText = (text: string, roots: Roots) =>
  nativeImportRelocation(text, roots).text;
export { normalizePacket as relocate, normalizeMemberPacket as normalizedMemberPacket };

export function beforeAuthority() {
  assert.equal(
    sha256(fs.readFileSync(path.join(fixture, "authority.json"))),
    "d9659038353d4eb8181baa0677ae296ad6ced6d8687d25873df60434eb660395",
    "immutable published packet, native-source and provider authority",
  );
  const authority = read("authority.json");
  for (const [file, digest] of Object.entries(authority.files))
    assert.equal(sha256(fs.readFileSync(path.join(fixture, file))), digest, file);
  const observation = JSON.parse(
    gunzipSync(
      fs.readFileSync(path.join(fixture, "public-0.567.0-observation.json.gz")),
    ).toString(),
  );
  assert.equal(observation.state, "process-closed");
  assert.deepEqual(observation.process, { exitStatus: 0, signal: null, error: null });
  const streams: Record<string, Buffer> = {};
  for (const stream of ["client", "server", "stderr"]) {
    const bytes = gunzipSync(
      fs.readFileSync(path.join(fixture, `public-0.567.0-${stream}.bin.gz`)),
    );
    streams[stream] = bytes;
    assert.equal(observation.streams[stream].capturedBytes, bytes.length);
    assert.equal(observation.streams[stream].observedBytes, bytes.length);
    assert.equal(observation.streams[stream].sha256, sha256(bytes));
    assert.equal(observation.streams[stream].truncated, false);
  }
  const client = decodeFrames(streams.client).messages;
  const server = decodeFrames(streams.server).messages;
  const packets = read("public-0.567.0.packets.json");
  assert.equal(packets.rows.length, 15);
  for (const row of packets.rows) {
    assert.deepEqual(
      client.find((packet) => packet.id === row.response.id),
      {
        jsonrpc: "2.0",
        id: row.response.id,
        method: row.method,
        params: row.params,
      },
    );
    assert.deepEqual(
      server.find((packet) => packet.id === row.response.id),
      row.response,
    );
  }
  assert.deepEqual(server.find((packet) => packet.id === 1)?.result, packets.initialization);
  return { authority, packets, inputs: read("original400.inputs.json"), observation };
}

type NativeDocument = { uri: string; languageId: string; version: number; text: string };
type CompletionQuery = {
  textDocument: { uri: string };
  position: { line: number; character: number };
};
type Stream = {
  pending: Buffer;
  bytes: number;
  hash: ReturnType<typeof createHash>;
  frameHashes: string[];
  reported: number;
};
export type NativeAudit = {
  roots: Roots;
  main: NativeDocument;
  completions: CompletionQuery[];
  summary: unknown;
};

/** Full raw transport is retained; framing is decoded one unchanged bounded frame at a time. */
export async function auditNative(
  file: string,
  workspace: string,
  runtime: string,
  live?: NativeAudit,
): Promise<NativeAudit> {
  const expected = read("public-0.567.0.native-sources.json") as Array<Record<string, unknown>>;
  const expectedByPath = new Map(expected.map((entry) => [entry.path, entry]));
  const seen = new Map<string, unknown>();
  const owners = new Map<
    number,
    { argv: string[]; role: string; binary: string; binarySha256: string }
  >();
  const streams = new Map<string, Stream>();
  const completions: CompletionQuery[] = [];
  let main: NativeDocument | undefined;
  let nativeProject: string | undefined;
  const onPacket = (direction: string, value: unknown) => {
    const packet = object(value);
    if (direction !== "client") return;
    if (packet.method === "textDocument/completion")
      completions.push(object(packet.params) as unknown as CompletionQuery);
    if (packet.method !== "textDocument/didOpen") return;
    const document = object(object(packet.params).textDocument) as unknown as NativeDocument;
    const physical = fileURLToPath(document.uri);
    const marker = `${path.sep}src${path.sep}`;
    const index = physical.lastIndexOf(marker);
    assert.ok(index > 0, "native source belongs to the generated project");
    const selected = physical.slice(0, index);
    nativeProject ??= selected;
    assert.equal(selected, nativeProject, "all 534 source identities belong to one project");
    const relative = path.relative(nativeProject, physical).split(path.sep).join("/");
    assert.equal(seen.has(relative), false, `duplicate native didOpen ${relative}`);
    const prior = expectedByPath.get(relative);
    assert.ok(prior, `unadmitted native source ${relative}`);
    const normalized = normalizeText(document.text, { workspace, nativeProject });
    assert.equal(document.languageId, prior.languageId);
    assert.equal(document.version, prior.version);
    assert.equal(
      sha256(normalized),
      prior.normalizedSourceSha256,
      `${relative} complete native code`,
    );
    assert.equal(normalized.length, prior.normalizedUtf16Length);
    assert.equal(Buffer.byteLength(normalized), prior.normalizedUtf8Bytes);
    seen.set(relative, {
      uri: document.uri,
      sourceSha256: sha256(document.text),
      normalizedSourceSha256: sha256(normalized),
      bytes: Buffer.byteLength(document.text),
    });
    if (relative === "src/c/Comp0.vue.ts") main = document;
  };
  const consume = (stream: Stream, direction: string, chunk: Buffer) => {
    stream.bytes += chunk.length;
    stream.hash.update(chunk);
    stream.pending = Buffer.concat([stream.pending, chunk]);
    for (;;) {
      const separator = stream.pending.indexOf("\r\n\r\n");
      if (separator < 0) return;
      const lengths = [
        ...stream.pending
          .subarray(0, separator)
          .toString("ascii")
          .matchAll(/^Content-Length: (\d+)$/gim),
      ];
      assert.equal(lengths.length, 1);
      const size = Number(lengths[0][1]);
      assert.ok(Number.isSafeInteger(size) && size >= 0);
      const end = separator + 4 + size;
      if (stream.pending.length < end) return;
      const frame = stream.pending.subarray(0, end);
      const decoded = decodeFrames(frame).messages;
      assert.equal(decoded.length, 1);
      stream.frameHashes.push(sha256(frame));
      onPacket(direction, decoded[0]);
      stream.pending = stream.pending.subarray(end);
    }
  };
  const events: unknown[] = [];
  for await (const line of createInterface({
    input: fs.createReadStream(file),
    crlfDelay: Infinity,
  })) {
    const event = JSON.parse(line);
    if (event.kind === "spawn") {
      assert.equal(owners.has(event.pid), false);
      assert.equal(fs.realpathSync(event.binary), fs.realpathSync(runtime));
      assert.equal(event.binarySha256, sha256(fs.readFileSync(runtime)));
      owners.set(event.pid, {
        ...event,
        role: event.argv.includes("--lsp") ? "lsp" : event.argv.includes("--api") ? "api" : "probe",
      });
      events.push(event);
    } else if (event.kind === "exit") events.push(event);
    else if (event.kind === "chunk" || event.kind === "frame") {
      const owner = owners.get(event.pid);
      assert.ok(owner, "native source identity precedes its traffic");
      if (owner.role !== "lsp") continue; // API MsgPack bytes remain in the complete archive.
      assert.ok(["client", "server"].includes(event.direction));
      const key = `${event.pid}/${event.direction}`;
      let stream = streams.get(key);
      if (!stream) {
        stream = {
          pending: Buffer.alloc(0),
          bytes: 0,
          hash: createHash("sha256"),
          frameHashes: [],
          reported: 0,
        };
        streams.set(key, stream);
      }
      if (event.kind === "chunk")
        consume(stream, event.direction, Buffer.from(event.chunkBase64, "base64"));
      else {
        const frame = Buffer.from(event.frameBase64, "base64");
        assert.equal(sha256(frame), stream.frameHashes[stream.reported++]);
        assert.deepEqual(decodeFrames(frame).messages, [event.packet]);
      }
    }
  }
  assert.equal([...owners.values()].filter((owner) => owner.role === "lsp").length, 1);
  const framed = [...streams].map(([stream, value]) => {
    assert.equal(value.pending.length, 0, `${stream} complete framing`);
    assert.equal(
      value.frameHashes.length,
      value.reported,
      `${stream} complete passive frame index`,
    );
    return { stream, bytes: value.bytes, sha256: value.hash.digest("hex"), frames: value.reported };
  });
  assert.equal(seen.size, 534);
  assert.deepEqual(
    [...seen.keys()].sort((a, b) => a.localeCompare(b)),
    [...expectedByPath.keys()].sort((a, b) => a.localeCompare(b)),
  );
  assert.ok(nativeProject && main);
  assert.equal(completions.length, 2);
  if (live) {
    assert.deepEqual({ workspace, nativeProject }, live.roots);
    assert.deepEqual(main, live.main, "closed capture retains the exact live-owned document");
  } else {
    assert.equal(fs.realpathSync(nativeProject), nativeProject);
    assert.ok(fs.statSync(nativeProject).isDirectory());
    const sessionRoot = path.resolve(nativeProject, "../..");
    assert.match(path.basename(sessionRoot), /^session-/u);
    assert.equal(fs.statSync(sessionRoot).mode & 0o777, 0o700);
  }
  return {
    roots: { workspace, nativeProject },
    main,
    completions,
    summary: {
      authority:
        "actual original 534 native didOpen sources; complete LSP framing, full API/LSP raw byte archive",
      owners: events,
      framed,
      sources: Object.fromEntries(seen),
    },
  };
}

export function membershipCounters(stderr: string) {
  const rows = stripVTControlCharacters(stderr)
    .split("\n")
    .filter((line) => line.includes('phase="canonical_uri_membership"'))
    .map((line) => {
      const stage = /stage="([a-z_]+)"/u.exec(line)?.[1];
      assert.ok(
        stage && ["merge_dependencies", "merge_materialized", "project_filter"].includes(stage),
      );
      const row: Record<string, string | number> = { stage };
      for (const name of [
        "entries",
        "parses",
        "probes",
        "physical_resolutions",
        "legacy_comparisons",
        "physical_resets",
        "primary_comparisons",
      ]) {
        const value = new RegExp(`\\b${name}=([0-9]+)\\b`, "u").exec(line)?.[1];
        if (name === "primary_comparisons" && stage !== "merge_dependencies") continue;
        assert.ok(value !== undefined, `${name}: ${line}`);
        row[name] = Number(value);
      }
      assert.equal(row.legacy_comparisons, 0, "the real file-URI corpus uses the proven index");
      assert.ok(Number(row.parses) <= Number(row.entries) + Number(row.probes));
      assert.ok(
        Number(row.physical_resolutions) <=
          Number(row.probes) + Number(row.entries) * Number(row.physical_resets),
      );
      return row;
    });
  assert.ok(rows.length > 0, "actual source membership counters are required");
  for (const stage of ["merge_dependencies", "merge_materialized", "project_filter"])
    assert.ok(
      rows.some((row) => row.stage === stage && Number(row.probes) > 0),
      stage,
    );
  return rows;
}

/** Preserve the full large side channel losslessly without duplicating it in artifacts. */
export async function archiveNative(file: string) {
  const hash = createHash("sha256");
  for await (const chunk of fs.createReadStream(file)) hash.update(chunk);
  const receipt = {
    bytes: fs.statSync(file).size,
    sha256: hash.digest("hex"),
    archive: `${path.basename(file)}.gz`,
  };
  await pipeline(fs.createReadStream(file), createGzip(), fs.createWriteStream(`${file}.gz`));
  const restored = createHash("sha256");
  let bytes = 0;
  for await (const chunk of fs.createReadStream(`${file}.gz`).pipe(createGunzip())) {
    bytes += chunk.length;
    restored.update(chunk);
  }
  assert.equal(bytes, receipt.bytes);
  assert.equal(restored.digest("hex"), receipt.sha256);
  fs.unlinkSync(file);
  return receipt;
}
