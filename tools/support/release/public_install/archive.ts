// Read public archives without extraction; compare the entire installed file set.
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { readTarGz } from "../../archive/tar.ts";
import { CustodyError, digest, exactPath } from "./identity.ts";
import { sriSha512 } from "./registry.ts";

export interface InstalledFile {
  readonly path: string;
  readonly sha256: string;
  readonly bytes: number;
}
export interface ArchiveComparison {
  readonly fileCount: number;
  readonly tarballSHA512: string;
  readonly allInstalledFileBytesEqualPublicArchive: true;
  readonly files: readonly InstalledFile[];
}

/** Every regular file below `packageDir`; links and special files refuse. */
export function installedFiles(packageDir: string): string[] {
  const files: string[] = [];
  const walk = (directory: string): void => {
    const entries = fs.readdirSync(directory, { withFileTypes: true });
    for (const entry of entries) {
      const filename = path.join(directory, entry.name);
      if (entry.isSymbolicLink() || (!entry.isDirectory() && !entry.isFile()))
        throw new CustodyError(`installed package contains a redirect/special file: ${filename}`);
    }
    for (const entry of entries) if (entry.isFile()) files.push(path.join(directory, entry.name));
    for (const entry of entries) if (entry.isDirectory()) walk(path.join(directory, entry.name));
  };
  walk(packageDir);
  return files;
}

const byPath = (left: InstalledFile, right: InstalledFile): number =>
  left.path < right.path ? -1 : left.path > right.path ? 1 : 0;

export function archiveComparison(
  packageDir: string,
  payload: Buffer,
  integrity: unknown,
): ArchiveComparison {
  exactPath(packageDir, undefined, true);
  const sha512 = createHash("sha512").update(payload).digest();
  if (!sha512.equals(sriSha512(integrity)))
    throw new CustodyError("public tarball integrity differs");
  const files: InstalledFile[] = [];
  const names = new Set<string>();
  for (const member of readTarGz(payload)) {
    const raw = member.name;
    const parts = raw.split("/");
    if (
      parts[0] !== "package" ||
      parts.some((part) => part === "" || part === "." || part === "..") ||
      raw.includes("\\") ||
      (member.kind !== "directory" && member.kind !== "file")
    )
      throw new CustodyError(`unexpected registry tarball member: ${raw}`);
    if (member.kind === "directory") continue;
    if (parts.length < 2) throw new CustodyError("archive file must have a package-relative name");
    const relative = parts.slice(1).join("/");
    if (names.has(relative)) throw new CustodyError(`duplicate public archive file: ${relative}`);
    names.add(relative);
    const installed = exactPath(path.join(packageDir, relative), packageDir);
    const data = fs.readFileSync(installed);
    if (!data.equals(member.data))
      throw new CustodyError(`installed bytes differ from actual public archive: ${relative}`);
    files.push({ path: installed, sha256: digest(data), bytes: data.length });
  }
  const actual = new Set(installedFiles(packageDir).map((file) => path.relative(packageDir, file)));
  if (
    names.size === 0 ||
    actual.size !== names.size ||
    [...actual].some((name) => !names.has(name))
  )
    throw new CustodyError("public archive and installed package file sets differ");
  return {
    fileCount: names.size,
    tarballSHA512: sha512.toString("hex"),
    allInstalledFileBytesEqualPublicArchive: true,
    files: files.sort(byPath),
  };
}

export interface ArchiveReceipt {
  readonly packageDirectory: string;
  readonly files: readonly InstalledFile[];
}

export function recheckFiles(receipts: readonly ArchiveReceipt[]): void {
  for (const receipt of receipts) {
    const packageDir = receipt.packageDirectory;
    const files = installedFiles(packageDir);
    const expected = new Set(receipt.files.map((item) => item.path));
    if (files.length !== expected.size || files.some((file) => !expected.has(file)))
      throw new CustodyError("public installed file set changed during collection");
    for (const item of receipt.files)
      if (digest(fs.readFileSync(exactPath(item.path, packageDir))) !== item.sha256)
        throw new CustodyError("public installed bytes changed during collection");
  }
}
