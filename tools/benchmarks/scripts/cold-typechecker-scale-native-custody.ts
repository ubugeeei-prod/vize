/** Native attribution artifact bodies and failure-safe process observations. */
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";

export type InputRow = Record<string, unknown>;
export interface NativeImage {
  path: string;
  sha256: string;
  role: string;
}
const fileHash = (file: string) => createHash("sha256").update(fs.readFileSync(file)).digest("hex");
const write = (file: string, value: unknown) =>
  fs.writeFileSync(file, JSON.stringify(value, null, 2) + "\n", { flag: "wx" });

export function assertLinuxX64Image(file: string, expectedSha256: string) {
  const bytes = fs.readFileSync(file);
  assert.deepEqual(
    [...bytes.subarray(0, 6)],
    [127, 69, 76, 70, 2, 1],
    "expected ELF64 little-endian image",
  );
  assert.equal(bytes.readUInt16LE(18), 62, "expected x86-64 ELF machine");
  assert.equal(fileHash(file), expectedSha256, "native image bytes changed");
}

export function retainInputBodies(
  snapshot: { rows: InputRow[] },
  out: string,
  images: NativeImage[],
  role: string,
) {
  const known = new Map<string, NativeImage>();
  for (const image of images) {
    assert(path.isAbsolute(image.path));
    assert.equal(fs.realpathSync(image.path), image.path);
    assert(!known.has(image.path), "duplicate native image identity");
    assertLinuxX64Image(image.path, image.sha256);
    known.set(image.path, image);
  }
  const blobs = path.join(out, "input-blobs");
  if (!fs.existsSync(blobs)) fs.mkdirSync(blobs);
  const index: InputRow[] = [];
  let textBytes = 0;
  for (const row of snapshot.rows) {
    if (row.kind !== "file") continue;
    assert.equal(typeof row.path, "string");
    assert.equal(typeof row.sha256, "string");
    const file = row.path as string,
      digest = row.sha256 as string,
      image = known.get(file);
    if (image) {
      assert.equal(image.sha256, digest);
      index.push({ ...row, body: "metadata-only-native-image", imageRole: image.role });
      continue;
    }
    const bytes = fs.readFileSync(file);
    try {
      assert(!bytes.includes(0), "unlisted binary/NUL body");
      assert(
        !bytes.subarray(0, 4).equals(Buffer.from([127, 69, 76, 70])) &&
          bytes.subarray(0, 2).toString("ascii") !== "MZ",
        "unlisted native image",
      );
      new TextDecoder("utf-8", { fatal: true }).decode(bytes);
    } catch (error) {
      write(path.join(out, role + "-body-refusal.json"), { ...row, reason: String(error) });
      throw error;
    }
    textBytes += bytes.length;
    assert(textBytes <= 512 * 1024 * 1024, "reviewed text-body retention bound exceeded");
    const target = path.join(blobs, digest);
    if (!fs.existsSync(target)) fs.writeFileSync(target, bytes, { flag: "wx" });
    assert.equal(fileHash(target), digest);
    assert.equal(fs.statSync(target).size, row.bytes);
    index.push({ ...row, body: "retained-utf8-text", blob: digest });
  }
  write(path.join(out, role + "-body-index.json"), { textBytes, rows: index });
}

export interface ReplayOutcome {
  stdout: Buffer | null;
  stderr: Buffer | null;
  status: number | null;
  signal: NodeJS.Signals | null;
  pid?: number;
  error?: Error;
}

/** Persist observations before any caller-supplied build-info/seal work can fail. */
export function recordReplayBeforeInspection<T>(
  out: string,
  replay: ReplayOutcome,
  elapsedNs: string,
  inspect: (status: InputRow) => T,
): T {
  return recordNativeBeforeInspection(out, "native-replay", replay, elapsedNs, inspect);
}

export function recordNativeBeforeInspection<T>(
  out: string,
  role: "native-replay" | "native-profile" | "phase-vize" | "vize",
  replay: ReplayOutcome,
  elapsedNs: string,
  inspect: (status: InputRow) => T,
): T {
  fs.writeFileSync(path.join(out, role + ".stdout.bin"), replay.stdout ?? Buffer.alloc(0), {
    flag: "wx",
  });
  fs.writeFileSync(path.join(out, role + ".stderr.bin"), replay.stderr ?? Buffer.alloc(0), {
    flag: "wx",
  });
  const status = {
    status: replay.status,
    signal: replay.signal,
    error: replay.error?.message ?? null,
    invokedPid: replay.pid ?? null,
    elapsedNs,
  };
  write(path.join(out, role + ".status.json"), status);
  return inspect(status);
}
