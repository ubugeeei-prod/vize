// usage: node add-linter-case.ts <worktree> <id-suffix> <root-dir-name> <file:role>...
// Appends an input-only linter differential case with computed sha256 and dialect coverage.
import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
const [worktree, idSuffix, root, ...specs] = process.argv.slice(2);
const { sourceInspectedDialectCoverage } = await import(
  path.join(worktree, "tests/differential/source-dialect-coverage.ts")
);
const manifestPath = path.join(worktree, "tests/_fixtures/differential/linter/manifest.json");
const manifest = JSON.parse(fs.readFileSync(manifestPath, "utf8"));
const files = specs.map((spec) => {
  const [file, role] = spec.split(":");
  const bytes = fs.readFileSync(path.join(worktree, "tests/_fixtures/differential/linter", root, file));
  return { path: file, role, sha256: crypto.createHash("sha256").update(bytes).digest("hex"), bytes };
});
const id = `linter/${idSuffix}`;
if (manifest.cases.some((entry: { id: string }) => entry.id === id)) throw new Error(`duplicate ${id}`);
const entry = {
  id,
  state: "active",
  targets: ["lint"],
  inputs: { root, files: files.map(({ path, role, sha256 }) => ({ path, role, sha256 })) },
  dialectCoverage: sourceInspectedDialectCoverage(
    files.map((file) => ({ filename: file.path, source: file.bytes })),
    "linter manifest pinned authored sources",
  ),
};
manifest.cases.push(entry);
manifest.cases.sort((a: { id: string }, b: { id: string }) => (a.id < b.id ? -1 : a.id > b.id ? 1 : 0));
fs.writeFileSync(manifestPath, `${JSON.stringify(manifest, null, 2)}\n`);
console.log(JSON.stringify(entry.dialectCoverage));
