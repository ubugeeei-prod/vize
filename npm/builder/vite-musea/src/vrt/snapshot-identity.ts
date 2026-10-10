import path from "node:path";
import type { ArtFileInfo, ViewportConfig } from "../types/index.js";

export interface SnapshotIdentityJob {
  art: Pick<ArtFileInfo, "path">;
  variantName: string;
  viewport: ViewportConfig;
}

export type SnapshotIdentities = Record<string, string>;

/** Validate a portable project-relative identity without changing its meaning. */
export function normalizeSnapshotIdentity(relative: string): string {
  if (
    typeof relative !== "string" ||
    !relative ||
    relative.includes("\\") ||
    relative.includes("\0") ||
    path.posix.isAbsolute(relative) ||
    path.win32.isAbsolute(relative) ||
    /^[a-z]:/i.test(relative) ||
    relative.split("/").some((part) => !part || part === "." || part === "..")
  ) {
    throw new Error(`Invalid project-relative snapshot identity: ${JSON.stringify(relative)}`);
  }
  return relative;
}

/** A hosted manifest supplies identities explicitly; it cannot fall back to local paths. */
export function resolveSnapshotIdentity(
  artPath: string,
  projectRoot = process.cwd(),
  identities?: SnapshotIdentities,
): string {
  if (identities !== undefined) {
    if (!Object.hasOwn(identities, artPath))
      throw new Error(`Snapshot identity missing for Art: ${artPath}`);
    return normalizeSnapshotIdentity(identities[artPath]);
  }
  const root = path.resolve(projectRoot);
  const absolute = path.resolve(root, artPath);
  const relative = path.relative(root, absolute).split(path.sep).join("/");
  try {
    return normalizeSnapshotIdentity(relative);
  } catch {
    throw new Error(`Art is outside its snapshot projectRoot: ${artPath}. Configure projectRoot.`);
  }
}

/** The viewport dimensions and scale remain part of identity even when it has a name. */
export function buildCaptureIdentity(
  job: SnapshotIdentityJob,
  projectRoot = process.cwd(),
  identities?: SnapshotIdentities,
): string {
  const { width, height, deviceScaleFactor = 1, name = "" } = job.viewport;
  if (
    typeof job.variantName !== "string" ||
    typeof name !== "string" ||
    !Number.isSafeInteger(width) ||
    width <= 0 ||
    !Number.isSafeInteger(height) ||
    height <= 0 ||
    !Number.isFinite(deviceScaleFactor) ||
    deviceScaleFactor <= 0
  ) {
    throw new Error("Invalid snapshot capture identity: variant or viewport");
  }
  return JSON.stringify([
    1,
    resolveSnapshotIdentity(job.art.path, projectRoot, identities),
    job.variantName,
    name || "",
    width,
    height,
    deviceScaleFactor,
  ]);
}

/** Reject corrupt ownership records, including noncanonical or unsupported tuples. */
export function validateCaptureIdentity(value: unknown): asserts value is string {
  if (typeof value !== "string") throw new Error("Invalid snapshot ownership identity");
  let tuple: unknown;
  try {
    tuple = JSON.parse(value);
  } catch {
    throw new Error("Invalid snapshot ownership identity JSON");
  }
  if (!Array.isArray(tuple) || tuple.length !== 7 || tuple[0] !== 1)
    throw new Error("Invalid snapshot ownership identity version or tuple");
  const [_, relative, variantName, name, width, height, deviceScaleFactor] = tuple;
  normalizeSnapshotIdentity(relative);
  const rebuilt = buildCaptureIdentity(
    { art: { path: relative }, variantName, viewport: { name, width, height, deviceScaleFactor } },
    process.cwd(),
    { [relative]: relative },
  );
  if (rebuilt !== value) throw new Error("Noncanonical snapshot ownership identity");
}
