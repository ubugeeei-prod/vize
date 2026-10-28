/** Complete authored input seal, including config and package files outside node_modules. */
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";

export function sealFixtureInputs(root: string) {
  const entries: { path: string; bytes: number; sha256: string }[] = [];
  function visit(directory: string) {
    for (const entry of fs.readdirSync(directory, { withFileTypes: true })) {
      if (entry.name === "node_modules") continue;
      const file = path.join(directory, entry.name);
      if (entry.isDirectory()) visit(file);
      else if (entry.isFile()) {
        const bytes = fs.readFileSync(file);
        entries.push({
          path: path.relative(root, file).replaceAll("\\", "/"),
          bytes: bytes.length,
          sha256: createHash("sha256").update(bytes).digest("hex"),
        });
      }
    }
  }
  visit(root);
  entries.sort((a, b) => a.path.localeCompare(b.path));
  return {
    entries,
    sha256: createHash("sha256").update(JSON.stringify(entries)).digest("hex"),
    files: entries.length,
    bytes: entries.reduce((total, row) => total + row.bytes, 0),
  };
}
