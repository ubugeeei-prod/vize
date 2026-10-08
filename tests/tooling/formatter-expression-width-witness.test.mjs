import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { validateExpressionWidthWitness } from "../differential/formatter-expression-width-witness.ts";
import { validateCurrentFormatterWitness } from "../differential/formatter-history-current-witness.ts";

const root = fileURLToPath(new URL("../..", import.meta.url));
const owner = "crates/vize_glyph/src/template/attributes/tests.rs";
const asset =
  "tests/_fixtures/differential/formatter-regressions/expression-print-width-7876/attributes-tests.cc87.original.txt";
const entry = {
  path: owner,
  sha256: "2d5d6bc0be19870417705dbaa6d1d55e1e532db6d2e39f65fa44e124d231dda3",
  revisions: [
    "8ebaea36738b696ac8423293fc6211e903d9d49b",
    "cc87bb5960ea9e49e82672205df919de58bb4b24",
  ],
};
const names = [
  "render_attribute_uses_single_quotes_when_value_contains_double_quotes",
  "render_attribute_escapes_double_quotes_when_value_contains_both_quote_styles",
  "write_rendered_attribute_indents_multiline_value_lines",
  "write_rendered_attribute_leaves_literal_multiline_values_verbatim",
  "write_rendered_attribute_leaves_template_literal_lines_verbatim",
];

void test("all five original attribute laws retain every byte except the two required false fields", () => {
  for (const name of names) assert.equal(validateExpressionWidthWitness(root, entry, name), true);
  assert.equal(
    validateExpressionWidthWitness(root, { ...entry, path: "other.rs" }, names[0]),
    false,
  );
  for (const changed of [
    { ...entry, sha256: "0".repeat(64) },
    { ...entry, revisions: [] },
  ])
    assert.throws(() => validateExpressionWidthWitness(root, changed, names[0]));
  assert.throws(() => validateExpressionWidthWitness(root, entry, "unregistered_law"));
});

void test("changed original assets, current law bytes and outside symlinks cannot inherit credit", (t) => {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-attribute-law-"));
  t.after(() => fs.rmSync(directory, { recursive: true, force: true }));
  for (const relative of [owner, asset]) {
    const file = path.join(directory, relative);
    fs.mkdirSync(path.dirname(file), { recursive: true });
    fs.copyFileSync(path.join(root, relative), file);
  }
  assert.equal(validateExpressionWidthWitness(directory, entry, names[0]), true);
  for (const relative of [owner, asset]) {
    const file = path.join(directory, relative);
    const original = fs.readFileSync(file);
    fs.appendFileSync(file, "\n");
    assert.throws(() => validateExpressionWidthWitness(directory, entry, names[0]));
    fs.writeFileSync(file, original);
    fs.unlinkSync(file);
    fs.symlinkSync(path.join(root, relative), file);
    assert.throws(() => validateExpressionWidthWitness(directory, entry, names[0]));
    fs.unlinkSync(file);
    fs.writeFileSync(file, original);
  }
  const file = path.join(directory, owner);
  fs.writeFileSync(
    file,
    fs.readFileSync(file, "utf8").replace("owns_value_lines: false", "owns_value_lines: true"),
  );
  assert.throws(() => validateExpressionWidthWitness(directory, entry, names[0]));
});

void test("expression extraction retains all ten complete original script witnesses", () => {
  const script = {
    path: "crates/vize_glyph/src/script.rs",
    sha256: "a205174795bb6d993e84b0cc29dfc5d10c36602dfb6216f1db2278a982b2e0ab",
    revisions: ["cc87bb5960ea9e49e82672205df919de58bb4b24"],
  };
  for (const name of [
    "test_format_tsx_component_script",
    "test_format_jsx_component_script",
    "test_format_js_expression_simple",
    "test_format_js_expression_with_optional_chaining",
    "test_format_js_expression_empty",
    "test_format_simple_script",
    "test_format_with_imports",
    "test_format_object",
    "test_format_empty_source",
    "test_format_whitespace_only",
  ])
    validateCurrentFormatterWitness(root, script, name);
});
