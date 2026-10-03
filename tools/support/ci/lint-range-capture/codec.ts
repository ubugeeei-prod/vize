import assert from "node:assert/strict";
import { gzipSync, gunzipSync } from "node:zlib";
import { sha256 } from "./common.ts";

export type CaptureHeader = {
  source: string;
  runId: string | null;
  attempt: string | null;
  actualProcessExit: number | null;
  actualSignal: string | null;
  complete: boolean;
  acceptance: false;
  files: number;
};
export function encodeFrames(header: CaptureHeader, files: { path: string; bytes: Buffer }[]) {
  assert.equal(header.files, files.length);
  assert.equal(header.acceptance, false);
  const lines = ["VIZE_LINT_RANGE_HEADER " + JSON.stringify(header)];
  files.forEach((file, index) => {
    const gzip = gzipSync(file.bytes, { level: 9 }),
      base64 = gzip.toString("base64");
    const chunks = Math.ceil(base64.length / 12000);
    lines.push(
      "VIZE_LINT_RANGE_FILE " +
        JSON.stringify({
          index,
          path: file.path,
          bytes: file.bytes.length,
          sha256: sha256(file.bytes),
          gzipBytes: gzip.length,
          gzipSha256: sha256(gzip),
          chunks,
        }),
    );
    for (let chunk = 0; chunk < chunks; chunk++)
      lines.push(
        "VIZE_LINT_RANGE_CHUNK " +
          JSON.stringify({
            file: index,
            index: chunk,
            data: base64.slice(chunk * 12000, (chunk + 1) * 12000),
          }),
      );
  });
  lines.push("VIZE_LINT_RANGE_END " + JSON.stringify({ files: files.length, acceptance: false }));
  return lines;
}
export function decodeFrames(log: string, expectedSource: string) {
  const frames = log
    .split(/\r?\n/u)
    .map((line) => line.replace(/^\uFEFF?(?:\d{4}-\d{2}-\d{2}T\S+Z )?/u, ""))
    .filter((line) => line.startsWith("VIZE_LINT_RANGE_"));
  const take = (label: string) => {
    const line = frames.shift();
    assert.equal(typeof line, "string", "Missing " + label);
    assert.ok(line && line.startsWith(label + " "), "Missing or out-of-order " + label);
    return JSON.parse(line.slice(label.length + 1));
  };
  const header: CaptureHeader = take("VIZE_LINT_RANGE_HEADER");
  assert.equal(header.source, expectedSource);
  assert.equal(header.acceptance, false);
  assert.ok(Number.isSafeInteger(header.files) && header.files >= 0);
  const files = new Map<string, Buffer>();
  for (let index = 0; index < header.files; index++) {
    const row = take("VIZE_LINT_RANGE_FILE");
    assert.equal(row.index, index);
    assert.ok(
      typeof row.path === "string" &&
        !row.path.includes("\\") &&
        !row.path.startsWith("/") &&
        !row.path.split("/").some((p: string) => ["", ".", ".."].includes(p)),
    );
    assert.equal(files.has(row.path), false, "Duplicate original path");
    assert.ok(Number.isSafeInteger(row.chunks) && row.chunks > 0);
    let base64 = "";
    for (let chunk = 0; chunk < row.chunks; chunk++) {
      const part = take("VIZE_LINT_RANGE_CHUNK");
      assert.equal(part.file, index);
      assert.equal(part.index, chunk);
      assert.ok(typeof part.data === "string" && /^[A-Za-z0-9+/]*={0,2}$/u.test(part.data));
      base64 += part.data;
    }
    const gzip = Buffer.from(base64, "base64");
    assert.equal(gzip.toString("base64"), base64);
    assert.equal(gzip.length, row.gzipBytes);
    assert.equal(sha256(gzip), row.gzipSha256);
    const bytes = gunzipSync(gzip);
    assert.equal(bytes.length, row.bytes);
    assert.equal(sha256(bytes), row.sha256);
    files.set(row.path, bytes);
  }
  const end = take("VIZE_LINT_RANGE_END");
  assert.equal(end.files, header.files);
  assert.equal(end.acceptance, false);
  assert.equal(frames.length, 0, "Duplicate or foreign frame stream");
  return { header, files };
}
