/**
 * Content identity of the binaries a benchmark measures (#3283).
 *
 * A version string is not an identity. Two builds of the same commit, a
 * rebuild that landed mid-run, or a binary another process replaced under a
 * shared CARGO_TARGET_DIR all answer `--version` identically, so an artifact
 * that records only versions cannot be reproduced and cannot even prove it
 * measured one binary throughout. That is precisely the attribution failure
 * this issue exists to prevent, so the gate records a sha256 and re-checks it.
 */

import { createHash } from "node:crypto";
import { copyFileSync, chmodSync, existsSync, mkdirSync, readFileSync } from "node:fs";
import { basename, join } from "node:path";

/** sha256 of a file's contents, or null when it cannot be read. */
export function fileSha256(filePath) {
  if (!filePath || !existsSync(filePath)) return null;
  try {
    return createHash("sha256").update(readFileSync(filePath)).digest("hex");
  } catch {
    return null;
  }
}

/**
 * Copy an executable into the benchmark's own work root and return the copy,
 * so a rebuild of the source path cannot change what is being measured
 * halfway through a run.
 *
 * Only self-contained executables may be pinned this way. A launcher shim
 * (`node_modules/.bin/tsgo` is a shell script that resolves its package
 * relative to its own location) breaks when moved, so those are hashed in
 * place instead — see `hashInPlace`.
 */
export function pinExecutable(sourcePath, workRoot) {
  const sha256 = fileSha256(sourcePath);
  if (sha256 == null) {
    throw new Error(`benchmark-binary: cannot hash ${sourcePath}`);
  }
  const pinDir = join(workRoot, "pinned-binaries");
  mkdirSync(pinDir, { recursive: true });
  const measuredPath = join(pinDir, `${basename(sourcePath)}-${sha256.slice(0, 16)}`);
  if (!existsSync(measuredPath)) {
    copyFileSync(sourcePath, measuredPath);
    chmodSync(measuredPath, 0o755);
  }
  return { source: sourcePath, measuredPath, sha256, pinned: true };
}

/** Record identity without moving the file, for launcher shims. */
export function hashInPlace(sourcePath) {
  return {
    source: sourcePath,
    measuredPath: sourcePath,
    sha256: fileSha256(sourcePath),
    pinned: false,
  };
}

/**
 * Fail closed when a measured binary changed while the run was in progress.
 * A timing attributed to a binary that no longer exists is worse than no
 * timing, because nothing downstream can tell the difference.
 */
export function assertBinariesUnchanged(binaries) {
  for (const [label, binary] of Object.entries(binaries)) {
    const current = fileSha256(binary.source);
    if (current !== binary.sha256) {
      throw new Error(
        `benchmark-binary: ${label} changed during the run (${binary.sha256} -> ${current ?? "missing"}); refusing to publish a timing`,
      );
    }
  }
}
