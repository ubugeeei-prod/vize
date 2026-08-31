import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import {
  assertBinariesUnchanged,
  fileSha256,
  hashInPlace,
  pinExecutable,
} from "../../tools/benchmarks/scripts/benchmark-binary.mjs";

function withTempDir<T>(run: (dir: string) => T): T {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-bench-binary-"));
  try {
    return run(dir);
  } finally {
    fs.rmSync(dir, { force: true, recursive: true });
  }
}

function writeBinary(dir: string, name: string, body: string): string {
  const file = path.join(dir, name);
  fs.writeFileSync(file, body);
  fs.chmodSync(file, 0o755);
  return file;
}

function expectedSha(body: string): string {
  return createHash("sha256").update(Buffer.from(body)).digest("hex");
}

test("fileSha256 hashes contents and reports null for anything unreadable", () => {
  withTempDir((dir) => {
    const body = "#!/bin/sh\necho vize\n";
    assert.equal(fileSha256(writeBinary(dir, "vize", body)), expectedSha(body));
    assert.equal(fileSha256(path.join(dir, "absent")), null);
    assert.equal(fileSha256(null), null);
    assert.equal(fileSha256(""), null);
  });
});

test("pinExecutable measures a private copy named by its content hash", () => {
  withTempDir((dir) => {
    const body = "#!/bin/sh\necho vize\n";
    const source = writeBinary(dir, "vize", body);
    const workRoot = path.join(dir, "work");
    const sha256 = expectedSha(body);

    const pinned = pinExecutable(source, workRoot);
    assert.deepEqual(pinned, {
      source,
      measuredPath: path.join(workRoot, "pinned-binaries", `vize-${sha256.slice(0, 16)}`),
      sha256,
      pinned: true,
    });
    assert.equal(fs.readFileSync(pinned.measuredPath, "utf8"), body);
    assert.equal(fs.statSync(pinned.measuredPath).mode & 0o111, 0o111);

    // Rebuilding the source does not change the copy already being measured.
    fs.writeFileSync(source, "#!/bin/sh\necho rebuilt\n");
    assert.equal(fs.readFileSync(pinned.measuredPath, "utf8"), body);
  });
});

test("pinExecutable refuses a source it cannot hash", () => {
  withTempDir((dir) => {
    assert.throws(() => pinExecutable(path.join(dir, "absent"), path.join(dir, "work")), {
      name: "Error",
      message: `benchmark-binary: cannot hash ${path.join(dir, "absent")}`,
    });
  });
});

test("hashInPlace records identity without moving a launcher shim", () => {
  withTempDir((dir) => {
    const body = '#!/bin/sh\nexec "$(dirname "$0")/../pkg/tsgo" "$@"\n';
    const source = writeBinary(dir, "tsgo", body);

    assert.deepEqual(hashInPlace(source), {
      source,
      measuredPath: source,
      sha256: expectedSha(body),
      pinned: false,
    });
  });
});

test("a binary replaced during the run fails the gate closed", () => {
  withTempDir((dir) => {
    const source = writeBinary(dir, "vize", "#!/bin/sh\necho one\n");
    const before = expectedSha("#!/bin/sh\necho one\n");
    const binaries = { vize: hashInPlace(source) };

    assert.equal(assertBinariesUnchanged(binaries), undefined);

    fs.writeFileSync(source, "#!/bin/sh\necho two\n");
    const after = expectedSha("#!/bin/sh\necho two\n");
    assert.throws(() => assertBinariesUnchanged(binaries), {
      name: "Error",
      message: `benchmark-binary: vize changed during the run (${before} -> ${after}); refusing to publish a timing`,
    });

    fs.rmSync(source);
    assert.throws(() => assertBinariesUnchanged(binaries), {
      name: "Error",
      message: `benchmark-binary: vize changed during the run (${before} -> missing); refusing to publish a timing`,
    });
  });
});
