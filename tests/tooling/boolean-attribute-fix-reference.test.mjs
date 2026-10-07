import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { fixedSource, loadCases, wholeJson } from "./boolean-attribute-fix-reference.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

await test("7905 preserves whole originals and independent complete edit controls", () => {
  const { cases, directory } = loadCases(root);
  const shell = fs.readFileSync(path.join(directory, "original-command.sh"), "utf8");
  const original = cases[0];
  assert.equal(
    shell.split("cat > CardList.vue <<'VUE'\n")[1].split("\nVUE")[0] + "\n",
    original.source,
  );
  assert.equal(original.source.replace('disabled="disabled"', "disabled"), original.fixed);
  const requested = fs.readFileSync(
    path.join(directory, "original-requested-all-four.vue.fixture"),
    "utf8",
  );
  assert.notEqual(original.fixed, requested, "the rule-isolated boolean corpus remains unchanged");
  assert.equal(wholeJson(original, false, true)[0].messages.length, 4);
  assert.equal(wholeJson(original, true, true)[0].messages.length, 0);
  const complete = fixedSource(original, true);
  assert.equal(complete.slice(complete.indexOf("<template>")), requested);
  const entries = [...new Set(cases.map((entry) => entry.entry))].sort((left, right) =>
    left < right ? -1 : left > right ? 1 : 0,
  );
  assert.deepEqual(entries, ["html", "jsx", "sfc", "tsx"]);
  for (const entry of cases) {
    assert.deepEqual(Object.keys(entry).sort(), [
      "diagnostics",
      "entry",
      "filename",
      "fixed",
      "id",
      "source",
    ]);
    const actual = wholeJson(entry)[0];
    assert.equal(actual.messages.length, entry.diagnostics.length);
    assert.equal(actual.warningCount, entry.diagnostics.length);
    if (entry.diagnostics.every((finding) => !finding.fix)) assert.equal(entry.fixed, entry.source);
    else assert.notEqual(entry.fixed, entry.source);
  }
  const refusals = cases.filter((entry) => entry.id.startsWith("hidden-enumerated"));
  assert.equal(refusals.length, 3);
  for (const entry of refusals) assert.equal(entry.diagnostics[0].fix, false);
});
