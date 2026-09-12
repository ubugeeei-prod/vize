import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { test } from "node:test";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

function readRepoFile(...segments: string[]): string {
  return fs.readFileSync(path.join(repoRoot, ...segments), "utf8");
}

function sectionBetween(source: string, start: RegExp, end: RegExp, label: string): string {
  const startMatch = start.exec(source);
  assert.ok(startMatch, `missing ${label}`);
  const tail = source.slice(startMatch.index);
  const endMatch = end.exec(tail.slice(startMatch[0].length));
  if (endMatch == null) return tail;
  return tail.slice(0, startMatch[0].length + endMatch.index);
}

function walkFiles(root: string): string[] {
  const files: string[] = [];
  for (const entry of fs.readdirSync(root, { withFileTypes: true })) {
    const full = path.join(root, entry.name);
    if (entry.isDirectory()) {
      if ([".git", "node_modules", "target", "results"].includes(entry.name)) continue;
      files.push(...walkFiles(full));
      continue;
    }
    if (/\.(rs|ts|tsx|js|mjs|ya?ml|toml)$/u.test(entry.name)) files.push(full);
  }
  return files;
}

test("Phase 2 exit gate is fully evaluated and keeps the allocation miss visible", () => {
  const phase = readRepoFile("davinci-road", "plan", "phase-2.md");
  const record = readRepoFile("davinci-road", "plan", "phase-2-records", "p2-20.md");
  const gate = sectionBetween(
    phase,
    /^## Exit gate \(machine-checkable\)/mu,
    /$a/mu,
    "P2 exit gate",
  );
  const items = [...gate.matchAll(/^- \[(?<checked>[ x])\] \*\*(?<title>[^*]+)\*\*/gmu)].map(
    (match) => ({ checked: match.groups!.checked, title: match.groups!.title }),
  );

  assert.equal(items.length, 12);
  assert.deepEqual(
    items.filter((item) => item.checked !== "x"),
    [],
    "P2 exit lines must all be evaluated at P2-20",
  );
  assert.match(gate, /Real Project Matrix run `34682248135`/);
  assert.match(gate, /Recorded miss/);
  assert.match(gate, /dom_compile_allocs_ratio_max = 1\.00/);
  assert.match(gate, /small` 39.*67/);
  assert.match(gate, /stress-interp` 536.*2264/);
  assert.match(record, /bench-compare` exits 1 with six exact allocation breaches/);
  assert.match(record, /The lanes cannot expire silently at the phase boundary/);
});

test("Phase 2 old-path flags are absent from live source roots", () => {
  const oldDomEnv = ["VIZE_DAVINCI_", "DOM"].join("");
  const oldTransformEnv = ["VIZE_DAVINCI_", "TRANSFORM"].join("");
  const forbidden = [
    new RegExp(String.raw`\b${oldDomEnv}\b`, "u"),
    new RegExp(String.raw`\b${oldTransformEnv}\b`, "u"),
    new RegExp(String.raw`\b${["TRANSFORM_", "LANE_FLAG"].join("")}\b`, "u"),
    new RegExp(String.raw`\b${["DOM_", "LANE_FLAG"].join("")}\b`, "u"),
    new RegExp(String.raw`\bdom_lane_selection\b`, "u"),
    new RegExp(String.raw`\bDomLaneSelection\b`, "u"),
    new RegExp(String.raw`\bskipped_legacy_flag\b`, "u"),
  ];
  const roots = ["crates", "tools", ".github"].map((root) => path.join(repoRoot, root));
  const matches: string[] = [];

  for (const file of roots.flatMap(walkFiles)) {
    const text = fs.readFileSync(file, "utf8");
    for (const pattern of forbidden) {
      if (pattern.test(text)) {
        matches.push(`${path.relative(repoRoot, file)} matched ${pattern.source}`);
      }
    }
  }

  assert.deepEqual(matches, []);
});

test("Phase 2 corpus expansion audit records a full hydrated scope proof", () => {
  const coverage = readRepoFile("davinci-road", "plan", "corpus-coverage.md");
  assert.match(coverage, /Hydrated: 142 of 142 manifest projects/);
  assert.match(coverage, /\| \*\*total sites\*\*[^|\n]*(?:\|[^|\n]*){5}\|\s+0 \|/);
  assert.match(coverage, /All manifest projects were hydrated for this run/);
});
