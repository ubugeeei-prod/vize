import assert from "node:assert/strict";
import { test } from "node:test";

import {
  keepsBytes,
  knownPaths,
  levelHits,
  plannedMoves,
  renamePath,
  renameWord,
  rewriteText,
  run,
} from "../../tools/support/levels/rename-codenames.ts";

// Spelled in parts so the rename never rewrites this file.
const name = ["da", "vinci"].join("");
const Name = name[0].toUpperCase() + name.slice(1);

void test("no tracked path or line outside the kept docs still carries a codename", () => {
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

void test("the capitalized name in prose becomes a level modifier, a plan item the plan", () => {
  assert.equal(
    rewriteText("crates/a/src/lib.rs", `// ${Name} lane, the ${Name} lane (${Name} P1-7); by ${Name} P2`),
    "// Level lane, the level lane (plan P1-7); by plan P2",
  );
  assert.equal(rewriteText(".github/workflows/x.yml", `name: ${Name} Kani`), "name: Level Kani");
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
      `https://github.com/ubugeeei-prod/vize/tree/main/levels/vize_l0 and level lane, levels.expr.parses`,
      `[guide](../${name}/roadmap.md) [crate](../../levels/vize_l1/src/lib.rs)`,
    ].join("\n"),
  );
});

void test("kept docs keep prose and names; references to moved tracked paths follow them", () => {
  const known = knownPaths([`${name}/vize_l0/src/lib.rs`, `tests/tooling/${name}-x.test.ts`]);
  const source = `${Name} keeps its prose; [l0](../../${name}/vize_l0/src/lib.rs), tests/tooling/${name}-x.test.ts, tests/${name}_gone.rs`;
  assert.equal(
    rewriteText(`docs/${name}/README.md`, source, undefined, known),
    `${Name} keeps its prose; [l0](../../levels/vize_l0/src/lib.rs), tests/tooling/level-x.test.ts, tests/${name}_gone.rs`,
  );
  // A replay after the move still finds the moved path through its new name.
  const moved = knownPaths(["levels/vize_l0/src/lib.rs"]);
  assert.equal(
    rewriteText(`docs/${name}/plan/x.md`, `see ${name}/vize_l0/src/lib.rs.`, undefined, moved),
    "see levels/vize_l0/src/lib.rs.",
  );
  assert.equal(rewriteText(`docs/${name}/README.md`, source), source);
});

void test("kept Markdown keeps code spans; kept plan data changes fully, records keep bytes", () => {
  const page = `${Name} prose \`${name}_harness\`\n\`\`\`sh\ncargo test --test ${name}_ts31\n\`\`\`\n`;
  assert.equal(rewriteText(`docs/${name}/plan/test-suites.md`, page, undefined, new Set()), page);
  assert.equal(
    rewriteText(`docs/${name}/plan/budgets.toml`, `${name}_bench = 1`),
    "level_bench = 1",
  );
  const history = `x\t${name}_bench\n`;
  assert.equal(rewriteText(`docs/${name}/plan/compiler-fix-history.tsv`, history), history);
  assert.equal(rewriteText(`docs/${name}/decisions/x.profile.json`, history), history);
});

void test("escaped docs paths and ANSI colour prefixes keep their boundaries", () => {
  assert.equal(
    rewriteText("tests/a.test.ts", `/docs\\/${name}\\/plan/ "\\x1B[32m${name} DOM sweep"`),
    `/docs\\/${name}\\/plan/ "\\x1B[32mlevels DOM sweep"`,
  );
});

// Level codenames, spelled in parts for the same reason.
const [dump, l1ToL2, l2, l3, stages] = [
  ["fo", "lio"],
  ["rical", "co"],
  ["dise", "gno"],
  ["imp", "eto"],
  ["spol", "vero"],
].map((parts) => parts.join(""));
const title = (word: string) => word[0].toUpperCase() + word.slice(1);

void test("a level codename becomes its level name in every case and separator", () => {
  assert.equal(renameWord(dump), "dump");
  assert.equal(renameWord(title(dump)), "Dump");
  assert.equal(renameWord(`s3-values-${dump}`), "s3-values-dump");
  assert.equal(renameWord(`CROQUIS_${dump.toUpperCase()}`), "CROQUIS_DUMP");
  assert.equal(renameWord(`vize_${l1ToL2}`), "vize_l1_to_l2");
  assert.equal(renameWord(`${l1ToL2}-lane`), "l1-to-l2-lane");
  assert.equal(renameWord(`${title(l1ToL2)}Pass`), "L1ToL2Pass");
  assert.equal(renameWord(`${title(l2)}${title(dump)}`), "L2Dump");
  assert.equal(renameWord(`${l3}-ops`), "l3-ops");
  assert.equal(renameWord(stages), "stages");
  assert.equal(renameWord(`${title(stages)}Feed`), "StageFeed");
  assert.equal(renameWord(`${name}-${l3}`), "level-l3");
});

void test("a level codename counts only where a word part starts", () => {
  assert.deepEqual(levelHits("historicalComplete portfolio"), []);
  assert.equal(renameWord("historicalComplete"), "historicalComplete");
  assert.equal(renameWord("portfolio"), "portfolio");
  assert.deepEqual(levelHits(`L3${title(l3)} ${dump}s check_${dump}`), [l3, dump, dump]);
});

void test("alias lines and docs keep level codenames; code and file paths follow", () => {
  assert.equal(
    rewriteText(
      "levels/vize_l3/src/lib.rs",
      `//! L3 (codename: ${title(l3)})\n//! ${title(l3)} ops\n`,
    ),
    `//! L3 (codename: ${title(l3)})\n//! L3 ops\n`,
  );
  assert.equal(
    rewriteText(
      "docs/content/guide.md",
      `${title(l3)} lowering, \`${l3}_ops\`, [x](./${dump}-format.md)`,
    ),
    `${title(l3)} lowering, \`${l3}_ops\`, [x](./dump-format.md)`,
  );
  assert.equal(
    renamePath(`docs/${name}/plan/${dump}-format.md`),
    `docs/${name}/plan/dump-format.md`,
  );
});

void test("docs data follows the level names; docs Markdown keeps them but follows file names", () => {
  assert.equal(
    rewriteText(`docs/${name}/plan/remarks.schema.json`, `"the [remarks] ${dump} page"`),
    `"the [remarks] dump page"`,
  );
  assert.equal(
    rewriteText(
      `docs/${name}/plan/remarks-format.md`,
      `${title(l3)} remarks, \`tests/_fixtures/${name}-remarks-baseline.${dump}\`, ${title(dump)}.${dump}s`,
      undefined,
      knownPaths([`tests/_fixtures/level-remarks-baseline.dump`]),
    ),
    `${title(l3)} remarks, \`tests/_fixtures/level-remarks-baseline.dump\`, ${title(dump)}.${dump}s`,
  );
});

void test("fixed words rename to what they name", () => {
  assert.equal(renameWord(`${name}-differential`), "legacy-differential");
  assert.equal(renameWord(`${name}-dom-differential`), "legacy-dom-differential");
  assert.equal(renameWord(`vize_${name}`), "vize_l0");
  assert.equal(renameWord(`vize_${name}_derive`), "vize_l0_derive");
});

void test("captured output keeps its bytes unless a live test parses it", () => {
  assert.equal(keepsBytes("crates/a/tests/fixtures/fix-history/capture-1/cargo.jsonl"), true);
  assert.equal(keepsBytes("tests/_fixtures/tooling/run/original-build.log"), true);
  assert.equal(keepsBytes("tests/_fixtures/a/before-armature.log.txt"), true);
  assert.equal(keepsBytes("crates/a/tests/fixtures/proof/stderr.txt"), true);
  assert.equal(keepsBytes("tests/tooling/fixtures/canonical-observer-logs/dom.log"), false);
  assert.equal(keepsBytes("tests/fixtures/l3-reference/slot-reference.lowered.jsonl"), false);
  assert.equal(keepsBytes("crates/vize/tests/fixtures/dump_cli/elided.stderr"), false);
  assert.equal(keepsBytes("editors/vscode/test/css-hover-source-custody.json"), true);
  assert.equal(keepsBytes("tests/_fixtures/tooling/frozen-source/base/Cargo.toml.txt"), true);
  assert.equal(keepsBytes("tests/tooling/fixtures/runner-pool/check.before.yml.txt"), true);
  assert.equal(keepsBytes("tests/_fixtures/a/source-custody/ledger-after.tsv.fixture"), true);
  assert.equal(keepsBytes("tests/_fixtures/a/notes.txt"), false);
});
