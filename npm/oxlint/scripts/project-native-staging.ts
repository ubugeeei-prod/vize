import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";
import { fileURLToPath } from "node:url";

interface SourcePreload {
  schema: "vize.oxlint.original-sfc-preload";
  wrapper: { path: string; sha256: string };
  binary: { path: string; sha256: string };
}

function authenticateFile(file: SourcePreload["binary"]): string {
  const physical = fs.realpathSync(file.path);
  assert.equal(physical, file.path, "selected source files must use their exact physical paths");
  const descriptor = fs.openSync(physical, "r");
  try {
    const before = fs.fstatSync(descriptor, { bigint: true });
    assert.ok(before.isFile());
    const digest = createHash("sha256");
    const chunk = Buffer.allocUnsafe(64 * 1024);
    let bytes = 0n;
    for (;;) {
      const length = fs.readSync(descriptor, chunk, 0, chunk.length, null);
      if (length === 0) break;
      bytes += BigInt(length);
      assert.ok(bytes <= before.size);
      digest.update(chunk.subarray(0, length));
    }
    assert.equal(bytes, before.size);
    const after = fs.fstatSync(descriptor, { bigint: true });
    const current = fs.lstatSync(physical, { bigint: true });
    assert.ok(current.isFile());
    for (const key of ["dev", "ino", "mode", "size", "mtimeNs", "ctimeNs"] as const) {
      assert.equal(after[key], before[key], "source file changed during authentication");
      assert.equal(current[key], after[key], "source path changed during authentication");
    }
    assert.equal(digest.digest("hex"), file.sha256);
    return physical;
  } finally {
    fs.closeSync(descriptor);
  }
}

/** Preload only Vize: a global NAPI override would also hijack Oxlint's addon. */
export function preloadProjectNative(custody: SourcePreload): void {
  assert.equal(custody.schema, "vize.oxlint.original-sfc-preload");
  assert.equal(process.env.NAPI_RS_NATIVE_LIBRARY_PATH, undefined);
  assert.equal(process.env.NAPI_RS_FORCE_WASI, undefined);
  const expected = fs.realpathSync(
    fileURLToPath(new URL("../../native/index.js", import.meta.url)),
  );
  assert.equal(
    custody.wrapper.path,
    expected,
    "only this source tree's native wrapper may preload",
  );
  const wrapper = authenticateFile(custody.wrapper);
  const binary = authenticateFile(custody.binary);
  const require = createRequire(import.meta.url);
  try {
    process.env.NAPI_RS_NATIVE_LIBRARY_PATH = binary;
    const binding = require(wrapper);
    assert.ok(require.cache[binary], "the selected physical image must actually load");
    assert.equal(
      require.cache[binary]?.exports,
      binding,
      "the wrapper must expose that exact image",
    );
  } finally {
    // All subsequent Oxlint imports, handshakes and subprocesses see no override.
    delete process.env.NAPI_RS_NATIVE_LIBRARY_PATH;
  }
}

if (process.env.VIZE_OXLINT_NATIVE_CUSTODY) {
  preloadProjectNative(
    JSON.parse(fs.readFileSync(process.env.VIZE_OXLINT_NATIVE_CUSTODY, "utf8")) as SourcePreload,
  );
}

export function stageProjectNative(
  artifactRoot: string,
  sourceBinary: string,
  sourceReceipt: string,
  expectedSha256: string,
): { artifacts: string; binary: string } {
  fs.mkdirSync(artifactRoot, { recursive: true });
  // Sticky target caches can contain prior evidence. Never reuse or remove it.
  const artifacts = fs.mkdtempSync(path.join(artifactRoot, "run-"));
  const binary = path.join(artifacts, "source-native.node");
  fs.copyFileSync(sourceBinary, binary, fs.constants.COPYFILE_EXCL);
  assert.equal(createHash("sha256").update(fs.readFileSync(binary)).digest("hex"), expectedSha256);
  fs.copyFileSync(
    sourceReceipt,
    path.join(artifacts, "build-receipt.json"),
    fs.constants.COPYFILE_EXCL,
  );
  // Keep this invocation's complete or failed evidence for the existing upload.
  return { artifacts, binary };
}

export function cleanupProjectNative(artifactRoot: string, ownedDirectory: string): void {
  const artifacts = path.resolve(ownedDirectory);
  assert.equal(path.dirname(artifacts), path.resolve(artifactRoot));
  assert.match(path.basename(artifacts), /^run-[A-Za-z0-9]{6}$/u);
  const entry = fs.lstatSync(artifacts);
  assert.ok(entry.isDirectory() && !entry.isSymbolicLink());
  assert.equal(path.dirname(fs.realpathSync(artifacts)), fs.realpathSync(artifactRoot));
  assert.ok(fs.statSync(path.join(artifacts, "qualification.json")).isFile());
  fs.rmSync(artifacts, { recursive: true });
}

if (process.argv[2] === "--cleanup") {
  const ownedDirectory = process.env.VIZE_OXLINT_STAGING_DIRECTORY;
  assert.ok(ownedDirectory, "only this successful invocation's output may be cleaned");
  cleanupProjectNative(
    fileURLToPath(new URL("../../../target/oxlint-original-project-transport", import.meta.url)),
    ownedDirectory,
  );
}
