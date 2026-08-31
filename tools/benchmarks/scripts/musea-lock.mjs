/** Cross-process ownership for Musea's fixed benchmark workspace. */

import { randomUUID } from "node:crypto";
import { existsSync, mkdirSync, readFileSync, renameSync, rmSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";

const OWNER_FILE = "owner.json";

export function museaWorkspaceLockPath(rootDir) {
  return join(rootDir, "target", "musea-benchmark", ".workspace-lock");
}

function readOwner(lockPath) {
  try {
    const value = JSON.parse(readFileSync(join(lockPath, OWNER_FILE), "utf8"));
    if (
      Number.isSafeInteger(value.pid) &&
      value.pid > 0 &&
      typeof value.token === "string" &&
      /^[A-Za-z0-9-]{1,128}$/.test(value.token) &&
      typeof value.acquiredAt === "string"
    ) {
      return value;
    }
  } catch {
    // A creator may be between atomic mkdir and writing its owner record.
  }
  return null;
}

function processIsAlive(pid) {
  try {
    process.kill(pid, 0);
    return true;
  } catch (error) {
    // EPERM means the process exists but this user cannot signal it. Only ESRCH
    // proves that reclaiming its workspace cannot disturb a live owner.
    return error?.code !== "ESRCH";
  }
}

function lockError(lockPath, owner) {
  if (owner == null) {
    return new Error(
      `musea-workspace-lock: ${lockPath} exists but its owner cannot be verified; refusing to disturb a possibly live benchmark`,
    );
  }
  return new Error(
    `musea-workspace-lock: ${lockPath} is already held by live pid ${owner.pid} since ${owner.acquiredAt}; only one Musea benchmark may use this checkout at a time`,
  );
}

/**
 * Atomically acquire the fixed workspace and return an ownership-checked release.
 * A dead owner's directory is first atomically renamed, so two reclaimers can
 * never remove a newly acquired lock.
 */
export function acquireMuseaWorkspaceLock(rootDir) {
  const lockPath = museaWorkspaceLockPath(rootDir);
  mkdirSync(dirname(lockPath), { recursive: true });

  for (;;) {
    try {
      mkdirSync(lockPath);
    } catch (error) {
      if (error?.code !== "EEXIST") throw error;
      const owner = readOwner(lockPath);
      if (owner == null || processIsAlive(owner.pid)) throw lockError(lockPath, owner);

      // The dead owner's token gives every reclaimer the same permanent
      // tombstone. Keeping it prevents a delayed reclaimer that read the old
      // owner from ever renaming a later live owner's directory.
      const quarantine = `${lockPath}.reclaimed-${owner.token}`;
      try {
        renameSync(lockPath, quarantine);
      } catch (renameError) {
        if (renameError?.code === "ENOENT" || existsSync(quarantine)) continue;
        throw renameError;
      }
      continue;
    }

    const owner = {
      pid: process.pid,
      token: randomUUID(),
      acquiredAt: new Date().toISOString(),
    };
    try {
      writeFileSync(join(lockPath, OWNER_FILE), `${JSON.stringify(owner)}\n`, { flag: "wx" });
    } catch (error) {
      rmSync(lockPath, { recursive: true, force: true });
      throw error;
    }

    let released = false;
    return () => {
      if (released) return;
      const current = readOwner(lockPath);
      if (current?.token !== owner.token) {
        throw new Error(
          `musea-workspace-lock: ownership of ${lockPath} changed before release; refusing to remove another process's lock`,
        );
      }
      rmSync(lockPath, { recursive: true, force: true });
      released = true;
    };
  }
}

export async function withMuseaWorkspaceLock(rootDir, fn) {
  const release = acquireMuseaWorkspaceLock(rootDir);
  try {
    return await fn();
  } finally {
    release();
  }
}
