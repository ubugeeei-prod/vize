#!/usr/bin/env node
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import {
  appendFileSync,
  existsSync,
  lstatSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  rmSync,
  statSync,
  writeFileSync,
} from "node:fs";
import { join, resolve } from "node:path";
import { performance } from "node:perf_hooks";
import { fileURLToPath } from "node:url";
import {
  digest,
  fileManifest,
  manifestName,
  rawPaths,
  verifyReconstruction,
} from "./native-phase-artifact-files.mjs";

const archiveName = "native-phase-evidence.tar.zst";
const receiptName = "receipt.json";
const tar = (args, binary = "tar") => {
  const result = spawnSync(binary, args, {
    encoding: "utf8",
    timeout: 600_000,
    maxBuffer: 8 * 1024 * 1024,
  });
  assert.equal(result.error, undefined, result.error?.message);
  assert.equal(result.status, 0, `native evidence tar failed: ${result.stderr}`);
  return result.stdout;
};
const elapsed = (start) => Math.round(performance.now() - start);

export function reconstruct(output, destination, { tarBinary = "tar" } = {}) {
  const start = performance.now();
  const receipt = JSON.parse(readFileSync(join(output, receiptName), "utf8"));
  assert.equal(receipt.schema, 1);
  assert.match(receipt.sourceSha, /^[0-9a-f]{40}$/);
  assert.match(receipt.driverSha, /^[0-9a-f]{40}$/);
  assert.deepEqual(receipt.rawPaths, rawPaths);
  const archive = join(output, archiveName);
  assert.equal(
    digest(readFileSync(archive)),
    receipt.archiveSha256,
    "damaged native evidence archive",
  );
  assert(!existsSync(destination), "reconstruction destination must be fresh");
  mkdirSync(destination);
  tar(
    ["--extract", "--zstd", "--same-permissions", "--file", archive, "--directory", destination],
    tarBinary,
  );
  const manifestPath = join(destination, manifestName);
  assert(lstatSync(manifestPath).isFile(), "reconstructed manifest must be a regular file");
  const bytes = readFileSync(manifestPath);
  assert.equal(digest(bytes), receipt.manifestSha256, "damaged native evidence manifest");
  const manifest = JSON.parse(bytes);
  assert.equal(manifest.schema, 1);
  assert.equal(manifest.sourceSha, receipt.sourceSha);
  assert.equal(manifest.driverSha, receipt.driverSha);
  assert.equal(manifest.files.length, receipt.fileCount);
  const actual = verifyReconstruction(destination, manifest);
  assert.equal(
    actual.files.reduce((total, file) => total + file.bytes, 0),
    receipt.rawBytes,
  );
  return { fileCount: actual.files.length, milliseconds: elapsed(start) };
}

export function pack(root, output, { sourceSha, driverSha = sourceSha, tarBinary = "tar" } = {}) {
  const start = performance.now();
  root = resolve(root);
  output = resolve(output);
  assert.match(sourceSha ?? "", /^[0-9a-f]{40}$/);
  assert.match(driverSha ?? "", /^[0-9a-f]{40}$/);
  assert(!existsSync(output), "native evidence output must be fresh");
  const version = tar(["--version"], tarBinary).split("\n")[0];
  assert.match(version, /GNU tar/, "native evidence packing requires GNU tar");
  mkdirSync(output);
  const work = mkdtempSync(join(root, "native-phase-artifact-work-"));
  try {
    const manifestStart = performance.now();
    const inventory = fileManifest(root);
    assert(inventory.files.length, "native evidence selection is empty");
    const manifest = { schema: 1, sourceSha, driverSha, files: inventory.files };
    const manifestBytes = Buffer.from(`${JSON.stringify(manifest)}\n`);
    writeFileSync(join(work, manifestName), manifestBytes);
    writeFileSync(
      join(work, "paths.nul"),
      inventory.files.map((file) => `${file.path}\0`).join(""),
    );
    const manifestMs = elapsed(manifestStart);
    const archiveStart = performance.now();
    const archive = join(output, archiveName);
    // Following every selected alias preserves the original ZIP's logical tree.
    // GNU tar emits shared-inode hardlink records, without touching source inodes.
    tar(
      [
        "--create",
        "--dereference",
        "--no-recursion",
        "--null",
        "--verbatim-files-from",
        "--use-compress-program=zstd -T4 -3",
        "--file",
        archive,
        "--directory",
        root,
        "--files-from",
        join(work, "paths.nul"),
        "--directory",
        work,
        manifestName,
      ],
      tarBinary,
    );
    const archiveMs = elapsed(archiveStart);
    const receipt = {
      schema: 1,
      sourceSha,
      driverSha,
      rawPaths,
      tarVersion: version,
      fileCount: manifest.files.length,
      rawBytes: manifest.files.reduce((total, file) => total + file.bytes, 0),
      uniqueInodeBytes: inventory.uniqueBytes,
      archiveBytes: statSync(archive).size,
      archiveSha256: digest(readFileSync(archive)),
      manifestSha256: digest(manifestBytes),
      manifestMs,
      archiveMs,
    };
    writeFileSync(join(output, receiptName), `${JSON.stringify(receipt, null, 2)}\n`);
    const reconstruction = reconstruct(output, join(work, "reconstructed"), { tarBinary });
    receipt.reconstructionMs = reconstruction.milliseconds;
    receipt.totalPackVerifyMs = elapsed(start);
    receipt.reconstructedFileCount = reconstruction.fileCount;
    writeFileSync(join(output, receiptName), `${JSON.stringify(receipt, null, 2)}\n`);
    return receipt;
  } finally {
    rmSync(work, { recursive: true, force: true });
  }
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const [operation, root, output] = process.argv.slice(2);
  assert(
    root && output && ["pack", "reconstruct"].includes(operation),
    "usage: native-phase-artifact.mjs pack ROOT OUTPUT | reconstruct OUTPUT DESTINATION",
  );
  const options = { tarBinary: process.env.VIZE_NATIVE_ARTIFACT_TAR ?? "tar" };
  // A failed pack remains red and still uploads the entire original raw selection.
  if (operation === "pack" && process.env.GITHUB_OUTPUT) {
    const paths = rawPaths.map((path) => `${resolve(root)}/${path}`).join("\n");
    appendFileSync(
      process.env.GITHUB_OUTPUT,
      `raw_paths<<NATIVE_RAW_PATHS\n${paths}\nNATIVE_RAW_PATHS\n`,
    );
  }
  const result =
    operation === "pack"
      ? pack(root, output, {
          ...options,
          sourceSha: process.env.SOURCE_SHA,
          driverSha: process.env.DRIVER_SHA,
        })
      : reconstruct(resolve(root), resolve(output), options);
  console.log(JSON.stringify(result, null, 2));
}
