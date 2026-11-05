import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";

import { RULES_DIR } from "../../tools/support/compat/davinci/lib/rule-parity-paths.ts";
import {
  parseRuleFile,
  parseRuleSource,
} from "../../tools/support/compat/davinci/lib/rule-parity-rules.ts";

const model = {
  templateHooks: ["enter_element", "check_directive"],
  markupHooks: ["enter_directive"],
};
const meta = (name) =>
  `static META: RuleMeta = RuleMeta { name: ${name}, description: "Example" };`;
const parse = (source) => parseRuleSource(source, "sample.rs", model);

void test("literal and file-local const names retain every rule signal and hook", () => {
  const body = `
    use vize_croquis::Croquis as Analysis;
    impl Rule for Sample {
      fn enter_element(&self) { Analysis::new(); }
      fn as_markup_rule(&self) { Some(self) }
      fn jsx_needs_lowering(&self) { true }
    }
    impl MarkupRule for Sample { fn enter_directive(&self) {} }
    impl ScriptRule for Sample { fn uses_ast(&self) -> bool { true } }
  `;
  const literal = parse(meta('"vue/example"') + body);
  const constant = parse(
    'pub(super) const RULE: &\'static str = "vue/example";\n' + meta("RULE") + body,
  );
  assert.deepEqual(constant.meta, { kind: "RuleMeta", name: "vue/example" });
  assert.deepEqual({ ...constant, source: "" }, { ...literal, source: "" });
  assert.deepEqual([...constant.hooks], ["enter_element"]);
  assert.deepEqual([...constant.markupHookSet], ["enter_directive"]);
  assert.equal(constant.asMarkup, true);
  assert.equal(constant.jsxLowered, true);
  assert.equal(constant.scriptUsesAst, true);
  assert.deepEqual([...constant.croquisItems], [["Croquis", 1]]);
  assert.equal(parse(meta('"vue/\\u{65}xample"')).meta.name, "vue/example");
});

void test("comments, strings and nested declarations cannot supply a local const", () => {
  for (const decoy of [
    '// const RULE: &str = "vue/foreign";\n',
    '/* const RULE: &str = "vue/foreign"; */\n',
    '/* outer /* const RULE: &str = "vue/foreign"; */ nested */\n',
    'const TEXT: &str = r#"const RULE: &str = "vue/foreign";"#;\n',
    'const TEXT: &str = "const RULE: &str = \\"vue/foreign\\";";\n',
    'mod foreign { const RULE: &str = "vue/foreign"; }\n',
    'fn foreign() { const RULE: &str = "vue/foreign"; }\n',
    "use foreign::RULE;\n",
  ]) {
    assert.throws(() => parse(decoy + meta("RULE")), {
      message: "META name requires one local const RULE in sample.rs",
    });
  }
  const actual = 'const RULE: &str = "vue/actual";\n';
  const decoys =
    '/* const RULE: &str = "vue/foreign"; */\nmod foreign { const RULE: &str = "vue/foreign"; }\n';
  assert.equal(parse(decoys + actual + meta("RULE")).meta.name, "vue/actual");
  assert.equal(
    parse('const TEXT: &str = r#"static META: RuleMeta = RuleMeta { name: "foreign/rule" };"#;')
      .meta,
    null,
  );
  assert.equal(
    parse('const TEXT: &str = r#"name: "foreign/rule""#;\n' + meta('"vue/actual"')).meta.name,
    "vue/actual",
  );
  for (const field of [
    'description: r#"name: "foreign/rule""#',
    'description: "Example", /* name: "foreign/rule" */',
    'details: Other { name: "foreign/rule" }',
  ]) {
    assert.throws(() => parse(`static META: RuleMeta = RuleMeta { ${field} };`), {
      message: "META without one name in sample.rs",
    });
  }
});

void test("missing, duplicate, nonliteral and foreign references fail closed", () => {
  assert.throws(() => parse(meta("MISSING")), {
    message: "META name requires one local const MISSING in sample.rs",
  });
  assert.throws(
    () => parse('const RULE: &str = "vue/a"; const RULE: &str = "vue/b";\n' + meta("RULE")),
    {
      message: "META name requires one local const RULE in sample.rs",
    },
  );
  for (const value of [
    "ALIAS",
    'make_name("vue/example")',
    'concat!("vue/", "example")',
    '"vue/example".trim()',
    'b"vue/example"',
    '""',
  ]) {
    assert.throws(() => parse(`const RULE: &str = ${value};\n` + meta("RULE")), {
      message: "META name const RULE is not a literal &str in sample.rs",
    });
  }
  assert.throws(() => parse('const RULE: String = "vue/example";\n' + meta("RULE")), {
    message: "META name const RULE is not a literal &str in sample.rs",
  });
  for (const value of [
    "foreign::RULE",
    "META.name",
    'make_name("vue/example")',
    '"vue/example".trim()',
  ]) {
    assert.throws(() => parse(meta(value)), {
      message: "META name is not a literal or local string constant in sample.rs",
    });
  }
});

void test("all current RuleMeta names are discovered without changing source files", () => {
  const files = (directory) =>
    fs.readdirSync(directory, { withFileTypes: true }).flatMap((entry) => {
      const child = path.join(directory, entry.name);
      return entry.isDirectory() ? files(child) : entry.name.endsWith(".rs") ? [child] : [];
    });
  const rules = files(RULES_DIR)
    .map((file) => parseRuleFile(file, model))
    .filter((file) => file.meta);
  assert.equal(rules.length, 279);
  assert.equal(new Set(rules.map((rule) => rule.meta.name)).size, 279);
  const actual = rules.find((rule) => rule.rel === "script/no_ref_object_reactivity_loss.rs");
  assert.deepEqual(actual.meta, {
    kind: "ScriptRuleMeta",
    name: "vue/no-ref-object-reactivity-loss",
  });
  assert.equal(actual.scriptUsesAst, true);
});
