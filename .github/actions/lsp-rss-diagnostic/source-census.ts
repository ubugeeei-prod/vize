import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";

const output = process.env["EVIDENCE"];
assert(typeof output === "string" && output.length > 0);
const root = "tests/_fixtures/_git/misskey/packages/frontend";
const suffixes = new Set([".vue", ".ts", ".tsx", ".mts", ".cts", ".js", ".jsx", ".mjs", ".cjs"]);
const excluded = new Set([".git", "node_modules", "target", "coverage", "dist"]);
const producers = ["defineEmits", "defineSlots", "emits", "slots", "$emit", "Emit"];
const counts: Record<string, number> = {};
const categories: Record<string, number> = {};
for (const name of [...producers, "backslash", "at_sign", "none"]) categories[name] = 0;
function increment(bank: Record<string, number>, name: string): void {
  bank[name] = (bank[name] ?? 0) + 1;
}
function walk(directory: string): void {
  for (const entry of fs.readdirSync(directory, { withFileTypes: true })) {
    if (entry.isSymbolicLink()) continue;
    const file = path.join(directory, entry.name);
    if (entry.isDirectory()) {
      if (!excluded.has(entry.name)) walk(file);
      continue;
    }
    if (!entry.isFile()) continue;
    const suffix = path.extname(entry.name);
    if (!suffixes.has(suffix)) continue;
    increment(counts, suffix);
    if (suffix !== ".vue") continue;
    const text = fs.readFileSync(file, "utf8");
    let seen = false;
    for (const name of producers) {
      if (text.includes(name)) { increment(categories, name); seen = true; }
    }
    if (text.includes("\\")) increment(categories, "backslash");
    if (text.includes("@")) increment(categories, "at_sign");
    if (!seen && !text.includes("\\") && !text.includes("@")) increment(categories, "none");
  }
}
walk(root);
fs.writeFileSync(path.join(output, "source-census.json"), JSON.stringify({
  scope: "plain frontend filesystem walk; excludes .git/node_modules/target/coverage/dist and symlinks",
  filesystemExtensionCounts: counts, supplementaryRawWholeVuePresence: categories,
  limitation: "not WalkBuilder, hidden/gitignore parity or exact runtime index; raw counts are not setup parsing, skipped Drawer calls or producer proof",
}, null, 2) + "\n");
