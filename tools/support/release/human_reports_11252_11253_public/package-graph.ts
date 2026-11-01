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
import { witnessKnownBins, type BinInstallContext } from "./bin-witness.ts";
import {
  collectBinContext,
  binInventory,
  contextFacts,
  type BinEdge,
  type BinManagerContext,
} from "./dependency-bin-context.ts";
import type { BinPolicy } from "./dependency-bin-policy.ts";
import type { HumanBinFacts, PublicPackageFacts } from "./dependency-bin-types.ts";
import { humanDependencyBinPartition } from "./dependency-bin-partition.ts";

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
  installContext?: BinInstallContext & {
    conformance?: { role: HumanBinFacts["role"]; policy: BinPolicy; manager: BinManagerContext };
  },
): Promise<{ packages: InstallReceipt["packages"]; absent: OptionalAbsence[] }> {
  const packages: InstallReceipt["packages"] = [],
    absent: OptionalAbsence[] = [];
  const seen = new Set<string>();
  const catalog = new Map<string, PublicPackageFacts>(),
    edges: BinEdge[] = [];
  const pending = new Map<string, PublicPackageFacts>();
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
        if (parent)
          edges.push({
            parentDirectory: parent.directory,
            dependencyName: name,
            version,
            directory,
          });
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
          assert.equal(downloaded.resolved, row.resolved);
          const declaration: PublicPackageFacts = {
            name,
            version,
            directory,
            manifest,
            archive: downloaded.body,
            integrity: downloaded.integrity,
            resolved: downloaded.resolved,
          };
          catalog.set(directory, declaration);
          await witnessKnownBins(
            directory,
            installContext
              ? { ...installContext, root, lock, archive: readers.archive }
              : undefined,
            sink,
          );
          if (installContext?.conformance && name === "vue" && version === "3.5.44") {
            // Defer only this closed role-specific partition until every
            // declared owner, including no-bin dependencies, is authenticated.
            pending.set(directory, declaration);
          } else {
            const comparison = observedArchiveComparison(
              directory,
              downloaded.body,
              downloaded.integrity,
              sink,
            );
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
          }
          seen.add(directory);
        } else {
          assert(
            catalog.get(directory)?.manifest.equals(manifest),
            "present owner changed between listed edges",
          );
        }
        // A first deduped occurrence may have no children. Walk every finite
        // JSON occurrence; physical payload deduplication must not discard a
        // later full occurrence's dependency closure.
        await visit(row, { name, version, directory, manifest });
      }
    }
  };
  const roots = array(listing);
  assert.equal(roots.length, 1);
  await visit(roots[0], null);
  for (const parent of pending.values()) {
    assert(installContext?.conformance);
    const context = collectBinContext(
      root,
      installContext.conformance.role,
      parent,
      catalog,
      edges,
      installContext.conformance.policy,
      installContext.conformance.manager,
    );
    sink("source-conformant-bin-inputs", context);
    const inventory = binInventory(parent.directory);
    sink(
      "source-conformant-bin-inventory",
      inventory.map(({ body, ...row }) => ({
        ...row,
        bytes: body?.length ?? null,
        sha256: body === null ? null : sha256(body),
      })),
    );
    const proof = humanDependencyBinPartition(contextFacts(context, archiveCache, inventory));
    sink("source-conformant-bin-partition", proof);
    packages.push({
      name: parent.name,
      version: parent.version,
      directory: parent.directory,
      resolved: parent.resolved,
      integrity: parent.integrity,
      archiveSha512: Buffer.from(parent.integrity.slice(7), "base64").toString("hex"),
      allInstalledBytesEqualArchive: false,
      binContext: context,
      sourceConformance: proof,
      files: proof.files.map(({ path, bytes, sha256 }) => ({ path, bytes, sha256 })),
    });
  }
  assert(packages.length > 0);
  return { packages, absent };
}
