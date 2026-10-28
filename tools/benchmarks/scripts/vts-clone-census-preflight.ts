/** Bench-only preflight: immutable parent, exact overlay, budgets and cfg0 projection. */
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
const parent = process.argv[2];
assert.equal(parent, "09ff11b5a3440381d4f4558652885ec19520e4f1");
const git = (...args: string[]) => {
  const r = spawnSync("git", args, { encoding: "utf8", maxBuffer: 16 * 1024 * 1024 });
  assert.equal(r.status, 0, r.stderr);
  return r.stdout.trim();
};
assert.equal(git("status", "--porcelain", "--untracked-files=all"), "");
git("merge-base", "--is-ancestor", parent, "HEAD");
const rust = [
  "crates/vize_canon/src/batch/virtual_project/dependency_scan.rs",
  "crates/vize_canon/src/batch/import_rewriter/collect.rs",
  "crates/vize_canon/src/batch/import_rewriter.rs",
  "crates/vize_canon/src/batch/import_rewriter/facts.rs",
  "crates/vize_croquis/src/script_parser/recovery.rs",
];
const expected = [
  ...rust,
  "tools/benchmarks/scripts/vts-clone-census.ts",
  "tools/benchmarks/scripts/vts-clone-census-fixture.ts",
  "tools/benchmarks/scripts/vts-clone-census-profile.ts",
  "tools/benchmarks/scripts/vts-clone-census-preflight.ts",
  "tests/tooling/vts-clone-census.test.ts",
  ".github/workflows/cold-typechecker-scale.yml",
].toSorted();
const changed = git("diff", "--name-only", parent, "HEAD").split("\n").filter(Boolean).toSorted();
assert.deepEqual(changed, expected, "unexpected source or workflow change");
function eraseInstrumentation(source: string) {
  const marker = /vize_carton::micro_(count|span)!\s*\(/gu;
  for (let match; (match = marker.exec(source));) {
    const start = match.index,
      open = marker.lastIndex - 1;
    let depth = 0,
      quoted = false,
      escaped = false,
      comma = -1,
      end = -1;
    for (let i = open; i < source.length; i++) {
      const ch = source[i];
      if (quoted) {
        if (escaped) escaped = false;
        else if (ch === "\\") escaped = true;
        else if (ch === '"') quoted = false;
        continue;
      }
      if (ch === '"') {
        quoted = true;
        continue;
      }
      if ("([{".includes(ch)) depth++;
      if (")]}".includes(ch)) depth--;
      if (ch === "," && depth === 1 && comma === -1) comma = i;
      if (depth === 0) {
        end = i;
        break;
      }
    }
    assert.ok(comma >= 0 && end > comma, "incomplete profiling macro");
    let replacement = "";
    if (match[1] === "span")
      replacement = source
        .slice(comma + 1, end)
        .trim()
        .replace(/,$/u, "")
        .trim();
    else {
      let tail = end + 1;
      while (/\s/u.test(source[tail] ?? "") && tail < source.length) tail++;
      assert.equal(source[tail], ";");
      end = tail;
    }
    source = source.slice(0, start) + replacement + source.slice(end + 1);
    marker.lastIndex = start + replacement.length;
  }
  return source;
}
const normal = (text: string) =>
  text
    .replace(/\s+/gu, "")
    .replace(/,\)/gu, ")")
    .replace(
      /Some\(facts\)=>\{(facts\.collect_occurrences\(&virtual_content,source_type\))\}/gu,
      "Some(facts)=>$1,",
    )
    .replace(
      /None=>\{(self\.rewriter\(\)\.collect_all_specifier_occurrences\(&virtual_content,source_type\))\}/gu,
      "None=>$1,",
    );
for (const file of rust) {
  const body = fs.readFileSync(file, "utf8"),
    base = git("show", parent + ":" + file);
  let projected = normal(eraseInstrumentation(body));
  if (file.endsWith("/dependency_scan.rs")) {
    const binding = "letcontent=self.module_source(file).unwrap_or(&file.content);";
    assert.equal(projected.split(binding).length, 2);
    projected = projected.replace(binding, "");
    assert.equal(projected.split("content.clone()").length, 2);
    projected = projected.replace(
      "content.clone()",
      "self.module_source(file).unwrap_or(&file.content).clone()",
    );
  }
  const expected = normal(base);
  if (projected !== expected) {
    let offset = 0;
    while (
      offset < Math.min(projected.length, expected.length) &&
      projected[offset] === expected[offset]
    )
      offset++;
    assert.fail(
      file +
        ": normal projection differs at " +
        offset +
        " actual=" +
        projected.slice(offset, offset + 120) +
        " expected=" +
        expected.slice(offset, offset + 120),
    );
  }
}
for (const file of changed) {
  const lines = fs.readFileSync(file, "utf8").split("\n").length;
  assert.ok(lines <= (file.endsWith(".rs") ? 500 : 350), file + ": source budget");
  assert.ok(!/\.(?:mjs|cjs|js|jsx|py|sh)$/u.test(file), "retired tooling source");
}
const schema = JSON.parse(fs.readFileSync("docs/davinci/plan/profile-export.schema.json", "utf8"));
assert.equal(schema.properties.schema_version.const, 1);
assert.ok(schema.properties.micro && schema.properties.spans);
console.log(
  JSON.stringify({
    parent,
    changed,
    normalOperandProjectionMatchesParent: true,
    sourceClean: true,
    budgetsPassed: true,
    retiredToolingFiles: 0,
    profileSchemaVersion: 1,
  }),
);
