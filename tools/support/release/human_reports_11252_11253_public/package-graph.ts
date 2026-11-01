import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { join } from "node:path";
import { observedArchiveComparison } from "./package-inventory.ts";
import { array, object, sha256, text, wholeJson } from "./value.ts";
import { publicArchive, publicPackageMetadata } from "./network.ts";
import {
  absentPackage,
  platformExclusion,
  presentPackage,
  physicalPathWitness,
  validateOptionalAbsence,
  type HostFacts,
  type OptionalAbsence,
} from "./optional-residency.ts";
import { lockPackageIntegrity } from "./lock-integrity.ts";
import type { InstallReceipt } from "./model.ts";
import type { Sink } from "./evidence.ts";

/** Physical installed nodes and nonresident declarations are deliberately
 * separate. A pnpm list category never establishes optionality or presence. */
export async function installedGraph(
  root: string,
  listing: unknown,
  lock: Buffer,
  host: HostFacts,
  required: ReadonlyMap<string, string>,
  archiveCache: Map<string, Buffer>,
  sink: Sink,
  readers = { archive: publicArchive, metadata: publicPackageMetadata },
): Promise<{ packages: InstallReceipt["packages"]; absent: OptionalAbsence[] }> {
  const packages: InstallReceipt["packages"] = [],
    absent: OptionalAbsence[] = [];
  const seen = new Set<string>();
  interface Parent {
    name: string;
    version: string;
    directory: string;
    manifest: Buffer;
  }
  const visit = async (value: unknown, parent: Parent | null): Promise<void> => {
    const current = object(value);
    for (const category of ["dependencies", "devDependencies", "optionalDependencies"]) {
      if (current[category] === undefined) continue;
      for (const [name, raw] of Object.entries(object(current[category]))) {
        const row = object(raw),
          version = text(row.version),
          path = text(row.path);
        sink("listed-package-physical-before-assertion", {
          parent: parent?.directory ?? root,
          category,
          name,
          row,
          physical: physicalPathWitness(root, path, parent?.directory ?? root, name),
        });
        const directory = presentPackage(root, path);
        sink("listed-package-residency", {
          parent: parent?.directory ?? root,
          category,
          name,
          row,
          directory,
        });
        if (directory === null) {
          assert(!required.has(name), `required provider cannot be absent: ${name}`);
          assert(parent, "root providers cannot be waived as optional");
          const manifest = object(wholeJson(parent.manifest));
          assert.equal(
            object(manifest.optionalDependencies)[name],
            version,
            "authenticated packed-parent exact optional declaration required",
          );
          const physical = absentPackage(root, path, parent.directory, name, version);
          sink("declared-optional-physical-absence", {
            parentDirectory: parent.directory,
            parentManifestRawBase64: parent.manifest.toString("base64"),
            name,
            version,
            physical,
          });
          const locked = lockPackageIntegrity(lock, name, version);
          const metadata = await readers.metadata(name, version, sink);
          sink("declared-optional-before-platform-assertion", {
            parent: parent.directory,
            name,
            version,
            metadata,
            physical,
            locked,
            host,
          });
          assert.equal(metadata.resolved, row.resolved);
          assert.equal(metadata.integrity, locked.integrity);
          const record: OptionalAbsence = {
            state: "DECLARED_OPTIONAL_NOT_INSTALLED",
            name,
            version,
            parentDirectory: parent.directory,
            parentName: parent.name,
            parentVersion: parent.version,
            parentManifestRawBase64: parent.manifest.toString("base64"),
            parentManifestSha256: sha256(parent.manifest),
            physical,
            metadata,
            lock: locked,
            host,
            excludedBy: platformExclusion(metadata.metadata, host),
            nativeLoadCredit: false,
          };
          validateOptionalAbsence(record, root, lock, host, packages);
          absent.push(record);
          sink("declared-optional-not-installed", record);
          // An absent declaration cannot supply resident descendants.
          for (const edge of ["dependencies", "devDependencies", "optionalDependencies"])
            assert(
              row[edge] === undefined || Object.keys(object(row[edge])).length === 0,
              "absent declaration has unverified descendants",
            );
          continue;
        }
        const manifest = readFileSync(join(directory, "package.json"));
        sink("installed-parent-before-assertion", {
          name,
          version,
          directory,
          rawBase64: manifest.toString("base64"),
          sha256: sha256(manifest),
        });
        const pkg = object(wholeJson(manifest));
        assert.equal(pkg.name, name);
        assert.equal(pkg.version, version);
        if (!seen.has(directory)) {
          const downloaded = await readers.archive(name, version, sink);
          const { body: archiveBody, ...metadata } = downloaded;
          sink("installed-package-before-archive-assertion", {
            entry: { name, version, directory },
            metadata,
            manifestRawBase64: manifest.toString("base64"),
            archiveBodyExcluded: true,
            archiveBytes: archiveBody.length,
          });
          const prior = archiveCache.get(name + "@" + version);
          if (prior)
            assert(prior.equals(downloaded.body), "package version cannot silently change archive");
          else archiveCache.set(name + "@" + version, downloaded.body);
          const comparison = observedArchiveComparison(
            directory,
            downloaded.body,
            downloaded.integrity,
            sink,
          );
          assert.equal(downloaded.resolved, row.resolved);
          packages.push({
            name,
            version,
            directory,
            resolved: downloaded.resolved,
            integrity: downloaded.integrity,
            archiveSha512: comparison.tarballSHA512,
            allInstalledBytesEqualArchive: true,
            files: comparison.files.map((file) => ({ ...file })),
          });
          seen.add(directory);
          await visit(row, { name, version, directory, manifest });
        }
      }
    }
  };
  const roots = array(listing);
  assert.equal(roots.length, 1);
  await visit(roots[0], null);
  assert(packages.length > 0);
  return { packages, absent };
}
