import { createHash } from "node:crypto";

export interface DevCacheRecord {
  hash: string;
  servedMtimeMs: number;
}

export interface DevOutputFile {
  readFile(filePath: string): Promise<Buffer | Uint8Array | string>;
  stat(filePath: string): Promise<{ mtime: Date; mtimeMs?: number }>;
  utimes(filePath: string, atime: Date, mtime: Date): Promise<void>;
}

export function digestBytes(bytes: Buffer | Uint8Array | string): string {
  return createHash("sha256").update(bytes).digest("hex");
}

/**
 * Parcel 2.16.4 answers 304 when `mtime.toUTCString()` is less than or equal to
 * If-Modified-Since. That header drops sub-second precision, so a rewrite in the
 * same UTC second reuses the previous bundle. Move Last-Modified to the next
 * second only when the bytes change. Dev filenames stay stable.
 */
export function nextServedMtimeMs(
  previous: DevCacheRecord | undefined,
  hash: string,
  mtimeMs: number,
): number {
  if (!previous || previous.hash === hash) return mtimeMs;
  const previousSecond = Math.floor(previous.servedMtimeMs / 1000);
  if (Math.floor(mtimeMs / 1000) > previousSecond) return mtimeMs;
  return (previousSecond + 1) * 1000;
}

/** True when Parcel's dev server would answer 304 for this mtime. */
export function parcelRespondsNotModified(ifModifiedSince: string, mtimeMs: number): boolean {
  const lastModified = new Date(mtimeMs).toUTCString();
  return Date.parse(lastModified) <= Date.parse(ifModifiedSince);
}

export async function advanceDevBundleMtime(
  fs: DevOutputFile,
  filePath: string,
  previous: DevCacheRecord | undefined,
): Promise<DevCacheRecord> {
  const hash = digestBytes(await fs.readFile(filePath));
  const stat = await fs.stat(filePath);
  const mtimeMs = stat.mtimeMs ?? stat.mtime.getTime();
  const servedMtimeMs = nextServedMtimeMs(previous, hash, mtimeMs);
  if (Math.floor(servedMtimeMs / 1000) !== Math.floor(mtimeMs / 1000)) {
    const served = new Date(servedMtimeMs);
    await fs.utimes(filePath, served, served);
  }
  return { hash, servedMtimeMs };
}
