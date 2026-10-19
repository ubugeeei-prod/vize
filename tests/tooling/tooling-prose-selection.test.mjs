import assert from "node:assert/strict";
import { test } from "node:test";
import {
  isProse,
  planToolingTests,
  proseReferences,
  readsProse,
} from "../../tools/support/compat/github/plan-tooling-tests.mjs";

void test("prose references come from literals, joined segments and globs", () => {
  const references = proseReferences(
    [
      'readRepoFile("docs", "content", "guide.md");',
      'const glob = "docs/davinci/plan/**/*.md";',
      "const dir = `docs/content/${language}/`;",
      'const any = "**/*"; const star = "*"; const dot = ".";',
    ].join("\n"),
  );
  assert.ok(references.includes("docs/content/guide.md"));
  assert.ok(references.includes("docs/davinci/plan/**/*.md"));
  assert.ok(references.includes("docs/content/"));
  for (const vague of ["**/*", "*", "."]) assert.ok(!references.includes(vague), vague);
});

void test("a prose path is read when a source names it, a directory above it or a glob over it", () => {
  const read = (path, source) => readsProse(path, [proseReferences(source)]);
  assert.equal(read("AGENTS.md", 'readFileSync("AGENTS.md")'), true);
  assert.equal(read("docs/content/guide/index.md", 'join(root, "docs", "content")'), true);
  assert.equal(read("docs/content/guide/index.md", 'const root = "docs/";'), true);
  assert.equal(read("docs/davinci/plan/x.md", '"docs/davinci/plan/**/*.md"'), true);
  assert.equal(read("docs/content/guide/index.md", '"docs/davinci/plan/**/*.md"'), false);
  assert.equal(read("docs/content/guide/index.md", '"docs-site" "documents"'), false);
  assert.equal(read("AGENTS.md", 'readFileSync("README.md"); const glob = "**/*";'), false);
});

void test("PRs select readers by name, while the merge tier stays complete", () => {
  assert.ok(isProse("docs/content/guide/index.md") && isProse("AGENTS.md"));
  assert.ok(!isProse("docs/scripts/build.ts"));
  const prose = planToolingTests(["docs/content/guide/index.md"]);
  assert.equal(prose.reason, "import closures and named repository paths");
  assert.ok(prose.tests.length > 0 && prose.tests.length < prose.totalTests / 4);
  // The merge runner requires the whole suite, prose-only candidates included.
  const merge = planToolingTests(["docs/content/guide/index.md"], { tier: "merge" });
  assert.equal(merge.tests.length, merge.totalTests);
  // Code is selected the same way, by the tests that name its directory.
  const mixed = planToolingTests(["docs/content/guide/index.md", "tools/support/ci/x.ts"]);
  assert.ok(mixed.tests.length < mixed.totalTests / 4);
  assert.ok(prose.tests.every((file) => mixed.tests.includes(file)));
});
