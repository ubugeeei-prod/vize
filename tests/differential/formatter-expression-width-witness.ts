import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { sha256 } from "./manifest.mjs";
import { retainedFormatterFunction } from "./formatter-history-source-artifact.ts";

const OWNER = "crates/vize_glyph/src/template/attributes/tests.rs";
const ORIGINAL = "2d5d6bc0be19870417705dbaa6d1d55e1e532db6d2e39f65fa44e124d231dda3";
const CURRENT = "a0846e4ef168a16fdce712a4280b6808840dbeb833d5ffd5484a3a82e8b773f7";
const ASSET =
  "tests/_fixtures/differential/formatter-regressions/expression-print-width-7876/attributes-tests.cc87.original.txt";
const NAMES = [
  "render_attribute_uses_single_quotes_when_value_contains_double_quotes",
  "render_attribute_escapes_double_quotes_when_value_contains_both_quote_styles",
  "write_rendered_attribute_indents_multiline_value_lines",
  "write_rendered_attribute_leaves_literal_multiline_values_verbatim",
  "write_rendered_attribute_leaves_template_literal_lines_verbatim",
];

function ownedBytes(root: string, relative: string) {
  const real = fs.realpathSync(path.join(root, relative));
  const contained = path.relative(fs.realpathSync(root), real);
  assert(contained && !contained.startsWith("..") && !path.isAbsolute(contained));
  return fs.readFileSync(real);
}

// The two original struct literals require one new false field. Every other
// authored byte of all five original laws, including their assertions, stays exact.
export function validateExpressionWidthWitness(root: string, entry: any, name: string) {
  if (entry.path !== OWNER) return false;
  assert.equal(entry.sha256, ORIGINAL, "original attribute owner pin changed");
  assert.deepEqual(entry.revisions, [
    "8ebaea36738b696ac8423293fc6211e903d9d49b",
    "cc87bb5960ea9e49e82672205df919de58bb4b24",
  ]);
  assert(NAMES.includes(name), "unregistered attribute law");
  const original = ownedBytes(root, ASSET);
  assert.equal(sha256(original), ORIGINAL, "original whole attribute owner changed");
  const current = ownedBytes(root, OWNER);
  assert.equal(sha256(current), CURRENT, "current whole attribute owner changed");
  for (const [index, registered] of NAMES.entries()) {
    const before = retainedFormatterFunction(original, registered).toString();
    const after = retainedFormatterFunction(current, registered).toString();
    if (index < 2) {
      const field = "        owns_value_lines: false,\n";
      assert.equal(after.split(field).length, 2, "required false field changed");
      assert.equal(after.replace(field, ""), before, "original complete attribute law changed");
    } else assert.equal(after, before, "original complete attribute law changed");
  }
  return true;
}
