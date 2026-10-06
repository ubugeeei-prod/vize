import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";

export const sha256 = (bytes) => createHash("sha256").update(bytes).digest("hex");
export function loadCorpus(root) {
  const directory = path.join(root, "tests/_fixtures/lint-default-correctness");
  const corpus = JSON.parse(fs.readFileSync(path.join(directory, "cases.json"), "utf8"));
  assert.equal(corpus.schema, "vize.lint.default-correctness");
  assert.equal(corpus.version, 1);
  assert.equal(corpus.cases.length, 15);
  for (const entry of corpus.cases) {
    assert.equal(
      fs.readFileSync(path.join(directory, entry.filename + ".fixture"), "utf8"),
      entry.source,
    );
    for (const finding of entry.diagnostics) {
      assert.ok(finding.start < finding.end && finding.end <= Buffer.byteLength(entry.source));
      assert.equal(finding.severity, finding.rule_name === "vue/v-on-style" ? "warning" : "error");
      assert.equal(
        finding.help,
        finding.rule_name === "script/no-import-compiler-macros"
          ? "Remove the macro from the import statement. Compiler macros are auto-imported."
          : finding.rule_name === "vue/v-on-style"
            ? 'Use `@event="handler"` instead of `v-on:event="handler"`'
            : null,
      );
      assert.deepEqual(finding.labels, []);
      if (finding.rule_name === "vue/v-on-style") {
        assert.equal(entry.id, "full-modifiers");
        assert.deepEqual(finding.fix, {
          message: "Use shorthand syntax",
          edits: [{ start: 50, end: 88, new_text: '@activate.native.stop="onActivate"' }],
        });
      } else {
        assert.equal(finding.fix, null);
      }
    }
  }
  return corpus;
}

function location(source, offset) {
  const before = Buffer.from(source).subarray(0, offset).toString("utf8");
  const lines = before.split("\n");
  return [lines.length, Array.from(lines.at(-1)).length + 1];
}

export function wholeJson(entry, findings = entry.diagnostics) {
  return [
    {
      file: entry.filename,
      messages: findings.map((finding) => {
        const [line, column] = location(entry.source, finding.start);
        const [endLine, endColumn] = location(entry.source, finding.end);
        return {
          ruleId: finding.rule_name,
          ruleDocsPath: finding.rule_name.startsWith("script/")
            ? "docs/content/rules/type-and-script.md"
            : "docs/content/rules/vue.md",
          severity: finding.severity === "error" ? 2 : 1,
          message: `[vize:${finding.rule_name}] ${finding.message}`,
          line,
          column,
          endLine,
          endColumn,
          ...(finding.help === null ? {} : { help: finding.help.replaceAll("`", "") }),
        };
      }),
      errorCount: findings.filter((finding) => finding.severity === "error").length,
      warningCount: findings.filter((finding) => finding.severity === "warning").length,
    },
  ];
}
