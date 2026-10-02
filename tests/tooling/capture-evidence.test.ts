import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { spawnSync } from "node:child_process";
import { mkdtempSync, mkdirSync, readFileSync, rmSync, symlinkSync, writeFileSync } from "node:fs";
import os from "node:os";
import path from "node:path";
import test from "node:test";
import { gunzipSync } from "node:zlib";

const exporter = path.resolve("tools/support/dependencies/export-capture-evidence.ts");
const sha256 = (bytes: Uint8Array) => createHash("sha256").update(bytes).digest("hex");

test("opt-in evidence transports all authored bytes and matches its separate gzip artifact", () => {
  const root = mkdtempSync(path.join(os.tmpdir(), "vize-capture-evidence-"));
  try {
    const input = path.join(root, "capture");
    mkdirSync(path.join(input, "nested"), { recursive: true });
    const source = Buffer.from("/* α */\r\n日本語😀\0\xff", "utf8");
    const help = Buffer.from("Usage: actual --script <SCRIPT>\n");
    writeFileSync(path.join(input, "nested", "source.bin"), source);
    writeFileSync(path.join(input, "help.stdout"), help);
    const output = path.join(root, "transport", "evidence.json.gz");
    const observed = spawnSync(process.execPath, [exporter, input, output, "capture-law"], {
      encoding: "utf8",
    });
    assert.equal(observed.status, 0, observed.stderr);
    assert.equal(observed.stderr, "");
    const lines = observed.stdout.trimEnd().split("\n");
    const header = JSON.parse(lines[0].slice("VIZE_CAPTURE_HEADER_V1 ".length));
    const framed = lines.slice(1, -1).map((line, index) => {
      const match = /^VIZE_CAPTURE_CHUNK_V1 (\d+)\/(\d+) ([A-Za-z0-9+/=]+)$/.exec(line);
      assert.notEqual(match, null);
      assert.equal(Number(match![1]), index + 1);
      assert.equal(Number(match![2]), header.chunks);
      return match![3];
    });
    assert.equal(framed.length, header.chunks);
    const compressed = Buffer.from(framed.join(""), "base64");
    assert.deepEqual(compressed, readFileSync(output));
    assert.equal(compressed.length, header.gzipBytes);
    assert.equal(sha256(compressed), header.gzipSha256);
    assert.equal(lines.at(-1), `VIZE_CAPTURE_END_V1 ${header.gzipSha256}`);
    const raw = gunzipSync(compressed);
    assert.equal(raw.length, header.rawBytes);
    assert.equal(sha256(raw), header.rawSha256);
    const pack = JSON.parse(raw.toString("utf8"));
    assert.deepEqual(pack, {
      schema: 1,
      label: "capture-law",
      files: [
        {
          path: "help.stdout",
          bytes: help.length,
          sha256: sha256(help),
          base64: help.toString("base64"),
        },
        {
          path: "nested/source.bin",
          bytes: source.length,
          sha256: sha256(source),
          base64: source.toString("base64"),
        },
      ],
    });
    assert.equal(header.files, 2);
    assert.equal(header.capturedBytes, source.length + help.length);
    assert.deepEqual(readFileSync(path.join(input, "nested", "source.bin")), source);
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});

test("evidence rejects symlinks and an output inside its source before framing", () => {
  const root = mkdtempSync(path.join(os.tmpdir(), "vize-capture-boundary-"));
  try {
    const input = path.join(root, "capture");
    mkdirSync(input);
    writeFileSync(path.join(root, "outside"), "must not read outside\n");
    symlinkSync(path.join(root, "outside"), path.join(input, "linked"));
    for (const output of [path.join(root, "evidence.gz"), path.join(input, "evidence.gz")]) {
      const observed = spawnSync(process.execPath, [exporter, input, output, "capture-law"], {
        encoding: "utf8",
      });
      assert.equal(observed.status, 1);
      assert.equal(observed.stdout, "");
    }
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});

test("evidence rejects too many files without truncating the capture", () => {
  const root = mkdtempSync(path.join(os.tmpdir(), "vize-capture-count-"));
  try {
    for (let index = 0; index < 129; index++)
      writeFileSync(path.join(root, `${index}.txt`), "captured\n");
    const observed = spawnSync(process.execPath, [exporter, root, `${root}.gz`, "capture-law"], {
      encoding: "utf8",
    });
    assert.equal(observed.status, 1);
    assert.equal(observed.stdout, "");
    assert.equal(readFileSync(path.join(root, "128.txt"), "utf8"), "captured\n");
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});
