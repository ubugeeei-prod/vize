import { closeSync, openSync, readFileSync, readSync, statSync } from "node:fs";
import { createHash } from "node:crypto";
import { join } from "node:path";
import type { CommandPlan, InstallReceipt } from "./model.ts";
import { installedFiles } from "../public_install/archive.ts";
import { absentPackage, managerFiles, physicalPathWitness } from "./optional-residency.ts";
function image(path: string): unknown {
  try {
    const stat = statSync(path),
      hash = createHash("sha256"),
      fd = openSync(path, "r"),
      buffer = Buffer.alloc(65536);
    try {
      for (;;) {
        const n = readSync(fd, buffer);
        if (!n) break;
        hash.update(buffer.subarray(0, n));
      }
    } finally {
      closeSync(fd);
    }
    return { path, bytes: stat.size, sha256: hash.digest("hex") };
  } catch (error) {
    return { path, error: String(error) };
  }
}
export function payloadWitness(install: InstallReceipt): unknown {
  return {
    scope: "authenticated package payloads only; not whole node_modules",
    declaredOptionalNotInstalled: install.declaredOptionalNotInstalled.map((row) => {
      const physical = physicalPathWitness(install.root, row.physical.path, row.parentDirectory, row.name);
      try {
        return {
          name: row.name,
          version: row.version,
          parent: row.parentDirectory,
          actual: absentPackage(
            install.root,
            row.physical.path,
            row.parentDirectory,
            row.name,
            row.version,
          ),
          physical,
        };
      } catch (error) {
        return {
          name: row.name,
          version: row.version,
          parent: row.parentDirectory,
          error: String(error), physical,
        };
      }
    }),
    managerFiles: (() => {
      try {
        return managerFiles(install.root);
      } catch (error) {
        return { error: String(error) };
      }
    })(),
    packages: install.packages.map((pkg) => {
      let actual: unknown;
      try {
        actual = installedFiles(pkg.directory).sort().map(image);
      } catch (error) {
        actual = { error: String(error), knownFiles: pkg.files.map((f) => image(f.path)) };
      }
      return { name: pkg.name, directory: pkg.directory, actual };
    }),
  };
}
export function inputWitness(plan: CommandPlan, root: string): unknown {
  return {
    id: plan.id,
    files: plan.sourceFiles.map((f) => {
      const path = join(root, f.path);
      try {
        const stat = statSync(path);
        if (stat.size > 1024 * 1024)
          return {
            path,
            bytes: stat.size,
            rawRefused: "input exceeds existing small-input witness bound",
          };
        const raw = readFileSync(path);
        return { expected: f.sha256, actual: image(path), rawBase64: raw.toString("base64") };
      } catch (error) {
        return { path, error: String(error) };
      }
    }),
  };
}
