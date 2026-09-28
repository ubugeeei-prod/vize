#!/usr/bin/env node
// Re-run on main after a conflict; the move and references are bounded and idempotent.
import { existsSync, mkdirSync, readFileSync, renameSync, writeFileSync } from "node:fs";

const oldPath = ".github/workflows/davinci-moonbit.yml";
const newPath = ".github/workflows/level-moonbit.yml";
if (existsSync(oldPath)) {
  if (existsSync(newPath)) throw new Error(`both workflow paths exist: ${oldPath}, ${newPath}`);
  mkdirSync(".github/workflows", { recursive: true });
  renameSync(oldPath, newPath);
}
if (!existsSync(newPath)) throw new Error(`missing workflow: ${newPath}`);

const replacements = [
  [newPath, "name: Davinci MoonBit", "name: Level MoonBit"],
  [newPath, oldPath, newPath],
  [newPath, "davinci-moonbit-${{", "level-moonbit-${{"],
  [newPath, "key: davinci-moonbit", "key: level-moonbit"],
  [
    "tests/tooling/github-workflows-check-gate.test.ts",
    '"davinci-moonbit.yml"',
    '"level-moonbit.yml"',
  ],
];

for (const [path, before, after] of replacements) {
  const source = readFileSync(path, "utf8");
  const matches = source.split(before).length - 1;
  if (matches > 1) throw new Error(`ambiguous replacement (${matches}): ${path} ${before}`);
  if (matches === 0 && !source.includes(after))
    throw new Error(`missing replacement: ${path} ${before}`);
  if (matches === 1) writeFileSync(path, source.replace(before, after));
}
