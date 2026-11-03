import assert from "node:assert/strict";
import { test } from "node:test";

import {
  plannedMoves,
  renamePath,
  renameWord,
  rewriteText,
  run,
} from "../../tools/support/levels/rename-program-codename.ts";

// Spelled in parts so the rename never rewrites this file.
const name = ["da", "vinci"].join("");
const Name = name[0].toUpperCase() + name.slice(1);

void test("no tracked path or line outside the kept docs still carries the program codename", () => {
  assert.deepEqual(run("--check"), []);
});

void test("a whole word becomes levels, an embedded one level, case and separators kept", () => {
  assert.equal(renameWord(name), "levels");
  assert.equal(renameWord(Name), "Levels");
  assert.equal(renameWord(name.toUpperCase()), "LEVELS");
  assert.equal(renameWord(`${name}_harness`), "level_harness");
  assert.equal(renameWord(`VIZE_${name.toUpperCase()}_CORPUS`), "VIZE_LEVEL_CORPUS");
  assert.equal(renameWord(`${Name}Feed`), "LevelFeed");
  assert.equal(renameWord(`${name}-level-dependencies`), "level-dependencies");
  assert.equal(renameWord(`${Name}LevelMap`), "LevelMap");
});

void test("paths move except under the kept docs directory", () => {
  assert.equal(renamePath(`${name}/vize_l2/src/lib.rs`), "levels/vize_l2/src/lib.rs");
  assert.equal(
    renamePath(`tests/tooling/${name}-budgets.test.ts`),
    "tests/tooling/level-budgets.test.ts",
  );
  assert.equal(renamePath(`crates/vize/benches/${name}.rs`), "crates/vize/benches/levels.rs");
  assert.equal(renamePath(`docs/${name}/plan/budgets.toml`), `docs/${name}/plan/budgets.toml`);
  assert.throws(
    () => plannedMoves([`a/${name}-x.ts`, "a/level-x.ts"]),
    /both map to a\/level-x\.ts/,
  );
});

void test("rewrites keep the docs directory, pinned links and links that resolve into the docs", () => {
  const pinned = `https://github.com/ubugeeei-prod/vize/blob/${"a".repeat(40)}/${name}/vize_l0/src/lib.rs`;
  const main = `https://github.com/ubugeeei-prod/vize/tree/main/${name}/vize_l0`;
  const source = [
    `see docs/${name}/README.md and ${pinned}`,
    `${main} and ${Name} lane, ${name}.expr.parses`,
    `[guide](../${name}/roadmap.md) [crate](../../${name}/vize_l1/src/lib.rs)`,
  ].join("\n");
  assert.equal(
    rewriteText("docs/content/guide.md", source),
    [
      `see docs/${name}/README.md and ${pinned}`,
      `https://github.com/ubugeeei-prod/vize/tree/main/levels/vize_l0 and Levels lane, levels.expr.parses`,
      `[guide](../${name}/roadmap.md) [crate](../../levels/vize_l1/src/lib.rs)`,
    ].join("\n"),
  );
});

void test("kept Markdown prose keeps the program name; links to moved paths follow them", () => {
  const source = `${Name} keeps its prose; [l0](../../${name}/vize_l0/src/lib.rs), tests/tooling/${name}-x.test.ts`;
  assert.equal(
    rewriteText(`docs/${name}/README.md`, source),
    `${Name} keeps its prose; [l0](../../levels/vize_l0/src/lib.rs), tests/tooling/level-x.test.ts`,
  );
});

void test("kept Markdown changes code spans and paths, kept data files change fully", () => {
  const page = `${Name} prose \`${name}_harness\`\n\`\`\`sh\ncargo test --test ${name}_ts31\n\`\`\`\n`;
  assert.equal(
    rewriteText(`docs/${name}/plan/test-suites.md`, page),
    `${Name} prose \`level_harness\`\n\`\`\`sh\ncargo test --test level_ts31\n\`\`\`\n`,
  );
  assert.equal(
    rewriteText(`docs/${name}/plan/budgets.toml`, `${name}_bench = 1`),
    "level_bench = 1",
  );
});

void test("escaped docs paths and ANSI colour prefixes keep their boundaries", () => {
  assert.equal(
    rewriteText("tests/a.test.ts", `/docs\\/${name}\\/plan/ "\\x1B[32m${name} DOM sweep"`),
    `/docs\\/${name}\\/plan/ "\\x1B[32mlevels DOM sweep"`,
  );
});
