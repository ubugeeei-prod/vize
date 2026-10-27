// Inventory an npm package archive without extracting or trusting member paths.
import { createHash } from "node:crypto";
import { readTarGz } from "../../../../tools/support/archive/tar.ts";

export interface ArchiveFile {
  path: string;
  bytes: number;
  sha256: string;
}

const quote = (value: string) => `'${value.replaceAll("\\", "\\\\").replaceAll("'", "\\'")}'`;

/** Regular `package/` members sorted by path; any other member is refused. */
export function packageInventory(payload: Uint8Array): ArchiveFile[] {
  const files: ArchiveFile[] = [];
  const seen = new Set<string>();
  for (const member of readTarGz(payload)) {
    const parts = member.name.split("/");
    if (
      member.name.startsWith("/") ||
      parts[0] !== "package" ||
      parts.some((part) => part === "" || part === "." || part === "..")
    )
      throw new Error(`unsafe package member: ${quote(member.name)}`);
    if (member.kind === "directory") continue;
    if (member.kind !== "file" || parts.length < 2)
      throw new Error(`non-regular package member: ${quote(member.name)}`);
    const relative = parts.slice(1).join("/");
    if (seen.has(relative)) throw new Error(`duplicate package member: ${quote(relative)}`);
    seen.add(relative);
    files.push({
      path: relative,
      bytes: member.data.length,
      sha256: createHash("sha256").update(member.data).digest("hex"),
    });
  }
  if (files.length === 0) throw new Error("empty package archive");
  return files.sort((left, right) =>
    left.path < right.path ? -1 : left.path > right.path ? 1 : 0,
  );
}
