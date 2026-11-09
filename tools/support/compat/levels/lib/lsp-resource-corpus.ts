import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";

const sha256 = (bytes: string | Buffer) => createHash("sha256").update(bytes).digest("hex");

/** Capture authored bytes before launching a measured server, excluding dependencies. */
export function captureLspResourceCorpus(
  workspace: string,
  dependencies: { corsaPath: string; vuePath: string },
) {
  const entries: { path: string; bytes: number; sha256: string }[] = [];
  const walk = (directory: string) => {
    for (const entry of fs.readdirSync(directory, { withFileTypes: true })) {
      // Dependencies and the byte-preserving native gate are harness inputs,
      // not authored corpus bytes.
      if (entry.name === "node_modules" || entry.name === ".vize-corsa-gate") continue;
      const file = path.join(directory, entry.name);
      if (entry.isDirectory()) walk(file);
      else {
        assert.ok(entry.isFile(), "authored fixture entries must be ordinary files");
        const bytes = fs.readFileSync(file);
        entries.push({
          path: path.relative(workspace, file).replaceAll("\\", "/"),
          bytes: bytes.length,
          sha256: sha256(bytes),
        });
      }
    }
  };
  walk(workspace);
  entries.sort((a, b) => a.path.localeCompare(b.path, "en"));
  const vuePackage = fs.readFileSync(path.join(dependencies.vuePath, "package.json"));
  return {
    sha256: sha256(JSON.stringify(entries)),
    entries,
    sourceBytes: entries.reduce((sum, entry) => sum + entry.bytes, 0),
    sfcFiles: entries.filter((entry) => entry.path.endsWith(".vue")).length,
    backendSha256: sha256(fs.readFileSync(dependencies.corsaPath)),
    vuePackageSha256: sha256(vuePackage),
    vueVersion: (JSON.parse(vuePackage.toString("utf8")) as { version: string }).version,
  };
}
