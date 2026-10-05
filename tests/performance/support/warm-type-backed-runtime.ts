import assert from "node:assert/strict";
import fs from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";

import { sha256 } from "../../differential/manifest.mjs";

/** Actual resolved packages and payloads are witnessed once outside request timing. */
export function runtimeGraph(roots: string[]) {
  const packages = new Map<string, Record<string, unknown>>();
  const pending = roots.map((file) => fs.realpathSync(file));
  while (pending.length) {
    const manifestPath = pending.shift()!;
    if (packages.has(manifestPath)) continue;
    const bytes = fs.readFileSync(manifestPath);
    const manifest = JSON.parse(bytes.toString("utf8"));
    const directory = path.dirname(manifestPath);
    const files: Array<Record<string, unknown>> = [];
    const visit = (relative: string) => {
      const absolute = path.join(directory, relative);
      const stat = fs.lstatSync(absolute);
      if (stat.isSymbolicLink()) {
        files.push({ path: relative, kind: "symlink", target: fs.readlinkSync(absolute) });
      } else if (stat.isDirectory()) {
        for (const name of fs
          .readdirSync(absolute)
          .toSorted((a, b) => (a < b ? -1 : a > b ? 1 : 0))) {
          if (name !== "node_modules") visit(path.join(relative, name));
        }
      } else {
        assert.ok(stat.isFile());
        const content = fs.readFileSync(absolute);
        files.push({
          path: relative,
          kind: "file",
          bytes: content.length,
          sha256: sha256(content),
        });
      }
    };
    visit("");
    const require = createRequire(manifestPath);
    const edges: Array<Record<string, unknown>> = [];
    for (const category of ["dependencies", "optionalDependencies", "peerDependencies"]) {
      for (const [name, requested] of Object.entries(manifest[category] ?? {})) {
        let resolved: string;
        try {
          resolved = require.resolve(`${name}/package.json`);
        } catch (failure) {
          // Export maps can hide package.json. Locate the same resolved entry's
          // own manifest, rather than substituting a different installed graph.
          try {
            let parent = path.dirname(require.resolve(name));
            while (true) {
              const candidate = path.join(parent, "package.json");
              if (
                fs.existsSync(candidate) &&
                JSON.parse(fs.readFileSync(candidate, "utf8")).name === name
              ) {
                resolved = candidate;
                break;
              }
              assert.notEqual(path.dirname(parent), parent);
              parent = path.dirname(parent);
            }
          } catch (entryFailure) {
            assert.notEqual(
              category,
              "dependencies",
              `required provider dependency ${name} is absent`,
            );
            edges.push({
              name,
              requested,
              category,
              state: "unresolved",
              error: String(failure),
              entryError: String(entryFailure),
            });
            continue;
          }
        }
        const physical = fs.realpathSync(resolved);
        pending.push(physical);
        edges.push({ name, requested, category, state: "resolved", manifestPath: physical });
      }
    }
    packages.set(manifestPath, {
      manifestPath,
      manifestSha256: sha256(bytes),
      manifest,
      packagePayload: files,
      packagePayloadSha256: sha256(JSON.stringify(files)),
      edges,
    });
  }
  return [...packages.values()].toSorted((a, b) => {
    const left = String(a.manifestPath);
    const right = String(b.manifestPath);
    return left < right ? -1 : left > right ? 1 : 0;
  });
}
