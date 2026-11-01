import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { lstatSync, readFileSync, realpathSync } from "node:fs";
import { createRequire } from "node:module";
import { dirname, join, relative, resolve } from "node:path";
import { readTarGz } from "../../archive/tar.ts";
import { fileImage, inside } from "../computed_event_custody.ts";
import { lockPackageIntegrity } from "./lock-integrity.ts";
import type { publicArchive } from "./network.ts";
import type { Sink } from "./evidence.ts";
import { object, sha256, text, wholeJson } from "./value.ts";

export interface BinInstallContext {
  path: string;
  managerVersion: string;
}
interface Context extends BinInstallContext {
  root: string;
  lock: Buffer;
  archive: typeof publicArchive;
}

function textFile(path: string, maximum: number) {
  const stat = lstatSync(path);
  assert(stat.isFile() && !stat.isSymbolicLink(), "witness requires an actual regular file");
  const image = fileImage(path);
  if (image.bytes > maximum)
    return { ...image, bodyState: "SIZE_EXCEEDS_TEXT_WITNESS", rawBase64: null };
  const raw = readFileSync(path);
  assert(raw.length <= maximum && raw.length === image.bytes);
  assert.equal(sha256(raw), image.sha256);
  let source: string;
  try {
    source = new TextDecoder("utf-8", { fatal: true }).decode(raw);
  } catch {
    return { ...image, bodyState: "NOT_UTF8_TEXT", rawBase64: null };
  }
  if (source.includes("\0")) return { ...image, bodyState: "CONTAINS_NUL", rawBase64: null };
  return { ...image, bodyState: "COMPLETE_UTF8_TEXT", rawBase64: raw.toString("base64") };
}

/** Collect missing facts only. No body, name, target or hash here exempts an
 * installed file from the original strict archive/set comparison. */
export async function witnessKnownBins(
  directory: string,
  context: Context | undefined,
  sink: Sink,
): Promise<void> {
  if (!context) return;
  for (const bin of ["tsc", "tsserver"]) {
    const path = join(directory, "node_modules/.bin", bin);
    if (lstatSync(path, { throwIfNoEntry: false }) === undefined) continue;
    try {
      assert(inside(context.root, directory));
      const shim = textFile(path, 4200);
      sink("nested-bin-body-witness", {
        directory,
        bin,
        shim,
        interpretation: "UNASSESSED",
        waiver: false,
      });
      const initialRaw = readFileSync(context.path);
      const initial = object(wholeJson(initialRaw));
      assert.equal(initial.root, context.root);
      const manager = object(initial.pnpm),
        managerPath = text(manager.path);
      const currentManager = textFile(managerPath, 4200);
      const managerManifestPath = resolve(dirname(managerPath), "../package.json");
      sink("nested-bin-manager-context", {
        directory,
        bin,
        reportedVersion: context.managerVersion,
        originalInstallContext: {
          path: context.path,
          bytes: initialRaw.length,
          sha256: sha256(initialRaw),
        },
        initialNode: initial.node,
        initialManager: manager,
        currentManager,
        managerImageUnchanged:
          currentManager.sha256 === manager.sha256 && currentManager.bytes === manager.bytes,
        originalArgv: initial.originalArgv,
        managerFiles: initial.managerFiles,
        generatorSemantics: "UNASSESSED",
        waiver: false,
      });
      if (lstatSync(managerManifestPath, { throwIfNoEntry: false }) !== undefined)
        sink("nested-bin-manager-owner-manifest", {
          directory,
          bin,
          manifest: textFile(managerManifestPath, 1024 * 1024),
          interpretation: "UNASSESSED",
        });
      if (currentManager.bodyState === "COMPLETE_UTF8_TEXT") {
        const source = Buffer.from(currentManager.rawBase64!, "base64").toString("utf8");
        const references = new Set(
          [
            ...source.matchAll(/(?:require\s*\(|from\s*|import\s*)\s*["'](\.{1,2}\/[^"']+)["']/gu),
          ].map((match) => match[1]!),
        );
        for (const specifier of references) {
          try {
            const target = realpathSync(createRequire(managerPath).resolve(specifier));
            assert(
              inside(dirname(managerManifestPath), target),
              "manager reference escaped its owner",
            );
            sink("nested-bin-manager-static-reference", {
              managerPath,
              specifier,
              image: fileImage(target),
              bodyExcluded: true,
              runtimeLoadObserved: false,
              waiver: false,
            });
          } catch (error) {
            sink("nested-bin-manager-reference-refusal", {
              managerPath,
              specifier,
              error: String(error),
              runtimeLoadObserved: false,
              waiver: false,
            });
          }
        }
      }

      // Inspect Node's physical metadata search order without loading a package
      // or relying on its exports permitting package.json as a public subpath.
      const search = createRequire(join(directory, "package.json")).resolve.paths("typescript");
      assert(search);
      let ownerCandidate: string | null = null;
      for (const base of search) {
        const candidate = join(base, "typescript/package.json");
        if (!inside(context.root, candidate)) break;
        if (lstatSync(candidate, { throwIfNoEntry: false }) !== undefined) {
          ownerCandidate = candidate;
          break;
        }
      }
      sink("nested-bin-owner-search", { directory, bin, search, ownerCandidate, waiver: false });
      assert(ownerCandidate, "no TypeScript bin-owner manifest in the original consumer");
      const ownerPath = realpathSync(ownerCandidate);
      assert(inside(context.root, ownerPath), "bin-owner manifest escaped the original consumer");
      const manifest = textFile(ownerPath, 1024 * 1024);
      sink("nested-bin-target-owner-manifest", {
        directory,
        bin,
        manifest,
        resolutionFrom: join(directory, "package.json"),
        waiver: false,
      });
      assert.equal(manifest.bodyState, "COMPLETE_UTF8_TEXT");
      const owner = object(wholeJson(Buffer.from(manifest.rawBase64!, "base64")));
      const name = text(owner.name),
        version = text(owner.version);
      assert.equal(name, "typescript");
      const declaredBin = text(object(owner.bin)[bin]);
      const ownerDirectory = dirname(ownerPath),
        declaredPath = resolve(ownerDirectory, declaredBin);
      assert(inside(ownerDirectory, declaredPath), "declared bin escaped its installed package");
      const target = realpathSync(declaredPath);
      assert(inside(ownerDirectory, target), "resolved bin escaped its installed package");
      const image = fileImage(target),
        locked = lockPackageIntegrity(context.lock, name, version);
      sink("nested-bin-declared-target", {
        directory,
        bin,
        owner: { name, version, directory: ownerDirectory },
        declaredBin,
        declaredPath,
        image,
        locked,
        bodyExcluded: true,
        shimBodyTargetBinding: "UNASSESSED",
        executed: false,
        waiver: false,
      });
      const downloaded = await context.archive(name, version, sink);
      const { body, ...metadata } = downloaded;
      const actualSri = "sha512-" + createHash("sha512").update(body).digest("base64");
      sink("nested-bin-target-public-archive", {
        directory,
        bin,
        metadata,
        archiveBytes: body.length,
        archiveSha256: sha256(body),
        actualSri,
        archiveBodyExcluded: true,
        installedTarget: image,
        waiver: false,
      });
      assert.equal(metadata.name, name);
      assert.equal(metadata.version, version);
      assert.equal(metadata.integrity, actualSri);
      assert.equal(metadata.integrity, locked.integrity);
      const memberPath = "package/" + relative(ownerDirectory, target).replaceAll("\\", "/");
      const members = readTarGz(body).filter((member) => member.name === memberPath);
      assert.equal(members.length, 1);
      assert.equal(members[0]!.kind, "file");
      assert.equal(members[0]!.size, image.bytes);
      assert.equal(sha256(members[0]!.data), image.sha256);
      sink("nested-bin-target-payload-binding", {
        directory,
        bin,
        owner: { name, version },
        memberPath,
        bytes: image.bytes,
        sha256: image.sha256,
        archiveIntegrity: metadata.integrity,
        archiveSha256: sha256(body),
        locked,
        targetBytesEqualPublicMember: true,
        shimBodyTargetBinding: "UNASSESSED",
        generatorSemantics: "UNASSESSED",
        executed: false,
        waiver: false,
      });
    } catch (error) {
      sink("nested-bin-witness-refusal", {
        directory,
        bin,
        path,
        error: String(error),
        waiver: false,
        interpretation: "UNASSESSED",
      });
    }
  }
}
