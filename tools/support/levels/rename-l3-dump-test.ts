import { execFileSync } from "node:child_process";
import { existsSync, readFileSync, writeFileSync } from "node:fs";
import { resolve } from "node:path";
import { fileURLToPath } from "node:url";

const root = fileURLToPath(new URL("../../../", import.meta.url));
const oldPath = "davinci/vize_l3/tests/folio_laws.rs";
const newPath = "davinci/vize_l3/tests/dump_laws.rs";
const phase = process.argv[2];
if (!["moves", "integrate", "check"].includes(phase)) {
  throw new Error("Usage: rename-l3-dump-test.ts moves|integrate|check");
}
const hasOld = existsSync(resolve(root, oldPath));
const hasNew = existsSync(resolve(root, newPath));
if (hasOld === hasNew) throw new Error(`Expected exactly one of ${oldPath} and ${newPath}`);
if (phase !== "moves" && hasOld) throw new Error(`Unmoved target: ${oldPath}`);
if (phase === "moves") {
  if (hasOld) execFileSync("git", ["mv", oldPath, newPath], { cwd: root });
} else {
  const path = resolve(root, "docs/davinci/decisions/2026-09-27-level-restructure.md");
  const source = readFileSync(path, "utf8");
  const original = "the remaining tooling names stay open.";
  const link = "[L3 dump law naming](./2026-10-04-l3-dump-test-name.md)";
  const addition = ` ${link} moves one native auto-discovered test target byte-for-byte, preserves its two round-trip laws and all historical/wire bytes, and requires fresh current-target Actions plus the full protected queue.`;
  if (!existsSync(resolve(root, "docs/davinci/decisions/2026-10-04-l3-dump-test-name.md"))) {
    throw new Error("Missing paired L3 naming decision");
  }
  if (source.includes(link)) {
    if (!source.includes(original + addition)) throw new Error("Unexpected L3 naming decision");
  } else {
    if (source.split(original).length !== 2) throw new Error("Ambiguous naming decision anchor");
    if (phase === "check") throw new Error("Unintegrated L3 naming decision");
    writeFileSync(path, source.replace(original, original + addition));
  }
}
