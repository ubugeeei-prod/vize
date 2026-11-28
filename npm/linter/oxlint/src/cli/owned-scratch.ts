import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";

/**
 * A run's scratch directory below a root this user owns. Each entry is named
 * `<prefix><host>-<pid>-<random>`, so a later run on the same host removes
 * the entries of runs that were killed before their own cleanup ran.
 */
export function createOwnedScratch(root: string, prefix: string): string {
  fs.mkdirSync(path.dirname(root), { recursive: true });
  try {
    fs.mkdirSync(root, { mode: 0o700 });
  } catch (error) {
    if ((error as NodeJS.ErrnoException).code !== "EEXIST") throw error;
  }
  const owned = ownedRoot(root);
  const stem = `${prefix}${hostTag()}-`;
  if (owned) reapAbandoned(root, stem);
  return fs.mkdtempSync(path.join(owned ? root : path.dirname(root), `${stem}${process.pid}-`));
}

/** This user's root for scratch directories under the system temp directory. */
export function userTemporaryRoot(name: string): string {
  const user = process.getuid?.();
  return path.join(os.tmpdir(), user === undefined ? name : `${name}-${user}`);
}

/**
 * A root somebody else made, or one replaced by a link, is never adopted:
 * scratch entries then go beside it, as they did before roots existed.
 */
function ownedRoot(root: string): boolean {
  const stat = fs.lstatSync(root);
  if (!stat.isDirectory() || stat.isSymbolicLink()) return false;
  const user = process.getuid?.();
  return user === undefined || stat.uid === user;
}

function hostTag(): string {
  return createHash("sha256").update(os.hostname()).digest("hex").slice(0, 8);
}

function reapAbandoned(root: string, stem: string): void {
  for (const entry of fs.readdirSync(root)) {
    if (!entry.startsWith(stem)) continue;
    const pid = Number(entry.slice(stem.length).split("-")[0]);
    if (!Number.isSafeInteger(pid) || pid <= 0 || pid === process.pid || isAlive(pid)) continue;
    fs.rmSync(path.join(root, entry), { recursive: true, force: true });
  }
}

function isAlive(pid: number): boolean {
  try {
    process.kill(pid, 0);
    return true;
  } catch (error) {
    return (error as NodeJS.ErrnoException).code !== "ESRCH";
  }
}
