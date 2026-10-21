import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";

import {
  accountRuleFiles,
  buildMatrix,
} from "../../tools/support/compat/davinci/lib/rule-parity-matrix.ts";
import { RULES_DIR } from "../../tools/support/compat/davinci/lib/rule-parity-paths.ts";
import { collectPrivateScriptCallbacks } from "../../tools/support/compat/davinci/lib/rule-parity-private-script.ts";
import { parseRuleSource } from "../../tools/support/compat/davinci/lib/rule-parity-rules.ts";

const model = {
  templateHooks: ["as_sfc_script_rule", "run_on_template", "run_on_sfc"],
  markupHooks: [],
};
const owner = `
mod adapter;
static META: RuleMeta = RuleMeta { name: "sample/owned-callback" };
pub struct Owner { callback: adapter::Delegate }
impl Rule for Owner {
  fn meta(&self) -> &'static RuleMeta { &META }
  fn as_sfc_script_rule(&self) -> Option<&dyn ScriptRule> { Some(&self.callback) }
}
`;
const callback = `
use super::META;
static LOCAL_META: ScriptRuleMeta = ScriptRuleMeta {
  name: META.name,
  description: META.description,
  default_severity: META.default_severity,
};
pub(super) struct Delegate;
impl ScriptRule for Delegate {
  fn meta(&self) -> &'static ScriptRuleMeta { &LOCAL_META }
  fn uses_ast(&self) -> bool { true }
}
`;
const sources = (parent = owner, child = callback) => [
  parseRuleSource(parent, "sample/owner.rs", model),
  parseRuleSource(child, "sample/owner/adapter.rs", model),
];
const associate = (files, registered = new Set(["Owner"])) =>
  collectPrivateScriptCallbacks(files, registered);

void test("direct private callbacks retain one generic owner identity and no template hook", () => {
  for (const files of [
    sources(),
    sources(
      owner,
      callback
        .replace("use super::META;", "use super::{META as PARENT, Other};")
        .replaceAll("META.", "PARENT."),
    ),
    sources(owner, callback).map((file) => ({
      ...file,
      rel: file.rel.replaceAll("sample/owner", "other/renamed"),
    })),
    sources(owner.replace("mod adapter;", "/* mod foreign; */\nmod adapter;"), callback),
    sources(owner.replace("mod adapter;", "#[allow(dead_code)]\nmod adapter;"), callback),
    sources().map((file, i) => (i ? file : { ...file, rel: "sample/owner/mod.rs" })),
  ]) {
    const { byCallback, byParent } = associate(files);
    assert.equal(byCallback.size, 1);
    assert.equal(byParent.size, 1);
    const binding = [...byParent.values()][0];
    assert.equal(binding.parent.meta.name, "sample/owned-callback");
    assert.equal(binding.callback.meta, null);
    assert.equal(binding.type, "Delegate");
    assert.equal(binding.field, "callback");
    assert.equal(binding.callback.scriptUsesAst, true);
    assert.deepEqual([...binding.parent.hooks], []);
    assert.deepEqual(binding.parent.ruleImplTypes, ["Owner"]);
  }
});

void test("foreign, missing, ambiguous and standalone callback evidence fails closed", async (t) => {
  const variants = [
    ["missing owner", () => sources().slice(1)],
    [
      "foreign module owner",
      () => sources().map((file, i) => (i ? { ...file, rel: "foreign/adapter.rs" } : file)),
    ],
    [
      "ambiguous module owner",
      () => [...sources(), { ...sources()[0], rel: "sample/owner/mod.rs" }],
    ],
    ["missing module declaration", () => sources(owner.replace("mod adapter;", ""))],
    ["public module", () => sources(owner.replace("mod adapter;", "pub mod adapter;"))],
    [
      "conditional module",
      () => sources(owner.replace("mod adapter;", "#[cfg(test)]\nmod adapter;")),
    ],
    [
      "foreign path attribute",
      () => sources(owner.replace("mod adapter;", '#[path = "foreign.rs"]\nmod adapter;')),
    ],
    [
      "stacked conditional attributes",
      () =>
        sources(owner.replace("mod adapter;", "#[cfg(test)]\n#[allow(dead_code)]\nmod adapter;")),
    ],
    [
      "conditional owner META",
      () => sources(owner.replace("static META", "#[cfg(test)]\nstatic META")),
    ],
    [
      "conditional owner type",
      () => sources(owner.replace("pub struct Owner", "#[cfg(test)]\npub struct Owner")),
    ],
    [
      "conditional owner trait",
      () => sources(owner.replace("impl Rule", "#[cfg(test)]\nimpl Rule")),
    ],
    [
      "conditional callback type",
      () =>
        sources(owner, callback.replace("pub(super) struct", "#[cfg(test)]\npub(super) struct")),
    ],
    [
      "conditional callback trait",
      () => sources(owner, callback.replace("impl ScriptRule", "#[cfg(test)]\nimpl ScriptRule")),
    ],
    [
      "conditional metadata import",
      () => sources(owner, callback.replace("use super", "#[cfg(test)]\nuse super")),
    ],
    [
      "conditional metadata projection",
      () =>
        sources(owner, callback.replace("static LOCAL_META", "#[cfg(test)]\nstatic LOCAL_META")),
    ],
    [
      "split metadata identifiers",
      () => sources(owner, callback.replace("name: META.name", "na me: META.name")),
    ],
    ["missing owner metadata", () => sources(owner.replace(/static META[^;]*;/, ""))],
    [
      "ambiguous owner metadata",
      () => sources(owner + 'static META: RuleMeta = RuleMeta { name: "other" };'),
    ],
    ["foreign owner metadata return", () => sources(owner.replace("{ &META }", "{ &OTHER }"))],
    [
      "foreign callback field type",
      () => sources(owner.replace("adapter::Delegate", "foreign::Delegate")),
    ],
    [
      "ambiguous typed fields",
      () =>
        sources(
          owner.replace(
            "callback: adapter::Delegate",
            "callback: adapter::Delegate, other: adapter::Delegate",
          ),
        ),
    ],
    [
      "missing returned field",
      () => sources(owner.replace("Some(&self.callback)", "Some(&self.other)")),
    ],
    [
      "conditional callback return",
      () =>
        sources(
          owner.replace("Some(&self.callback)", "if true { Some(&self.callback) } else { None }"),
        ),
    ],
    ["missing parent import", () => sources(owner, callback.replace("use super::META;", ""))],
    [
      "foreign parent import",
      () => sources(owner, callback.replace("super::META", "foreign::META")),
    ],
    ["ambiguous parent import", () => sources(owner, callback + "use foreign::META;")],
    ["shadowed parent import", () => sources(owner, callback + "const META: Other = Other;")],
    [
      "changed metadata projection",
      () => sources(owner, callback.replace("META.description", "OTHER.description")),
    ],
    [
      "missing metadata projection",
      () => sources(owner, callback.replace("default_severity: META.default_severity,", "")),
    ],
    [
      "duplicate metadata projection",
      () =>
        sources(owner, callback.replace("name: META.name,", "name: META.name, name: META.name,")),
    ],
    [
      "different callback metadata return",
      () => sources(owner, callback.replace("&LOCAL_META", "&OTHER_META")),
    ],
    ["public callback type", () => sources(owner, callback.replace("pub(super)", "pub"))],
    [
      "ambiguous callback traits",
      () => sources(owner, callback + "impl Rule for Delegate { fn meta(&self) { &META } }"),
    ],
    [
      "independent callback META",
      () =>
        sources(
          owner,
          callback
            .replace("LOCAL_META", "META")
            .replace("name: META.name", 'name: "sample/standalone"'),
        ),
    ],
  ];
  for (const [name, files] of variants) {
    await t.test(name, () => assert.throws(() => associate(files()), /callback|META|owner/i));
  }
  await t.test("registered callback without META", () => {
    assert.throws(
      () => associate(sources(), new Set(["Owner", "Delegate"])),
      /independently registered/,
    );
  });
  await t.test("unregistered owner", () => {
    assert.throws(() => associate(sources(), new Set()), /owner is not registered/);
  });
  for (const trait of ["Rule", "ScriptRule", "CssRule", "MuseaRule"]) {
    await t.test(`unknown ${trait} without META`, () => {
      const unknown = parseRuleSource(`impl ${trait} for Unknown {}`, "unknown.rs", model);
      assert.throws(() => accountRuleFiles([unknown], new Set(["Unknown"])), /without META/);
    });
  }
  await t.test("the existing metadata-free MarkupRule helper remains permitted", () => {
    const markup = parseRuleSource("impl MarkupRule for Facade {}", "facade.rs", model);
    const result = accountRuleFiles([markup], new Set());
    assert.equal(result.ruleFiles.length, 0);
    assert.equal(result.nonRuleFiles[0].kind, "helper");
  });
});

void test("the actual repository matrix adds the parent's SFC script surface without a second rule or JSX change", () => {
  const matrix = buildMatrix();
  assert.equal(matrix.privateScripts.byCallback.size, 1);
  const binding = [...matrix.privateScripts.byParent.values()][0];
  assert.equal(binding.parent.meta.name, "vue/no-mutating-props");
  assert.equal(binding.callback.meta, null);
  assert.equal(binding.type, "OptionsApiScript");
  assert.equal(binding.field, "options");
  assert.equal(matrix.rules.size, matrix.ruleFiles.length);
  assert.equal(
    matrix.ruleFiles.filter((file) => file.meta.name === binding.parent.meta.name).length,
    1,
  );
  const rule = matrix.rules.get(binding.parent.meta.name);
  assert.equal(rule.registered, true);
  assert.equal(rule.family, "template-family");
  assert.equal(rule.sfc, "yes");
  assert.equal(rule.surfaces.filter((surface) => surface === "script-oxc").length, 1);
  assert.match(rule.sfcDetail, /script-blocks/);
  assert.equal(binding.parent.hooks.has("as_sfc_script_rule"), false);
  const raw = fs.readFileSync(path.join(RULES_DIR, binding.parent.rel), "utf8");
  const priorRaw = raw.replace(/\n    fn as_sfc_script_rule[^]*?\n    }\n/, "\n");
  assert.notEqual(priorRaw, raw);
  const prior = parseRuleSource(priorRaw, binding.parent.rel, matrix.model);
  assert.deepEqual([...binding.parent.hooks], [...prior.hooks]);
  assert.equal(binding.parent.asMarkup, prior.asMarkup);
  assert.equal(binding.parent.jsxLowered, prior.jsxLowered);
  assert.equal(rule.jsx, "yes");
  assert.equal(rule.jsxLane, "fallback");
});
