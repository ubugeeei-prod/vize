import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const directory = path.join(root, "crates/vize/tests/fixtures/props-destructuring-original-7988");
const read = (name) => fs.readFileSync(path.join(directory, name), "utf8");
const digest = (bytes) => createHash("sha256").update(bytes).digest("hex");

await test("props style CLI corpus retains whole original inputs, config, command and authored vectors", () => {
  const source = JSON.parse(read("source.json"));
  assert.equal(source.issue, 7988);
  assert.deepEqual(source.author, { login: "ubugeeei", id: 71201308 });
  assert.deepEqual(source.originalInputs, ["MyBadgeA.vue", "MyBadgeB.vue"]);
  assert.equal(source.cases, 12);
  assert.deepEqual(source.formats, ["plain", "json"]);
  assert.equal(source.repetitions, 2);
  for (const pin of source.pins) {
    const bytes = fs.readFileSync(path.join(directory, pin.path));
    assert.equal(bytes.length, pin.bytes, pin.path);
    assert.equal(digest(bytes), pin.sha256, pin.path);
  }
  assert.deepEqual(
    fs
      .readdirSync(directory)
      .filter((name) => name !== "source.json")
      .sort(),
    source.pins.map((pin) => pin.path).sort(),
  );
  const body = read("original-issue.md");
  const inputs = [...body.matchAll(/```vue\n([\s\S]*?)\n```/g)].map((match) => `${match[1]}\n`);
  assert.equal(inputs.length, 2);
  source.originalInputs.forEach((name, index) => assert.equal(read(`${name}.txt`), inputs[index]));
  const config = body.match(/```json\n([\s\S]*?)\n```/)[1];
  assert.equal(read(source.originalConfig), `${config}\n`);
  assert.equal(
    read(source.originalCommand),
    "vize lint -f plain --help-level short MyBadgeA.vue MyBadgeB.vue\n",
  );
  const cases = JSON.parse(read("cases.json"));
  assert.equal(cases.length, source.cases);
  assert.equal(new Set(cases.map((entry) => entry.id)).size, cases.length);
  assert.equal(cases[0].id, "original-default");
  assert.equal(cases[0].config, source.originalConfig);
  for (const entry of cases) {
    const result = JSON.parse(read(entry.json));
    assert.deepEqual(
      result.map((row) => row.file),
      entry.inputs,
    );
    for (const row of result) {
      assert.equal(row.errorCount, row.messages.filter((message) => message.severity === 2).length);
      assert.equal(
        row.warningCount,
        row.messages.filter((message) => message.severity === 1).length,
      );
    }
    assert(read(entry.plain).startsWith("Patina lint report: "));
    assert.equal(entry.exit, Number(result.some((row) => row.errorCount > 0)));
  }
});
