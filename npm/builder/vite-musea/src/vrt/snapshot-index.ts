import crypto from "node:crypto";
import fs from "node:fs/promises";
import type { FileHandle } from "node:fs/promises";
import path from "node:path";
import {
  buildCaptureIdentity,
  normalizeSnapshotIdentity,
  validateCaptureIdentity,
  type SnapshotIdentities,
  type SnapshotIdentityJob,
} from "./snapshot-identity.js";
import { buildSnapshotName } from "./utils.js";

export const SNAPSHOT_INDEX_FILE = "identities.json";
export const SNAPSHOT_LOCK_FILE = "identities.lock";

export interface SnapshotIndexOptions {
  projectRoot?: string;
  snapshotIdentities?: SnapshotIdentities;
  adoptLegacySnapshots?: boolean;
}

type IndexData = { version: 1; owners: Record<string, string> };
type PlannedJob = { identity: string; legacyName: string | null };

/** Reserve one owner per PNG before parallel captures can touch the filesystem. */
export class SnapshotIndex {
  private owners: Record<string, string> = Object.create(null);
  private token = crypto.randomUUID();
  private lock: FileHandle | undefined;
  private closed = false;
  private closing = false;
  private pending: Promise<unknown> = Promise.resolve();

  private constructor(
    private readonly directory: string,
    private readonly options: SnapshotIndexOptions,
  ) {}

  static async open(directory: string, options: SnapshotIndexOptions = {}): Promise<SnapshotIndex> {
    validateIdentityMap(options.snapshotIdentities);
    const index = new SnapshotIndex(path.resolve(directory), {
      ...options,
      projectRoot: path.resolve(options.projectRoot ?? process.cwd()),
      snapshotIdentities:
        options.snapshotIdentities === undefined ? undefined : { ...options.snapshotIdentities },
    });
    await fs.mkdir(index.directory, { recursive: true });
    try {
      index.lock = await fs.open(path.join(index.directory, SNAPSHOT_LOCK_FILE), "wx");
    } catch (error) {
      if (hasCode(error, "EEXIST"))
        throw new Error(
          `Snapshot directory is locked: ${index.directory}. Another VRT run may be active.`,
        );
      throw error;
    }
    try {
      await index.lock.writeFile(index.token);
      const file = path.join(index.directory, SNAPSHOT_INDEX_FILE);
      try {
        index.owners = validateIndex(JSON.parse(await fs.readFile(file, "utf8")));
      } catch (error) {
        if (!hasCode(error, "ENOENT"))
          throw new Error(`Invalid snapshot ownership index: ${file}`, { cause: error });
      }
      return index;
    } catch (error) {
      await index.close();
      throw error;
    }
  }

  plan(jobs: readonly SnapshotIdentityJob[]): Promise<Map<string, string>> {
    if (this.closed || this.closing)
      return Promise.reject(new Error("Snapshot ownership index is closed"));
    const result = this.pending.then(() => this.planBatch(jobs));
    this.pending = result.catch(() => {});
    return result;
  }

  private async planBatch(jobs: readonly SnapshotIdentityJob[]): Promise<Map<string, string>> {
    const planned = this.prepare(jobs);
    const alreadyOwned = new Map(Object.entries(this.owners).map(([name, id]) => [id, name]));
    if (planned.every(({ identity }) => alreadyOwned.has(identity))) {
      const result = new Map(
        planned.map(({ identity }) => [identity, alreadyOwned.get(identity)!]),
      );
      await this.validateFiles(result.values());
      return result;
    }
    const files = await fs.readdir(this.directory);
    const existing = new Map<string, string[]>();
    for (const name of files) {
      const key = foldName(name);
      existing.set(key, [...(existing.get(key) ?? []), name]);
    }
    const owners = { ...this.owners };
    const byIdentity = new Map(Object.entries(owners).map(([name, id]) => [id, name]));
    const byName = new Map(Object.entries(owners).map(([name, id]) => [foldName(name), id]));
    const groups = new Map<string, number>();
    for (const job of planned) {
      if (job.legacyName) {
        const key = foldName(job.legacyName);
        groups.set(key, (groups.get(key) ?? 0) + 1);
      }
    }
    const result = new Map<string, string>();
    for (const { identity, legacyName } of planned) {
      const owned = byIdentity.get(identity);
      if (owned) {
        result.set(identity, owned);
        continue;
      }
      const legacyKey = legacyName ? foldName(legacyName) : "";
      const ambiguous =
        (groups.get(legacyKey) ?? 0) > 1 || (existing.get(legacyKey)?.length ?? 0) > 1;
      if (
        legacyName &&
        ambiguous &&
        existing.has(legacyKey) &&
        !byName.has(legacyKey) &&
        this.options.adoptLegacySnapshots
      ) {
        throw new Error(`Cannot adopt ambiguous legacy snapshot: ${legacyName}`);
      }
      let name =
        legacyName && !ambiguous && !byName.has(legacyKey) ? legacyName : hashedName(identity);
      const key = foldName(name);
      if (byName.has(key)) throw new Error(`Snapshot filename ownership collision: ${name}`);
      if (existing.has(key) && !this.options.adoptLegacySnapshots)
        throw new Error(`Unowned snapshot ${name}. Explicit adoptLegacySnapshots is required.`);
      if (existing.has(key)) {
        const matches = existing.get(key)!;
        if (matches.length !== 1)
          throw new Error(`Cannot adopt ambiguous legacy snapshot: ${name}`);
        if (!isSafeName(matches[0])) throw new Error(`Invalid snapshot filename: ${matches[0]}`);
        name = matches[0];
      }
      owners[name] = identity;
      byName.set(key, identity);
      byIdentity.set(identity, name);
      result.set(identity, name);
    }
    await this.validateFiles(result.values());
    await this.persist(owners);
    this.owners = owners;
    return result;
  }

  async close(): Promise<void> {
    if (this.closed || this.closing) return;
    this.closing = true;
    await this.pending;
    this.closed = true;
    const lock = this.lock;
    if (!lock) return;
    const file = path.join(this.directory, SNAPSHOT_LOCK_FILE);
    try {
      if ((await fs.readFile(file, "utf8")) === this.token) await fs.unlink(file);
    } catch (error) {
      if (!hasCode(error, "ENOENT")) throw error;
    } finally {
      await lock.close();
      this.lock = undefined;
    }
  }

  private prepare(jobs: readonly SnapshotIdentityJob[]): PlannedJob[] {
    const identities = new Set<string>();
    return jobs
      .map((job) => {
        const identity = buildCaptureIdentity(
          job,
          this.options.projectRoot,
          this.options.snapshotIdentities,
        );
        if (identities.has(identity))
          throw new Error(`Duplicate snapshot capture identity: ${identity}`);
        identities.add(identity);
        let legacyName: string | null = null;
        try {
          const relative = JSON.parse(identity)[1] as string;
          const candidate = buildSnapshotName(relative, job.variantName, job.viewport);
          if (isSafeName(candidate)) legacyName = candidate;
        } catch {
          // An unpaired UTF-16 surrogate cannot be percent encoded; its JSON tuple can be hashed.
        }
        return { identity, legacyName };
      })
      .sort((a, b) => (a.identity < b.identity ? -1 : a.identity > b.identity ? 1 : 0));
  }

  private async persist(owners: Record<string, string>): Promise<void> {
    const file = path.join(this.directory, SNAPSHOT_INDEX_FILE);
    const temporary = `${file}.${this.token}.tmp`;
    const ordered = Object.fromEntries(
      Object.entries(owners).sort(([a], [b]) => (a < b ? -1 : a > b ? 1 : 0)),
    );
    try {
      await fs.writeFile(
        temporary,
        `${JSON.stringify({ version: 1, owners: ordered } satisfies IndexData, null, 2)}\n`,
        { flag: "wx" },
      );
      await fs.rename(temporary, file);
    } finally {
      await fs.rm(temporary, { force: true });
    }
  }

  private async validateFiles(names: Iterable<string>): Promise<void> {
    for (const name of names) {
      try {
        const stat = await fs.lstat(path.join(this.directory, name));
        if (!stat.isFile()) throw new Error(`Snapshot must be a regular PNG file: ${name}`);
      } catch (error) {
        if (!hasCode(error, "ENOENT")) throw error;
      }
    }
  }
}

function validateIdentityMap(identities: SnapshotIdentities | undefined): void {
  if (identities === undefined) return;
  if (!isRecord(identities)) throw new Error("Invalid snapshot identity map");
  const seen = new Set<string>();
  for (const identity of Object.values(identities)) {
    normalizeSnapshotIdentity(identity);
    if (seen.has(identity)) throw new Error(`Duplicate Art snapshot identity: ${identity}`);
    seen.add(identity);
  }
}

function validateIndex(data: unknown): Record<string, string> {
  if (!isRecord(data) || data.version !== 1 || !isRecord(data.owners))
    throw new Error("Snapshot ownership index must have version 1 and owners");
  const owners: Record<string, string> = Object.create(null);
  const names = new Set<string>();
  const identities = new Set<string>();
  for (const [name, identity] of Object.entries(data.owners)) {
    if (!isSafeName(name)) throw new Error(`Invalid snapshot filename: ${name}`);
    validateCaptureIdentity(identity);
    if (names.has(foldName(name)) || identities.has(identity))
      throw new Error(`Duplicate snapshot ownership: ${name}`);
    if (/^snapshot-[0-9a-f]{64}\.png$/i.test(name) && foldName(name) !== hashedName(identity))
      throw new Error(`Snapshot filename does not match its identity: ${name}`);
    names.add(foldName(name));
    identities.add(identity);
    owners[name] = identity;
  }
  return owners;
}

function hashedName(identity: string): string {
  return `snapshot-${crypto.createHash("sha256").update(identity).digest("hex")}.png`;
}

function foldName(name: string): string {
  return name.toLowerCase();
}

function isSafeName(name: string): boolean {
  return (
    name.endsWith(".png") &&
    Buffer.byteLength(name) <= 255 &&
    !/[\\/<>:"|?*]/.test(name) &&
    !Array.from(name).some(
      (character) => character.charCodeAt(0) < 32 || character.charCodeAt(0) > 126,
    ) &&
    name !== ".png"
  );
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return value !== null && typeof value === "object" && !Array.isArray(value);
}

function hasCode(error: unknown, code: string): boolean {
  return isRecord(error) && error.code === code;
}
