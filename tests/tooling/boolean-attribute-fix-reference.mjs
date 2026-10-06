import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";

export const RULE = "vue/no-boolean-attr-value";
export const sha256 = (bytes) => createHash("sha256").update(bytes).digest("hex");
export const fixturePath = "crates/vize_patina/tests/fixtures/issue-7905";

export function loadCases(root) {
  const directory = path.join(root, fixturePath);
  const manifest = JSON.parse(fs.readFileSync(path.join(directory, "source.json"), "utf8"));
  assert.equal(manifest.issue, 7905);
  assert.deepEqual(manifest.author, {
    login: "ubugeeei",
    id: 71201308,
    coAuthor: "ubugeeei <71201308+ubugeeei@users.noreply.github.com>",
  });
  for (const input of [...manifest.inputs, manifest.cases]) {
    const bytes = fs.readFileSync(path.join(directory, input.file));
    assert.equal(bytes.length, input.bytes, input.file);
    assert.equal(sha256(bytes), input.sha256, input.file);
  }
  const body = fs.readFileSync(path.join(directory, "original-issue.md"), "utf8");
  assert.equal(sha256(body), manifest.bodySha256);
  const cases = JSON.parse(fs.readFileSync(path.join(directory, manifest.cases.file), "utf8"));
  assert.equal(cases.length, 21);
  assert.equal(cases.length, manifest.cases.count);
  assert.equal(new Set(cases.map((entry) => entry.id)).size, cases.length);
  assert.equal(
    cases[0].source,
    fs.readFileSync(path.join(directory, "CardList.vue.fixture"), "utf8"),
  );
  return { cases, directory, manifest };
}

function location(source, offset) {
  const lines = source.slice(0, offset).split("\n");
  return { line: lines.length, column: [...lines.at(-1)].length + 1 };
}

function message(source, target, rule, text) {
  const start = source.indexOf(target);
  assert.ok(start >= 0, target);
  assert.equal(source.slice(start, start + target.length), target);
  const from = location(source, start);
  const to = location(source, start + target.length);
  return {
    ruleId: rule,
    ruleDocsPath: "docs/content/rules/vue.md",
    severity: 1,
    message: `[vize:${rule}] ${text}`,
    line: from.line,
    column: from.column,
    endLine: to.line,
    endColumn: to.column,
  };
}

export function wholeJson(entry, after = false, originalRules = false) {
  const messages = [];
  if (originalRules) {
    messages.push(
      message(
        entry.source,
        "<MyCard>",
        "vue/html-self-closing",
        "Empty component should be self-closing",
      ),
      message(
        entry.source,
        "<my-card />",
        "vue/component-name-in-template-casing",
        "Component should use PascalCase",
      ),
      message(
        entry.source,
        "v-slot:header",
        "vue/v-slot-style",
        "Expected '#header' instead of 'v-slot:header'",
      ),
    );
  }
  for (const finding of entry.diagnostics) {
    if (after && finding.fix) continue;
    messages.push(
      message(
        entry.source,
        finding.target,
        RULE,
        `Boolean attribute "${finding.name}" should not have value "${finding.value}"`,
      ),
    );
  }
  return [{ file: entry.filename, messages, errorCount: 0, warningCount: messages.length }];
}
