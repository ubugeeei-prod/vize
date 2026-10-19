// Opt-in trial: preserve an entire authenticated artifact ZIP with SHA-256 dedup.
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import {
  chmodSync,
  closeSync,
  copyFileSync,
  constants,
  existsSync,
  lstatSync,
  mkdirSync,
  mkdtempSync,
  openSync,
  readdirSync,
  readFileSync,
  rmSync,
  statSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { parseArgs } from "node:util";
import {
  SCHEMA,
  byCodePoint,
  canonicalJson,
  digestFile,
  digestToFile,
  ensure,
  field,
  integer,
  jsonBytes,
  parseManifest,
  safeName,
  text,
  validateManifest,
  verifyTree,
  zipRow,
} from "./artifact-payload-files.ts";
import type { Json, JsonObject, ManifestRow } from "./artifact-payload-files.ts";
import { ZipWriter, readZip, zipEntryChunks } from "./artifact-payload-zip.ts";

export const PAYLOAD = "objects.zip";
export const MANIFEST = "manifest.json";
export const RECEIPT = "receipt.json";

const milliseconds = (start: number): number => Math.round(performance.now() - start);
const readJson = (path: string): unknown => JSON.parse(readFileSync(path, "utf8"));

export async function pack(
  original: string,
  output: string,
  metadata: unknown,
  sourceSha: string,
  driverSha: string,
): Promise<JsonObject> {
  const started = performance.now();
  ensure(!existsSync(output), "payload output must be fresh");
  for (const sha of [sourceSha, driverSha])
    ensure(/^[0-9a-f]{40}$/.test(sha), "invalid source or driver SHA");
  ensure(
    text(field(metadata, "workflow_run"), "head_sha") === sourceSha,
    "original source identity differs",
  );
  const originalDigest = await digestFile(original);
  ensure(
    text(metadata, "digest") === `sha256:${originalDigest}`,
    "original archive digest differs",
  );
  ensure(
    integer(metadata, "size_in_bytes") === statSync(original).size,
    "original archive size differs",
  );
  const infos = readZip(original);
  mkdirSync(output);
  const provenance = {
    schema: SCHEMA,
    sourceSha,
    driverSha,
    artifactId: integer(metadata, "id"),
    originalZipSha256: originalDigest,
  };
  const entries: ManifestRow[] = [];
  const names = new Set<string>();
  const objects = new Map<string, number>();
  let hashingMs = 0;
  let compressionMs = 0;
  const target = new ZipWriter(join(output, PAYLOAD), true);
  const temporary = mkdtempSync(join(tmpdir(), "vize-payload-body-"));
  try {
    const body = join(temporary, "body");
    for (const info of infos) {
      let row = zipRow(info);
      ensure(!names.has(row.path), "duplicate original path");
      names.add(row.path);
      if (row.kind === "file") {
        const hashStart = performance.now();
        const { sha, size } = await digestToFile(
          zipEntryChunks(original, info),
          openSync(body, "w"),
        );
        ensure(size === info.size, "original body size differs");
        hashingMs += performance.now() - hashStart;
        row = { ...row, sha256: sha, bytes: size };
        const known = objects.get(sha);
        if (known === undefined) {
          const compressStart = performance.now();
          await target.add(`objects/${sha}`, body, { method: 8, level: 6 });
          compressionMs += performance.now() - compressStart;
          objects.set(sha, size);
        } else {
          ensure(known === size, "same digest has different size");
        }
      }
      entries.push(row);
    }
  } finally {
    target.close();
    rmSync(temporary, { recursive: true, force: true });
  }
  const manifest: JsonObject = {
    ...provenance,
    entries: [...entries].sort((left, right) => byCodePoint(left.path, right.path)).map(rowJson),
  };
  const manifestBytes = jsonBytes(manifest);
  writeFileSync(join(output, MANIFEST), manifestBytes);
  const payloadPath = join(output, PAYLOAD);
  const receipt: JsonObject = {
    ...provenance,
    originalArtifactName: text(metadata, "name"),
    originalRunId: integer(field(metadata, "workflow_run"), "id"),
    originalZipBytes: statSync(original).size,
    entryCount: entries.length,
    fileCount: entries.filter((row) => row.kind === "file").length,
    rawBytes: entries.reduce((total, row) => total + (row.bytes ?? 0), 0),
    objectCount: objects.size,
    uniqueBodyBytes: [...objects.values()].reduce((total, bytes) => total + bytes, 0),
    payloadZipBytes: statSync(payloadPath).size,
    manifestBytes: manifestBytes.length,
    payloadZipSha256: await digestFile(payloadPath),
    manifestSha256: createHash("sha256").update(manifestBytes).digest("hex"),
    wholeBodyReadAndHashMs: Math.round(hashingMs),
    uniqueObjectCompressionMs: Math.round(compressionMs),
    packMs: milliseconds(started),
    compression: "deflate-6",
    qualification: "transport trial only; original source acceptance is retained separately",
  };
  validateManifest(parseManifest(manifest), receipt);
  writeFileSync(join(output, RECEIPT), jsonBytes(receipt));
  return receipt;
}

function rowJson(row: ManifestRow): JsonObject {
  return Object.fromEntries(
    Object.entries(row).filter(([, value]) => value !== undefined),
  ) as JsonObject;
}

function isRegularFile(path: string): boolean {
  try {
    return lstatSync(path).isFile();
  } catch {
    return false;
  }
}

export async function restore(
  output: string,
  destination: string,
  sourceSha: string,
  driverSha: string,
  artifactId: number | undefined,
): Promise<JsonObject> {
  const started = performance.now();
  ensure(!existsSync(destination), "restore destination must be fresh");
  const inventory = readdirSync(output);
  ensure(
    inventory.length === 3 &&
      [MANIFEST, PAYLOAD, RECEIPT].every((name) => inventory.includes(name)),
    "payload envelope inventory differs",
  );
  for (const name of [PAYLOAD, MANIFEST, RECEIPT])
    ensure(isRegularFile(join(output, name)), "invalid envelope file");
  const receipt = readJson(join(output, RECEIPT));
  ensure(
    field(receipt, "sourceSha") === sourceSha &&
      field(receipt, "driverSha") === driverSha &&
      field(receipt, "artifactId") === artifactId,
    "transport source or driver identity differs",
  );
  ensure(
    (await digestFile(join(output, PAYLOAD))) === field(receipt, "payloadZipSha256"),
    "damaged payload archive",
  );
  ensure(
    (await digestFile(join(output, MANIFEST))) === field(receipt, "manifestSha256"),
    "damaged payload manifest",
  );
  const manifest = parseManifest(readJson(join(output, MANIFEST)));
  const objects = validateManifest(manifest, receipt);
  ensure(
    manifest.originalZipSha256 === field(receipt, "originalZipSha256"),
    "original digest differs",
  );
  const payload = join(output, PAYLOAD);
  const infos = readZip(payload);
  const temporary = mkdtempSync(join(dirname(destination), "vize-payload-restore-"));
  try {
    const listed = new Set(infos.map((info) => info.name));
    ensure(
      infos.length === objects.size &&
        listed.size === objects.size &&
        [...objects.keys()].every((sha) => listed.has(`objects/${sha}`)),
      "missing or unexpected payload object",
    );
    for (const info of infos) {
      const sha = info.name.replace(/^objects\//, "");
      const fd = openSync(
        join(temporary, sha),
        constants.O_WRONLY | constants.O_CREAT | constants.O_EXCL,
      );
      const actual = await digestToFile(zipEntryChunks(payload, info), fd);
      ensure(actual.sha === sha && actual.size === objects.get(sha), "damaged whole object body");
    }
    mkdirSync(destination);
    for (const row of manifest.entries) {
      const path = join(destination, safeName(row.path));
      mkdirSync(dirname(path), { recursive: true });
      if (row.kind === "directory") {
        mkdirSync(path, { recursive: true });
      } else {
        // Copies deliberately restore independent mutable files, including identical bodies.
        copyFileSync(join(temporary, row.sha256 ?? ""), path, constants.COPYFILE_EXCL);
        chmodSync(path, row.mode);
      }
    }
    // Apply directory modes after writing their children.
    for (const row of [...manifest.entries].reverse()) {
      if (row.kind === "directory") chmodSync(join(destination, row.path), row.mode);
    }
  } finally {
    rmSync(temporary, { recursive: true, force: true });
  }
  const verification = await verifyTree(destination, manifest);
  return {
    ...verification,
    restoreAndFullVerifyMs: milliseconds(started),
    sourceSha,
    driverSha,
    artifactId: artifactId ?? null,
  };
}

/** Read-only authenticated artifact download without recording redirect credentials. */
export async function download(destination: string): Promise<JsonObject> {
  const started = performance.now();
  const repository = process.env.GITHUB_REPOSITORY ?? "";
  const artifactId = process.env.PAYLOAD_ARTIFACT_ID ?? "";
  const sourceSha = process.env.PAYLOAD_SOURCE_SHA ?? "";
  ensure(/^[A-Za-z0-9_.-]+\/[A-Za-z0-9_.-]+$/.test(repository), "invalid repository");
  ensure(/^[1-9][0-9]*$/.test(artifactId), "invalid artifact ID");
  ensure(/^[0-9a-f]{40}$/.test(sourceSha), "invalid source SHA");
  ensure(!existsSync(destination), "download destination must be fresh");
  const endpoint = `repos/${repository}/actions/artifacts/${artifactId}`;
  // gh follows cross-host redirects with its normal credential stripping. Never expose
  // stderr, debug output, headers, tokens, or the temporary signed redirect URL.
  const env = { ...process.env, GH_DEBUG: "" };
  const metadataResult = spawnSync("gh", ["api", endpoint], {
    env,
    stdio: ["inherit", "pipe", "pipe"],
    maxBuffer: 64 * 1024 * 1024,
  });
  ensure(metadataResult.status === 0, "artifact metadata request failed");
  const metadata: unknown = JSON.parse(metadataResult.stdout.toString("utf8"));
  const run = field(metadata, "workflow_run");
  ensure(
    !field(metadata, "expired") && field(metadata, "id") === Number(artifactId),
    "artifact unavailable",
  );
  ensure(text(run, "head_sha") === sourceSha, "original source identity differs");
  ensure(
    field(run, "repository_id") === field(run, "head_repository_id"),
    "fork artifact is not admitted",
  );
  ensure(/^sha256:[0-9a-f]{64}$/.test(text(metadata, "digest")), "artifact has no SHA-256 digest");
  mkdirSync(destination);
  writeFileSync(join(destination, "original-metadata.json"), jsonBytes(metadata as Json));
  const archive = join(destination, "original.zip");
  const fd = openSync(archive, "wx");
  let status: number | null;
  try {
    status = spawnSync("gh", ["api", `${endpoint}/zip`], {
      env,
      stdio: ["inherit", fd, "pipe"],
    }).status;
  } finally {
    closeSync(fd);
  }
  ensure(status === 0, "artifact body request failed");
  ensure(
    statSync(archive).size === integer(metadata, "size_in_bytes"),
    "original archive size differs",
  );
  ensure(
    `sha256:${await digestFile(archive)}` === text(metadata, "digest"),
    "original archive digest differs",
  );
  const report: JsonObject = {
    artifactId: Number(artifactId),
    sourceSha,
    originalRunId: integer(run, "id"),
    originalZipBytes: statSync(archive).size,
    downloadAndDigestMs: milliseconds(started),
  };
  writeFileSync(join(destination, "download.json"), jsonBytes(report));
  return report;
}

const usage =
  "usage: artifact-payload.ts download DESTINATION\n" +
  "       artifact-payload.ts {pack,restore,verify} INPUT OUTPUT --source-sha SHA --driver-sha SHA " +
  "[--metadata PATH] [--artifact-id ID] [--report PATH]";

async function main(): Promise<void> {
  const { positionals, values } = parseArgs({
    allowPositionals: true,
    options: {
      metadata: { type: "string" },
      "source-sha": { type: "string" },
      "driver-sha": { type: "string" },
      "artifact-id": { type: "string" },
      report: { type: "string" },
    },
  });
  const [operation, input, output] = positionals;
  let result: JsonObject;
  if (operation === "download" && input !== undefined && positionals.length === 2) {
    result = await download(input);
  } else {
    const sourceSha = values["source-sha"];
    const driverSha = values["driver-sha"];
    if (input === undefined || output === undefined || positionals.length !== 3)
      throw new Error(usage);
    if (sourceSha === undefined || driverSha === undefined) throw new Error(usage);
    const artifactId = values["artifact-id"];
    if (artifactId !== undefined && !/^[+-]?\d+$/.test(artifactId.trim())) throw new Error(usage);
    if (operation === "pack") {
      if (values.metadata === undefined) throw new Error(usage);
      result = await pack(input, output, readJson(values.metadata), sourceSha, driverSha);
    } else if (operation === "restore") {
      result = await restore(
        input,
        output,
        sourceSha,
        driverSha,
        artifactId === undefined ? undefined : Number(artifactId),
      );
    } else if (operation === "verify") {
      result = await verifyTree(input, parseManifest(readJson(output)));
    } else {
      throw new Error(usage);
    }
  }
  if (values.report !== undefined) writeFileSync(values.report, jsonBytes(result));
  console.log(canonicalJson(result));
}

if (import.meta.main) await main();
