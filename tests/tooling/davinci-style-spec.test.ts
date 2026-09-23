// TS-41 bijection for P4-12a. Every `### Snn —` rule in
// docs/davinci/plan/style-spec.md has
// crates/vize_glyph/tests/style_spec/Snn/{input,output}.vue, and every pair
// has one rule. The orphan case is injected: the checker fails closed on a
// pair the spec does not name. This test does not run the formatter
// (that is P4-12b).

import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const specPath = path.join(repoRoot, "docs/davinci/plan/style-spec.md");
const fixtureRoot = path.join(repoRoot, "crates/vize_glyph/tests/style_spec");

const RULE_HEADING = /^### (S\d{2}) — \S/u;

export type FixtureDir = {
  id: string;
  files: string[];
};

/** Rule ids in document order. A heading that is not `### Snn — Title` is invisible. */
export function ruleIds(spec: string): string[] {
  const ids: string[] = [];
  for (const line of spec.split("\n")) {
    const match = RULE_HEADING.exec(line);
    if (match?.[1]) ids.push(match[1]);
  }
  return ids;
}

/**
 * Spec headings and fixture directories are one set.
 * `dirs` is the directory listing, not a filtered subset: a stray id fails.
 */
export function bijectionViolations(spec: string, dirs: readonly FixtureDir[]): string[] {
  const violations: string[] = [];
  const ids = ruleIds(spec);
  if (ids.length === 0) violations.push("spec has no rules");

  const seen = new Set<string>();
  for (const id of ids) {
    if (seen.has(id)) violations.push(`rule ${id} is duplicated`);
    seen.add(id);
  }

  const byId = new Map<string, FixtureDir>();
  for (const dir of dirs) {
    if (byId.has(dir.id)) violations.push(`fixture ${dir.id} is duplicated`);
    byId.set(dir.id, dir);
  }

  for (const id of seen) {
    const dir = byId.get(id);
    if (!dir) {
      violations.push(`rule ${id} has no fixture pair`);
      continue;
    }
    const files = [...dir.files].sort();
    if (!files.includes("input.vue")) violations.push(`fixture ${id} is missing input.vue`);
    if (!files.includes("output.vue")) violations.push(`fixture ${id} is missing output.vue`);
    for (const file of files) {
      if (file !== "input.vue" && file !== "output.vue") {
        violations.push(`fixture ${id} has extra file ${file}`);
      }
    }
  }

  for (const dir of dirs) {
    if (!seen.has(dir.id)) violations.push(`fixture ${dir.id} has no rule`);
  }

  return violations;
}

function readFixtureDirs(root: string): FixtureDir[] {
  const entries = fs.readdirSync(root, { withFileTypes: true });
  const dirs: FixtureDir[] = [];
  for (const entry of entries) {
    if (!entry.isDirectory()) {
      dirs.push({ id: entry.name, files: [] });
      continue;
    }
    const files = fs.readdirSync(path.join(root, entry.name));
    dirs.push({ id: entry.name, files });
  }
  return dirs;
}

function load(): { spec: string; dirs: FixtureDir[] } {
  return {
    spec: fs.readFileSync(specPath, "utf8"),
    dirs: readFixtureDirs(fixtureRoot),
  };
}

test("every style rule has a fixture pair and every pair has a rule", () => {
  const { spec, dirs } = load();
  const ids = ruleIds(spec);
  assert.deepEqual(bijectionViolations(spec, dirs), []);
  assert.equal(ids.length, 24, "S01–S24");
  assert.deepEqual(
    ids,
    Array.from({ length: 24 }, (_, index) => `S${String(index + 1).padStart(2, "0")}`),
  );

  for (const dir of dirs) {
    for (const name of ["input.vue", "output.vue"]) {
      const text = fs.readFileSync(path.join(fixtureRoot, dir.id, name), "utf8");
      assert.ok(text.length > 0, `${dir.id}/${name} is empty`);
      if (name === "output.vue") {
        assert.ok(text.endsWith("\n"), `${dir.id} output must end in one LF`);
        assert.ok(!text.endsWith("\n\n"), `${dir.id} output has a trailing blank line`);
        assert.equal(text.includes("\r"), false, `${dir.id} output contains CR`);
      }
    }
  }
});

test("an orphan fixture fails the bijection", () => {
  const { spec, dirs } = load();
  const orphan: FixtureDir = { id: "S99", files: ["input.vue", "output.vue"] };
  assert.deepEqual(bijectionViolations(spec, [...dirs, orphan]), ["fixture S99 has no rule"]);
});

test("a rule without a pair fails the bijection", () => {
  const { spec, dirs } = load();
  const withMissing = `${spec}\n### S99 — Missing pair\n`;
  assert.deepEqual(bijectionViolations(withMissing, dirs), ["rule S99 has no fixture pair"]);
});

test("a pair missing output.vue fails the bijection", () => {
  const { spec, dirs } = load();
  const broken = dirs.map((dir) => (dir.id === "S01" ? { id: dir.id, files: ["input.vue"] } : dir));
  assert.deepEqual(bijectionViolations(spec, broken), ["fixture S01 is missing output.vue"]);
});

test("a duplicated rule id fails the bijection", () => {
  const spec = "### S01 — One\n### S01 — Two\n";
  const dirs: FixtureDir[] = [{ id: "S01", files: ["input.vue", "output.vue"] }];
  assert.deepEqual(bijectionViolations(spec, dirs), ["rule S01 is duplicated"]);
});

function outputOf(id: string): string {
  return fs.readFileSync(path.join(fixtureRoot, id, "output.vue"), "utf8");
}

test("the fixtures pin the departures from today's Glyph", () => {
  const blocks = outputOf("S03");
  assert.ok(blocks.indexOf("<style>") < blocks.indexOf("<template>"));
  assert.ok(blocks.indexOf("<template>") < blocks.indexOf("<script"));

  const order = outputOf("S13");
  assert.match(order, /<span id="main" title="t" class="box">/u);
  assert.doesNotMatch(order, /<b id="x"/u);

  const slots = outputOf("S15");
  assert.match(slots, /<Layout v-slot="slotProps">/u);
  assert.match(slots, /<template #header>/u);
  assert.doesNotMatch(slots, /#default/u);

  const closing = outputOf("S11");
  assert.match(closing, /<br \/>/u);
  assert.match(closing, /<div><\/div>/u);
  assert.match(closing, /<UserCard \/>/u);
  assert.match(closing, /<UserCard>\s*<span>A<\/span>/u);
  assert.match(closing, /^ {2}\/>$/mu);

  const wrap = outputOf("S12");
  const title = wrap.split("\n").find((line) => line.includes("title="));
  assert.ok(title && title.length > 100, "a single attribute stays on one line past print width");
  assert.match(wrap, /type="submit"\n {4}class="primary action"/u);

  const covered = outputOf("S23")
    .split("\n")
    .find((line) => line.includes("<input"));
  assert.ok(covered && covered.length > 100, "a suppressed line is not split");
});
