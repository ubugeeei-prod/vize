import { lstatSync, readFileSync, readdirSync, readlinkSync } from "node:fs";
import { join } from "node:path";
import { readTarGz } from "../../archive/tar.ts";
import { archiveComparison } from "../public_install/archive.ts";
import type { Sink } from "./evidence.ts";
import { sha256 } from "./value.ts";

interface PhysicalEntry {
  path: string;
  kind: "file" | "directory" | "symlink" | "other";
  bytes: number;
  sha256: string | null;
  linkTarget: string | null;
}

/** Diagnostic observations only. The shared comparison still decides whether
 * every installed file and every public payload byte are exactly equal. */
export function observedArchiveComparison(
  directory: string,
  payload: Buffer,
  integrity: unknown,
  sink: Sink,
) {
  const physical: PhysicalEntry[] = [];
  const walk = (relative: string): void => {
    for (const name of readdirSync(join(directory, relative)).sort()) {
      const path = relative ? `${relative}/${name}` : name;
      const absolute = join(directory, path);
      const stat = lstatSync(absolute);
      const kind = stat.isSymbolicLink()
        ? "symlink"
        : stat.isDirectory()
          ? "directory"
          : stat.isFile()
            ? "file"
            : "other";
      physical.push({
        path,
        kind,
        bytes: stat.size,
        sha256: kind === "file" ? sha256(readFileSync(absolute)) : null,
        linkTarget: kind === "symlink" ? readlinkSync(absolute) : null,
      });
      if (kind === "directory") walk(path);
    }
  };
  try {
    walk("");
  } catch (error) {
    sink("installed-package-file-inventory", {
      directory,
      complete: false,
      physical,
      error: String(error),
      bodyBytesExcluded: true,
    });
    throw error;
  }
  sink("installed-package-file-inventory", {
    directory,
    complete: true,
    physical,
    bodyBytesExcluded: true,
  });
  const archive = readTarGz(payload).map((member) => ({
    path: member.name,
    packageRelativePath: member.name.startsWith("package/") ? member.name.slice(8) : null,
    kind: member.kind,
    typeflag: member.typeflag,
    mode: member.mode,
    bytes: member.size,
    sha256: sha256(member.data),
    linkTarget: member.linkname,
  }));
  const expectedFiles = archive
    .filter((member) => member.kind === "file" && member.packageRelativePath !== null)
    .map((member) => member.packageRelativePath!);
  const actualFiles = physical.filter((entry) => entry.kind === "file").map((entry) => entry.path);
  const expected = new Set(expectedFiles),
    actual = new Set(actualFiles);
  sink("public-package-file-inventory", {
    directory,
    archiveBytes: payload.length,
    archiveSha256: sha256(payload),
    archive,
    installedRegularFiles: actualFiles,
    expectedRegularFiles: expectedFiles,
    difference: {
      extraInstalledRegularFiles: actualFiles.filter((path) => !expected.has(path)),
      missingInstalledRegularFiles: expectedFiles.filter((path) => !actual.has(path)),
      duplicateArchiveRegularPaths: expectedFiles.filter(
        (path, index) => expectedFiles.indexOf(path) !== index,
      ),
      installedNonRegularPaths: physical.filter((entry) => entry.kind !== "file"),
    },
    observationOnly: true,
    bodyBytesExcluded: true,
    payloadOrExtraFilesWaived: false,
  });
  return archiveComparison(directory, payload, integrity);
}
