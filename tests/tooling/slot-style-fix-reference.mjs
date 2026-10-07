import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";

export const RULE = "vue/v-slot-style";
export const sha256 = (bytes) => createHash("sha256").update(bytes).digest("hex");
export const fixturePath = "crates/vize_patina/tests/fixtures/issue-7905-slot-style";

export function loadCases(root) {
  const directory = path.join(root, fixturePath);
  const manifest = JSON.parse(fs.readFileSync(path.join(directory, "source.json"), "utf8"));
  assert.equal(manifest.issue, 7905);
  assert.equal(manifest.rule, RULE);
  assert.deepEqual(manifest.author, {
    login: "ubugeeei",
    id: 71201308,
    coAuthor: "ubugeeei <71201308+ubugeeei@users.noreply.github.com>",
  });
  const originals = manifest.inputs.map((input) => {
    const bytes = fs.readFileSync(path.join(directory, input.file));
    assert.equal(bytes.length, input.bytes, input.file);
    assert.equal(sha256(bytes), input.sha256, input.file);
    return { ...input, text: bytes.toString("utf8") };
  });
  assert.equal(originals.length, 7);
  assert.equal(originals[0].sha256, manifest.bodySha256);
  const bytes = fs.readFileSync(path.join(directory, manifest.cases.file));
  assert.equal(bytes.length, manifest.cases.bytes);
  assert.equal(sha256(bytes), manifest.cases.sha256);
  const cases = JSON.parse(bytes);
  assert.equal(cases.length, manifest.cases.count);
  assert.equal(cases.length, 29);
  assert.equal(new Set(cases.map((entry) => entry.id)).size, cases.length);
  assert.equal(
    cases[0].source,
    originals.find((input) => input.file.endsWith("/CardList.vue.fixture")).text,
  );
  return { directory, manifest, originals, cases };
}

export function fixedSource(entry, originalRules = false) {
  return originalRules ? entry.fixed.replace('disabled="disabled"', "disabled") : entry.fixed;
}

function message(source, target, rule, text, authoredStart = null) {
  if (authoredStart === null) assert.equal(source.split(target).length - 1, 1, target);
  const start = authoredStart ?? source.indexOf(target);
  assert.ok(start >= 0, target);
  assert.equal(source.slice(start, start + target.length), target);
  const from = source.slice(0, start).split("\n");
  const to = source.slice(0, start + target.length).split("\n");
  return {
    ruleId: rule,
    ruleDocsPath: "docs/content/rules/vue.md",
    severity: 1,
    message: `[vize:${rule}] ${text}`,
    line: from.length,
    column: [...from.at(-1)].length + 1,
    endLine: to.length,
    endColumn: [...to.at(-1)].length + 1,
  };
}

export function wholeJson(entry, after = false, originalRules = false) {
  const source = after ? fixedSource(entry, originalRules) : entry.source;
  const messages = [];
  if (originalRules)
    messages.push(
      message(
        source,
        "<MyCard>",
        "vue/html-self-closing",
        "Empty component should be self-closing",
        source.indexOf("<MyCard></MyCard>"),
      ),
      message(
        source,
        "<my-card />",
        "vue/component-name-in-template-casing",
        "Component should use PascalCase",
      ),
    );
  for (const finding of entry.diagnostics) {
    if (after && finding.edit) continue;
    messages.push(message(source, finding.target, RULE, finding.message));
  }
  if (originalRules && !after)
    messages.push(
      message(
        source,
        'disabled="disabled"',
        "vue/no-boolean-attr-value",
        'Boolean attribute "disabled" should not have value "disabled"',
      ),
    );
  return [{ file: entry.filename, messages, errorCount: 0, warningCount: messages.length }];
}
