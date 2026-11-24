import assert from "node:assert/strict";
import { test } from "node:test";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";

import {
  compare,
  parseBaseline,
  renderBaseline,
} from "../../tools/commands/ci/readability/baseline.ts";
import {
  countFile,
  emptyCounts,
  rustTestModules,
  type Counts,
} from "../../tools/commands/ci/readability/counters.ts";
import { countTree, inScope, main } from "../../tools/commands/ci/readability-ratchet.ts";
import { maskScript } from "../../tools/support/readability/script-comments.ts";

const counts = (values: Partial<Counts>): Counts => ({ ...emptyCounts(), ...values });

test("Rust counters see issue comments, restating comments, imports and limits", () => {
  const source = [
    "use crate::model::Node;",
    "use std::fmt;",
    "",
    "// Keep the order stable (#1234).",
    "// TODO(#77): drop the copy once callers borrow.",
    "const LIMIT: usize = 4096;",
    "",
    "pub fn walk(depth: usize) -> bool {",
    "    // Check the depth",
    "    let check = depth > 128;",
    '    let text = "issue 999 and 512 stay strings";',
    "    check && text.len() > LIMIT",
    "}",
    "",
    "#[cfg(test)]",
    "mod tests {",
    "    #[test]",
    "    fn walk_respects_limit_13579() {",
    "        assert!(!super::walk(4096));",
    "    }",
    "}",
    "",
  ].join("\n");
  assert.deepEqual(countFile("crates/demo/src/walk_1234.rs", source, { lines: true }), {
    lines: 0,
    "long-functions": 0,
    "issue-comments": 1,
    "issue-names": 2,
    "restating-comments": 1,
    "mixed-imports": 1,
    "hardcoded-limits": 1,
  });
});

test("function length counts only bodies over 120 lines and ignores declarations", () => {
  const body = Array.from({ length: 121 }, (_, index) => `    let v${index} = ${index};`);
  const source = ["pub trait Walk {", "    fn walk(&self);", "}", "fn long() {", ...body, "}", ""];
  assert.equal(
    countFile("crates/demo/src/a.rs", source.join("\n"), { lines: true })["long-functions"],
    1,
  );
  const short = ["fn short() {", ...body.slice(0, 117), "}", ""].join("\n");
  assert.equal(countFile("crates/demo/src/a.rs", short, { lines: true })["long-functions"], 0);
});

test("script masking keeps template code and hides comments inside literals", () => {
  const masked = maskScript(
    'const url = "http://x/#1234"; // see #4321\nconst re = /\\/*#999/u;\nconst t = `a ${b /* #5678 */} // #8765`;\n',
  );
  assert.deepEqual(
    masked.comments.map((comment) => [comment.line, comment.kind, comment.lines.join("|")]),
    [
      [1, "line", " see #4321"],
      [3, "block", " #5678 "],
    ],
  );
  assert.equal(masked.code.split("\n")[2].includes("${b"), true);
});

test("TypeScript counters group imports by node, packages and relative paths", () => {
  const grouped =
    'import fs from "node:fs";\n\nimport { x } from "vite";\n\nimport { y } from "./y.ts";\n';
  const mixed = 'import { y } from "./y.ts";\nimport fs from "node:fs";\n';
  assert.equal(countFile("npm/a/src/a.ts", grouped, { lines: true })["mixed-imports"], 0);
  assert.equal(countFile("npm/a/src/a.ts", mixed, { lines: true })["mixed-imports"], 1);
  const tests = 'test("keeps #' + '1234 fixed", () => {});\nconst timeout = 30000;\nwait(2500);\n';
  assert.deepEqual(
    [
      countFile("npm/a/src/a.ts", tests, { lines: true })["issue-names"],
      countFile("npm/a/src/a.ts", tests, { lines: true })["hardcoded-limits"],
      countFile("npm/a/src/a.test.ts", tests, { lines: true })["hardcoded-limits"],
    ],
    [1, 1, 0],
  );
});

test("cfg(test) module declarations mark their files as test code", () => {
  assert.deepEqual(
    rustTestModules("crates/a/src/parser.rs", "#[cfg(test)]\nmod witness;\nmod real;\n"),
    ["crates/a/src/parser/witness.rs", "crates/a/src/parser/witness/"],
  );
  assert.deepEqual(
    rustTestModules("crates/a/src/lib.rs", "#[cfg(all(test, unix))]\nmod tests;\n"),
    ["crates/a/src/tests.rs", "crates/a/src/tests/"],
  );
});

test("the scope skips fixtures, generated paths and other languages", () => {
  assert.deepEqual(
    [
      "crates/a/src/lib.rs",
      "crates/a/tests/fixtures/x.rs",
      ".github/scripts/x.ts",
      "upstream/forks/x.ts",
      "docs/content/ja/x.ts",
      "npm/native/index.d.ts",
      "tools/config/pkl/x.pkl",
      "npm/ui/src/a.vue",
    ].map(inScope),
    [true, false, false, false, false, false, false, true],
  );
});

test("the baseline round-trips and drops all-zero rows", () => {
  const files = new Map([
    ["b.rs", counts({ "issue-comments": 2 })],
    ["a.rs", counts({ lines: 600, "mixed-imports": 1 })],
    ["c.rs", counts({})],
  ]);
  const text = renderBaseline(files);
  assert.deepEqual([...parseBaseline(text).keys()], ["a.rs", "b.rs"]);
  assert.deepEqual(parseBaseline(text).get("a.rs"), files.get("a.rs"));
});

test("area totals allow splits and moves and reject new debt", () => {
  const baseline = new Map([
    ["crates/a/src/big.rs", counts({ "issue-comments": 3, lines: 900 })],
    ["crates/b/src/old.rs", counts({ "mixed-imports": 1 })],
  ]);
  const split = new Map([
    ["crates/a/src/big.rs", counts({ "issue-comments": 1, lines: 600 })],
    ["crates/a/src/big/part.rs", counts({ "issue-comments": 2 })],
    ["crates/c/src/old.rs", counts({ "mixed-imports": 1 })],
  ]);
  assert.deepEqual(compare(baseline, split).findings, []);
  assert.equal(compare(baseline, split).lowerable, 2);

  const grown = new Map([
    ["crates/a/src/big.rs", counts({ "issue-comments": 3, lines: 901 })],
    ["crates/a/src/new.rs", counts({ "issue-comments": 1 })],
    ["crates/b/src/old.rs", counts({ "mixed-imports": 1 })],
  ]);
  assert.deepEqual(
    compare(baseline, grown).findings.map((finding) => [
      finding.area,
      finding.counter,
      finding.baseline,
      finding.current,
      finding.files.map((file) => file.path),
    ]),
    [
      ["crates/a", "lines", 900, 901, ["crates/a/src/big.rs"]],
      ["crates/a", "issue-comments", 3, 4, ["crates/a/src/new.rs"]],
    ],
  );
});

test("a moved oversized file keeps its length budget", () => {
  const baseline = new Map([["tests/a/check_cli.rs", counts({ lines: 3750 })]]);
  assert.deepEqual(
    compare(baseline, new Map([["tests/b/check_cli.rs", counts({ lines: 3750 })]])).findings,
    [],
  );
  assert.equal(
    compare(baseline, new Map([["tests/b/check_cli.rs", counts({ lines: 3751 })]])).findings.length,
    1,
  );
});

test("only authored declarations, comment references and script blocks are counted", () => {
  const script = [
    "// TODO(#1234) plus an independent reference to issue #5678",
    "// test('issue #3456', () => {})",
    "const example = \x60test('issue #3456', f)\x60;",
    "i++ / 2; // issue #3457",
    "if (ok) /[/*] #5678/.test(s);",
    "const x = 'line \\" + "\r\n" + "// issue #3458';",
    "test('issue #4567', () => {});",
  ].join("\n");
  const counts = countFile("example.ts", script, { lines: true });
  assert.equal(counts["issue-names"], 1);
  assert.equal(counts["issue-comments"], 3);
  const vue =
    "<template><!-- <script>/* issue #2345 */</script> --><p/></template><script>/* issue #1234 */ const n=1;</script>";
  assert.equal(countFile("example.vue", vue, { lines: true })["issue-comments"], 1);
  const rust = '// #[test] fn case_1234() {}\nconst S: &str = "#[test] fn case_3456() {}";';
  assert.equal(countFile("example.rs", rust, { lines: true })["issue-names"], 0);
});

test("only an initial generated comment excludes a source", (t) => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "readability-header-"));
  t.after(() => fs.rmSync(root, { recursive: true, force: true }));
  const files = ["tools/example.ts", "tools/generated.ts"];
  fs.mkdirSync(path.join(root, "tools"));
  fs.writeFileSync(
    path.join(root, files[0]),
    "const help = 'Do not edit the default profile'; // issue #1234\n",
  );
  fs.writeFileSync(
    path.join(root, files[1]),
    "// Generated by the test fixture.\n// issue #1234\n",
  );
  const counts = countTree(root, files);
  assert.equal(counts.get(files[0])?.["issue-comments"], 1);
  assert.equal(counts.has(files[1]), false);
  assert.equal(main(["--reseed", "--root", root]), 1);
});
