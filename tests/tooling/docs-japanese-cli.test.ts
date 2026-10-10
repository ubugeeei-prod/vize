import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";

const japanese = readFileSync(
  new URL("../../docs/content/ja/guide/cli.md", import.meta.url),
  "utf8",
);
const english = readFileSync(new URL("../../docs/content/guide/cli.md", import.meta.url), "utf8");
const retained = JSON.parse(
  readFileSync(new URL("./fixtures/docs-japanese-cli-blocks.json", import.meta.url), "utf8"),
);

test("Japanese CLI review retains every full published example and current command table", () => {
  for (const block of retained.blocks)
    assert.ok(japanese.includes(block), `missing published block: ${block}`);
  for (const [, block] of english.matchAll(/```[^\n]+\n([\s\S]*?)```/g)) {
    for (const line of block.trim().split("\n"))
      assert.ok(japanese.includes(line), `missing current example: ${line}`);
  }
  for (const [, command] of english.matchAll(/^\| `([^`]+)`\s*\|/gm)) {
    assert.ok(japanese.includes(`| \`${command}\``), `missing command: ${command}`);
  }
  assert.match(japanese, /^## Doctor$/m);
  assert.ok(japanese.includes("Rust `vize check`"));
  for (const group of japanese.matchAll(/(?:^\|[^\n]*\n)+/gm)) {
    for (const row of group[0].trim().split("\n")) assert.equal(row.split("|").length, 4, row);
  }
  assert.match(japanese, /YAML と Markdown の整形は未実装/);
  assert.match(japanese, /終了コード `0`[\s\S]*`1`[\s\S]*`2`/);
});

test("Japanese CLI keeps check and legacy command fragments exactly once", () => {
  for (const id of ["check", "チェック", "建てる", "糸くず", "検査官", "準備ができて", "美術館"]) {
    assert.equal([...japanese.matchAll(new RegExp(`id="${id}"`, "g"))].length, 1, id);
  }
  assert.match(japanese, /<span id="check"><\/span>/);
  assert.match(japanese, /^## 型チェック$/m);
  assert.doesNotMatch(japanese, /^\|[^\n]*錆び/m);
  assert.doesNotMatch(japanese, /(?:^|\n)## (糸くず|検査官|美術館)\n/);
});
