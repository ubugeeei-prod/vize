import { createHash, randomUUID } from "node:crypto";
import {
  existsSync,
  closeSync,
  constants,
  fstatSync,
  linkSync,
  lstatSync,
  mkdirSync,
  openSync,
  readFileSync,
  renameSync,
  unlinkSync,
  writeFileSync,
} from "node:fs";
import { join } from "node:path";

export type BundleIdentity = Record<string, unknown>;
export type BundleBuild = { code: string; modules: string[] };
type Receipt = BundleBuild & {
  version: 1;
  identity: BundleIdentity;
  key: string;
  contentSha256: string;
};

export const sha256 = (value: string | Buffer) => createHash("sha256").update(value).digest("hex");
const contentDigest = ({ code, modules }: BundleBuild) => sha256(JSON.stringify({ code, modules }));
class BundleConflict extends Error {}

function readBundle(path: string, identity: BundleIdentity, key: string): Receipt {
  if (!lstatSync(path).isFile()) throw new Error("Runtime bundle storage must be a regular file");
  const fd = openSync(path, constants.O_RDONLY | constants.O_NOFOLLOW);
  let receipt: Receipt;
  try {
    if (!fstatSync(fd).isFile()) throw new Error("Runtime bundle storage must be a regular file");
    receipt = JSON.parse(readFileSync(fd, "utf8")) as Receipt;
  } finally {
    closeSync(fd);
  }
  if (
    receipt.version !== 1 ||
    receipt.key !== key ||
    JSON.stringify(receipt.identity) !== JSON.stringify(identity) ||
    typeof receipt.code !== "string" ||
    !receipt.code.length ||
    !Array.isArray(receipt.modules) ||
    !receipt.modules.length ||
    receipt.modules.some((id) => typeof id !== "string" || !id.length) ||
    new Set(receipt.modules).size !== receipt.modules.length ||
    receipt.contentSha256 !== contentDigest(receipt)
  )
    throw new Error("Runtime bundle receipt or contents mismatch");
  return receipt;
}

function publish(pending: string, path: string, identity: BundleIdentity, receipt: Receipt) {
  try {
    linkSync(pending, path);
    return;
  } catch (error) {
    if ((error as NodeJS.ErrnoException).code !== "EEXIST") throw error;
  }
  let existing: Receipt | null = null;
  try {
    existing = readBundle(path, identity, receipt.key);
  } catch {
    /* Invalid bytes are never used. */
  }
  if (existing) {
    if (existing.contentSha256 !== receipt.contentSha256)
      throw new BundleConflict("Conflicting runtime bundle bytes for identical inputs");
    return;
  }
  const quarantine = `${path}.${randomUUID()}.invalid`;
  try {
    try {
      if (!lstatSync(path).isFile()) throw new Error("Unknown runtime bundle storage");
      renameSync(path, quarantine);
    } catch (error) {
      if ((error as NodeJS.ErrnoException).code !== "ENOENT") throw error;
    }
    const moved = lstatSync(quarantine, { throwIfNoEntry: false });
    if (moved && !moved.isFile()) {
      // A concurrent replacement can race the pre-rename no-follow guard.
      // Preserve unknown storage, restoring its name when still available.
      if (!lstatSync(path, { throwIfNoEntry: false })) renameSync(quarantine, path);
      throw new Error("Unknown runtime bundle storage");
    }
    // A concurrent writer may have repaired the entry after the invalid read.
    try {
      existing = readBundle(quarantine, identity, receipt.key);
    } catch {
      /* Only invalid entries are replaced. */
    }
    if (existing) {
      try {
        linkSync(quarantine, path);
      } catch (error) {
        if ((error as NodeJS.ErrnoException).code !== "EEXIST") throw error;
      }
      if (existing.contentSha256 !== receipt.contentSha256)
        throw new BundleConflict("Conflicting runtime bundle bytes for identical inputs");
    }
    try {
      linkSync(pending, path);
    } catch (error) {
      if ((error as NodeJS.ErrnoException).code !== "EEXIST") throw error;
      if (readBundle(path, identity, receipt.key).contentSha256 !== receipt.contentSha256)
        throw new BundleConflict("Conflicting runtime bundle bytes for identical inputs");
    }
  } finally {
    try {
      if (lstatSync(quarantine).isFile()) unlinkSync(quarantine);
    } catch {
      /* Storage is optional. */
    }
  }
}

/** Job-local immutable bytes only: never stores a DOM, module, render or trace. */
export async function immutableRuntimeBundle({
  directory,
  inputs,
  build,
  canStore = () => true,
}: {
  directory: string;
  inputs: () => BundleIdentity | null;
  build: () => Promise<BundleBuild>;
  canStore?: (bundle: BundleBuild) => boolean;
}): Promise<BundleBuild & { cache: "hit" | "built" | "bypassed" }> {
  const identity = inputs();
  if (identity === null) return { ...(await build()), cache: "bypassed" };
  const serialized = JSON.stringify(identity);
  const key = sha256(serialized);
  const path = join(directory, `${key}.json`);
  try {
    if (existsSync(path)) {
      const cached = readBundle(path, identity, key);
      if (canStore(cached)) return { ...cached, cache: "hit" };
    }
  } catch {
    /* Corrupt or unreadable storage is a miss, never executable input. */
  }
  const result = await build();
  if (
    !result.code ||
    !Array.isArray(result.modules) ||
    !result.modules.length ||
    result.modules.some((id) => typeof id !== "string" || !id.length) ||
    new Set(result.modules).size !== result.modules.length
  )
    throw new Error("Incomplete runtime bundle");
  if (JSON.stringify(inputs()) !== serialized)
    throw new Error("Runtime bundle inputs changed during build");
  if (!canStore(result)) return { ...result, cache: "bypassed" };
  const receipt: Receipt = {
    version: 1,
    identity,
    key,
    ...result,
    contentSha256: contentDigest(result),
  };
  const pending = join(directory, `.${key}-${randomUUID()}.pending`);
  try {
    mkdirSync(directory, { recursive: true });
    writeFileSync(pending, JSON.stringify(receipt), { flag: "wx" });
    publish(pending, path, identity, receipt);
  } catch (error) {
    if (error instanceof BundleConflict) throw error;
    return { ...result, cache: "bypassed" };
  } finally {
    try {
      if (lstatSync(pending).isFile()) unlinkSync(pending);
    } catch {
      /* Storage is optional. */
    }
  }
  return { ...result, cache: "built" };
}
