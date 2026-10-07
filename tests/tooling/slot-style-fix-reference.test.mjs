import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { fixedSource, loadCases, sha256, wholeJson } from "./slot-style-fix-reference.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

await test("7905 slot contracts retain whole original carriers, edits and complete unsafe controls", () => {
  const { cases, originals } = loadCases(root);
  const original = cases[0];
  const shell = originals.find((input) => input.file.endsWith("/original-command.sh")).text;
  assert.equal(
    shell.split("cat > CardList.vue <<'VUE'\n")[1].split("\nVUE")[0] + "\n",
    original.source,
  );
  assert.equal(original.fixed, original.source.replace("v-slot:header", "#header"));
  assert.equal(
    fixedSource(original, true),
    original.source
      .replace("v-slot:header", "#header")
      .replace('disabled="disabled"', "disabled")
      .replace("<MyCard></MyCard>", "<MyCard />")
      .replace("<my-card />", "<MyCard />"),
  );
  assert.deepEqual(
    wholeJson(original, false, true)[0].messages.map((finding) => finding.ruleId),
    [
      "vue/html-self-closing",
      "vue/component-name-in-template-casing",
      "vue/v-slot-style",
      "vue/no-boolean-attr-value",
    ],
  );
  assert.deepEqual(
    wholeJson(original, true, true)[0].messages.map((finding) => finding.ruleId),
    [],
  );
  const requested = originals.find((input) =>
    input.file.endsWith("/original-requested-all-four.vue.fixture"),
  ).text;
  const complete = fixedSource(original, true);
  assert.equal(complete.slice(complete.indexOf("<template>")), requested);
  for (const entry of cases) {
    const bytes = Buffer.from(entry.source);
    let output = Buffer.from(bytes);
    for (const finding of [...entry.diagnostics].reverse()) {
      assert.equal(bytes.subarray(finding.start, finding.end).toString("utf8"), finding.target);
      const text = bytes.subarray(0, finding.start).toString("utf8").split("\n");
      const endText = bytes.subarray(0, finding.end).toString("utf8").split("\n");
      assert.deepEqual(finding.utf16, {
        start: { line: text.length - 1, character: text.at(-1).length },
        end: { line: endText.length - 1, character: endText.at(-1).length },
      });
      if (finding.edit) {
        const edit = finding.edit;
        assert.equal(edit.start, finding.start);
        assert.ok(edit.end > edit.start && edit.end <= finding.end);
        output = Buffer.concat([
          output.subarray(0, edit.start),
          Buffer.from(edit.newText),
          output.subarray(edit.end),
        ]);
      }
    }
    assert.equal(output.toString("utf8"), entry.fixed, entry.id);
    assert.equal(wholeJson(entry)[0].warningCount, entry.diagnostics.length);
    assert.equal(
      wholeJson(entry, true)[0].warningCount,
      entry.diagnostics.filter((finding) => !finding.edit).length,
    );
  }
  for (const id of [
    "modifier-refusal",
    "bare-modifier-refusal",
    "pug-authored-edit-refusal",
    "petite-vue-edit-refusal",
  ]) {
    const entry = cases.find((entry) => entry.id === id);
    assert.equal(entry.fixed, entry.source);
    assert.equal(entry.diagnostics.length, 1);
    assert.equal(entry.diagnostics[0].edit, null);
    assert.deepEqual(wholeJson(entry, true), wholeJson(entry));
  }
  const booleanCases = fs.readFileSync(
    path.join(root, "crates/vize_patina/tests/fixtures/issue-7905/cases.json"),
  );
  assert.equal(JSON.parse(booleanCases).length, 21);
  assert.equal(
    sha256(booleanCases),
    "562774614a706183787ec869a164438dff4b8d99e1847c97ffba5b83456cbf6c",
  );
});
