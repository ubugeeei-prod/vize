import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import * as fs from "node:fs";
import * as path from "node:path";

export const lockName = /(?:^|\/)(?:pnpm-lock\.yaml|package-lock\.json|yarn\.lock)$/;

export interface AuthoredFile {
  file: string;
  mode: string;
  oid: string;
  bytes: Buffer;
}

export function git(cwd: string, args: string[]): string {
  return execFileSync("git", args, { cwd, encoding: "utf8", maxBuffer: 32 * 1024 * 1024 });
}

export function blobOid(bytes: Buffer): string {
  return createHash("sha1").update(`blob ${bytes.length}\0`).update(bytes).digest("hex");
}

export function readAuthored(cwd: string, file: string, mode: string): Buffer {
  const absolute = path.join(cwd, file);
  const stat = fs.lstatSync(absolute);
  assert.equal(stat.isSymbolicLink(), mode === "120000", `source kind changed: ${file}`);
  if (mode === "120000") return Buffer.from(fs.readlinkSync(absolute));
  assert.ok(stat.isFile(), `authored source is not a file: ${file}`);
  if (process.platform !== "win32")
    assert.equal(Boolean(stat.mode & 0o111), mode === "100755", `source mode changed: ${file}`);
  return fs.readFileSync(absolute);
}

export function assertNoSourceMutation(
  authored: AuthoredFile[],
  cwd: string,
  includeLocks = false,
): void {
  for (const entry of authored) {
    if (!includeLocks && lockName.test(entry.file)) continue;
    assert.equal(
      blobOid(readAuthored(cwd, entry.file, entry.mode)),
      entry.oid,
      `authored source changed: ${entry.file}`,
    );
  }
}

export function restoreAuthored(cwd: string, entry: AuthoredFile): void {
  const file = path.join(cwd, entry.file);
  fs.mkdirSync(path.dirname(file), { recursive: true });
  try {
    fs.unlinkSync(file);
  } catch (error) {
    if ((error as NodeJS.ErrnoException).code !== "ENOENT") throw error;
  }
  if (entry.mode === "120000") fs.symlinkSync(entry.bytes.toString(), file);
  else fs.writeFileSync(file, entry.bytes, { mode: entry.mode === "100755" ? 0o755 : 0o644 });
}

export function nearestPackage(owner: string, name: string): string | null {
  let directory = owner;
  while (true) {
    const candidate = path.join(directory, "node_modules", name);
    if (fs.existsSync(path.join(candidate, "package.json"))) return fs.realpathSync(candidate);
    const parent = path.dirname(directory);
    if (directory === parent) return null;
    directory = parent;
  }
}

export function walkFiles(
  directory: string,
  visit: (file: string) => void,
  excluded = new Set<string>(),
): void {
  if (!fs.existsSync(directory)) return;
  for (const entry of fs.readdirSync(directory, { withFileTypes: true })) {
    if (excluded.has(entry.name)) continue;
    const file = path.join(directory, entry.name);
    if (entry.isDirectory()) walkFiles(file, visit, excluded);
    else if (entry.isFile() || (entry.isSymbolicLink() && fs.statSync(file).isFile())) visit(file);
  }
}
