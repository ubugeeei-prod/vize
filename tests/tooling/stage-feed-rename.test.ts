import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { mkdirSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { eligible, moves, rewrite, run } from "../../tools/support/levels/rename-stage-feed.ts";

const old = ["spol", "vero"].join("");
const Old = ["Spol", "vero"].join("");

test("live sources and docs indexes carry no stage feed codename", () => {
  assert.deepEqual(run("check"), []);
});

test("the rename maps the member, identifiers, span key and prose", () => {
  const rust = `// ${Old} feed in \`${old}_payload.rs\`\nlet ${old} = &json["${old}"]; // the ${Old} feed`;
  assert.equal(
    rewrite("crates/a/src/lib.rs", rust),
    `// Stage feed in \`stage_feed_payload.rs\`\nlet stage_feed = &json["stageFeed"]; // the stage feed`,
  );
  const ts = `{ ${old}: raw.${old}, ${old}Profile, key: "levels.${old}.step" } // the ${Old} feed`;
  assert.equal(
    rewrite("playground/src/a.ts", ts),
    `{ stageFeed: raw.stageFeed, stageProfile, key: "stages.step" } // the stage feed`,
  );
  assert.equal(rewrite("crates/a/b.snap", `#inspector=%22${old}%22`), "#inspector=%22stageFeed%22");
  assert.equal(
    rewrite("playground/src/a.ts", rewrite("playground/src/a.ts", ts)),
    rewrite("playground/src/a.ts", ts),
  );
});

test("docs keep historical prose and frozen custody keeps its bytes", () => {
  const doc = `The ${Old} feed (\`${old}-feed.schema.json\`), \`--test ${old}_payload\``;
  assert.equal(
    rewrite("docs/davinci/plan/remarks-format.md", doc),
    `The ${Old} feed (\`stage-ladder-feed.schema.json\`), \`--test ${old}_payload\``,
  );
  assert.equal(
    rewrite("docs/davinci/plan/test-suites.md", doc),
    `The ${Old} feed (\`stage-ladder-feed.schema.json\`), \`--test stage_feed_payload\``,
  );
  assert.equal(eligible("crates/vize_curator/src/inspector/payload.rs"), true);
  assert.equal(eligible("tests/_fixtures/tooling/moonbit-pinned-transport/a.log"), false);
  assert.equal(eligible("tools/support/cross_file/async_component/public-before.json.txt"), false);
});

test("an unmapped spelling in a live file fails loudly", (context) => {
  const cwd = mkdtempSync(path.join(os.tmpdir(), "vize-stage-feed-rename-"));
  context.after(() => rmSync(cwd, { recursive: true, force: true }));
  const write = (relative: string, text: string) => {
    mkdirSync(path.dirname(path.join(cwd, relative)), { recursive: true });
    writeFileSync(path.join(cwd, relative), text);
  };
  for (const [, to] of moves) write(to, "");
  write("crates/a/src/lib.rs", `//! ${Old} negotiates.\n`);
  execFileSync("git", ["init", "-q"], { cwd });
  execFileSync("git", ["add", "."], { cwd });
  assert.throws(() => run("check", cwd), /Unmapped spellings remain in:\ncrates\/a\/src\/lib\.rs/u);
  write("crates/a/src/lib.rs", `//! The ${Old} feed.\n`);
  assert.deepEqual(run("check", cwd), ["crates/a/src/lib.rs"]);
});
