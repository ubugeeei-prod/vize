// usage: merge-manifest.mjs <worktree> <branch>... — adds each branch's new linter cases to the worktree manifest.
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
const [worktree, ...branches] = process.argv.slice(2);
const rel = "tests/_fixtures/differential/linter/manifest.json";
const file = path.join(worktree, rel);
const git = (...args) => execFileSync("git", ["-C", worktree, ...args], { encoding: "utf8", maxBuffer: 1 << 28 });
const manifest = JSON.parse(fs.readFileSync(file, "utf8"));
const have = new Set(manifest.cases.map((entry) => entry.id));
for (const branch of branches) {
  const base = git("merge-base", branch, "origin/main").trim();
  const before = new Set(JSON.parse(git("show", `${base}:${rel}`)).cases.map((entry) => entry.id));
  for (const entry of JSON.parse(git("show", `${branch}:${rel}`)).cases)
    if (!before.has(entry.id) && !have.has(entry.id)) {
      manifest.cases.push(entry);
      have.add(entry.id);
      console.log(`added ${entry.id}`);
    }
}
manifest.cases.sort((a, b) => (a.id < b.id ? -1 : a.id > b.id ? 1 : 0));
fs.writeFileSync(file, `${JSON.stringify(manifest, null, 2)}\n`);
