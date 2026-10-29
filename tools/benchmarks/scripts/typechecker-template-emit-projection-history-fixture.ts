import assert from "node:assert/strict";
import { mkdirSync, readFileSync, readdirSync, statSync, writeFileSync } from "node:fs";
import { basename, dirname, join, relative, resolve, sep } from "node:path";
import { gunzipSync, gzipSync } from "node:zlib";
import { projectionCoordinateProof } from "./typechecker-template-emit-projection-history-coordinates.ts";
import { projectionKinds } from "./typechecker-template-emit-projection-history-model.ts";
import type {
  BuildRetainedProjectionFixtureOptions,
  ProjectionEditScript,
  ProjectionKind,
  RetainedPacketManifest,
  RetainedPacketRow,
  RetainedProjectionFixture,
  RetainedProjectionManifest,
  RetainedProjectionPackets,
} from "./typechecker-template-emit-projection-history-model.ts";
import {
  diffGeneratedBytes,
  retainedFamilies,
  retainedManifestSha256,
  retainedManifestUrl,
  sha256,
} from "./typechecker-template-emit-projection-history-shared.ts";

interface RetainedPayload {
  schema: number;
  blobs: Record<string, string>;
}

type PacketRepresentation = {
  representation: NonNullable<RetainedPacketRow["representation"]>;
  source?: string;
};

function walk(
  root: string,
  directory: string,
  output: RetainedPacketRow[] = [],
): RetainedPacketRow[] {
  for (const entry of readdirSync(directory, { withFileTypes: true }).sort((a, b) =>
    a.name.localeCompare(b.name, "en"),
  )) {
    const path = join(directory, entry.name);
    if (entry.isDirectory()) walk(root, path, output);
    else {
      assert(entry.isFile(), `unsupported retained packet member: ${path}`);
      const bytes = readFileSync(path);
      const stat = statSync(path);
      output.push({
        path: relative(root, path).split(sep).join("/"),
        bytes: bytes.length,
        mode: stat.mode & 0o777,
        sha256: sha256(bytes),
      });
    }
  }
  return output;
}

function packet(root: string): RetainedPacketRow[] {
  const files = walk(root, root).filter(({ path }) => path !== "receipts.json");
  files.sort((left, right) => left.path.localeCompare(right.path, "en"));
  return files;
}

function putBlob(blobs: Map<string, string>, bytes: Buffer): void {
  const digest = sha256(bytes);
  const encoded = bytes.toString("base64");
  const previous = blobs.get(digest);
  if (previous === undefined) blobs.set(digest, encoded);
  else assert.equal(previous, encoded, "SHA-256 collision in retained packet");
}

export function probeBytes(original: number, virtual: number): Buffer {
  return Buffer.from(
    Array.from({ length: original + 1 }, (_value, index) => `o${index}>${index};`).join("") +
      Array.from({ length: virtual + 1 }, (_value, index) => `v${index}>${index};`).join(""),
  );
}

function representation(
  row: RetainedPacketRow,
  root: string,
  rows: readonly RetainedPacketRow[],
): PacketRepresentation {
  if (row.bytes === 0) return { representation: "empty" };
  if (row.path.endsWith("/canon/import-probes.txt")) {
    const prefix = row.path.slice(0, -"import-probes.txt".length);
    const original = rows.find(({ path }) => path === `${prefix}pre-rewrite-text.ts`);
    const virtual = rows.find(({ path }) => path === `${prefix}text.ts`);
    assert(original && virtual, `missing import-probe source for ${row.path}`);
    const derived = probeBytes(original.bytes, virtual.bytes);
    if (derived.length === row.bytes && sha256(derived) === row.sha256)
      return { representation: "derived-import-probes" };
  }
  if (row.path.endsWith("/canon/pre-rewrite-text.ts")) {
    const source = row.path.replace("pre-rewrite-text.ts", "text.ts");
    const sourceRow = rows.find((candidate) => candidate.path === source);
    assert(sourceRow, `missing canon text for ${row.path}`);
    if (row.sha256 === sourceRow.sha256) return { representation: "same-as", source };
  }
  const bytes = readFileSync(join(root, row.path));
  assert.equal(sha256(bytes), row.sha256);
  return { representation: "inline" };
}

/** Build the minimal content-addressed subset from two already verified artifacts. */
export function buildRetainedProjectionFixture({
  baselineRoot,
  incidentRoot,
  baselineArtifact,
  incidentArtifact,
  manifestPath,
  payloadPath,
}: BuildRetainedProjectionFixtureOptions): RetainedProjectionManifest {
  baselineRoot = resolve(baselineRoot);
  incidentRoot = resolve(incidentRoot);
  manifestPath = resolve(manifestPath);
  payloadPath = resolve(payloadPath);
  const packets: Record<ProjectionKind, RetainedPacketManifest> = {
    baseline: { files: packet(baselineRoot) },
    incident: { files: packet(incidentRoot) },
  };
  const packetBytes: RetainedProjectionPackets = {
    baseline: new Map(
      packets.baseline.files.map((row) => [row.path, readFileSync(join(baselineRoot, row.path))]),
    ),
    incident: new Map(
      packets.incident.files.map((row) => [row.path, readFileSync(join(incidentRoot, row.path))]),
    ),
  };
  assert.deepEqual(
    packets.baseline.files.map((row) => row.path),
    packets.incident.files.map((row) => row.path),
    "retained packet membership changed",
  );
  const blobs = new Map<string, string>();
  for (const [kind, root] of [
    ["baseline", baselineRoot],
    ["incident", incidentRoot],
  ] as const) {
    packets[kind].files = packets[kind].files.map((row) => ({
      ...row,
      ...representation(row, root, packets[kind].files),
    }));
    for (const row of packets[kind].files) {
      const bytes = readFileSync(join(root, row.path));
      assert.equal(bytes.length, row.bytes);
      assert.equal(sha256(bytes), row.sha256);
      if (row.representation === "inline") putBlob(blobs, bytes);
    }
  }
  const editScripts: ProjectionEditScript[] = [];
  for (const family of retainedFamilies) {
    for (const [surface, path] of [
      ["canon", `${family}/canon/text.ts`],
      ["content-mapper", `${family}/content-mapper/text.ts`],
      ["maestro", `${family}/maestro-document-0/text.ts`],
    ] as const) {
      const before = readFileSync(join(baselineRoot, path));
      const after = readFileSync(join(incidentRoot, path));
      editScripts.push({
        id: `${family}/${surface}`,
        family,
        surface,
        path,
        beforeBytes: before.length,
        beforeSha256: sha256(before),
        afterBytes: after.length,
        afterSha256: sha256(after),
        edits: diffGeneratedBytes(before, after),
      });
    }
  }
  const payload: RetainedPayload = {
    schema: 1,
    blobs: Object.fromEntries(
      [...blobs].sort(([left], [right]) => left.localeCompare(right, "en")),
    ),
  };
  const uncompressed = Buffer.from(`${JSON.stringify(payload)}\n`);
  const gzipOptions = { level: 9, mtime: 0 };
  const compressed = gzipSync(uncompressed, gzipOptions);
  const manifest: RetainedProjectionManifest = {
    schema: 1,
    kind: "vize.template-emit-projection-history",
    families: retainedFamilies,
    artifacts: { baseline: baselineArtifact, incident: incidentArtifact },
    packets,
    editScripts,
    coordinateProof: projectionCoordinateProof({ editScripts }, packetBytes),
    payload: {
      file: basename(payloadPath),
      compression: "gzip",
      bytes: compressed.length,
      sha256: sha256(compressed),
      uncompressedBytes: uncompressed.length,
      uncompressedSha256: sha256(uncompressed),
      blobCount: blobs.size,
    },
  };
  mkdirSync(dirname(manifestPath), { recursive: true });
  writeFileSync(payloadPath, compressed);
  writeFileSync(manifestPath, `${JSON.stringify(manifest, null, 2)}\n`);
  return manifest;
}

/** Load and authenticate every retained byte before semantic qualification. */
export function loadRetainedProjectionFixture(
  manifestUrl: URL = retainedManifestUrl,
): RetainedProjectionFixture {
  const manifestBytes = readFileSync(manifestUrl);
  if (retainedManifestSha256 !== "REGENERATE")
    assert.equal(sha256(manifestBytes), retainedManifestSha256, "retained manifest custody drift");
  const manifest = JSON.parse(manifestBytes.toString("utf8")) as RetainedProjectionManifest;
  assert.equal(manifest.schema, 1);
  assert.equal(manifest.kind, "vize.template-emit-projection-history");
  assert.deepEqual(manifest.families, retainedFamilies);
  const payloadUrl = new URL(manifest.payload.file, manifestUrl);
  const compressed = readFileSync(payloadUrl);
  assert.equal(compressed.length, manifest.payload.bytes);
  assert.equal(sha256(compressed), manifest.payload.sha256);
  assert.equal(manifest.payload.compression, "gzip");
  const uncompressed = gunzipSync(compressed);
  assert.equal(uncompressed.length, manifest.payload.uncompressedBytes);
  assert.equal(sha256(uncompressed), manifest.payload.uncompressedSha256);
  const payload = JSON.parse(uncompressed.toString("utf8")) as RetainedPayload;
  assert.equal(payload.schema, 1);
  assert.equal(Object.keys(payload.blobs).length, manifest.payload.blobCount);
  const packets: RetainedProjectionPackets = {
    baseline: new Map<string, Buffer>(),
    incident: new Map<string, Buffer>(),
  };
  for (const kind of projectionKinds) {
    const files = new Map<string, Buffer>();
    const rows = new Map<string, RetainedPacketRow>();
    let previous = "";
    for (const row of manifest.packets[kind].files) {
      assert(row.path > previous, `unsorted ${kind} packet`);
      previous = row.path;
      assert.equal(row.mode, 0o644);
      assert(!rows.has(row.path), `duplicate ${kind} packet path: ${row.path}`);
      rows.set(row.path, row);
      let bytes: Buffer | undefined;
      if (row.representation === "inline") {
        const encoded = payload.blobs[row.sha256];
        assert.equal(typeof encoded, "string", `missing retained blob: ${row.sha256}`);
        bytes = Buffer.from(encoded, "base64");
      } else if (row.representation === "empty") bytes = Buffer.alloc(0);
      else if (
        !row.representation ||
        !["same-as", "derived-import-probes"].includes(row.representation)
      )
        assert.fail(`unknown retained representation: ${row.representation}`);
      if (bytes) files.set(row.path, bytes);
    }
    for (const row of rows.values()) {
      if (row.representation === "same-as") {
        const sourcePath = row.source;
        assert(typeof sourcePath === "string");
        const source = files.get(sourcePath);
        assert(source, `missing same-as source: ${sourcePath}`);
        files.set(row.path, source);
      }
    }
    for (const row of rows.values()) {
      if (row.representation === "derived-import-probes") {
        const prefix = row.path.slice(0, -"import-probes.txt".length);
        const original = files.get(`${prefix}pre-rewrite-text.ts`);
        const virtual = files.get(`${prefix}text.ts`);
        assert(original && virtual, `missing import-probe source: ${row.path}`);
        files.set(row.path, probeBytes(original.length, virtual.length));
      }
    }
    for (const row of rows.values()) {
      const bytes = files.get(row.path);
      assert(bytes, `unmaterialized retained path: ${row.path}`);
      assert.equal(bytes.length, row.bytes, row.path);
      assert.equal(sha256(bytes), row.sha256, row.path);
    }
    packets[kind] = files;
  }
  assert.deepEqual([...packets.baseline.keys()], [...packets.incident.keys()]);
  return { manifest, packets };
}
