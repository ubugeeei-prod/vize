import assert from "node:assert/strict";
import { readdirSync, readFileSync, statSync } from "node:fs";
import { join, relative } from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

// Lint reads each SFC script block from one shared parse (`SfcScriptParses`
// via `LintContext::sfc_scripts`) and each template expression from its
// retained `js_ast`. Every rule-local JavaScript parse below re-parses source
// the pass already holds. The ledger may only shrink: a new site fails, and a
// removed site fails until its entry shrinks with it.
const LEDGER: Readonly<Record<string, number>> = {
  "linter/corsa_session/vue_dependencies.rs": 1,
  "linter/native_type_aware/driver.rs": 1,
  "linter/native_type_aware/expression_bindings/mapped.rs": 1,
  "linter/native_type_aware/parsing.rs": 2,
  "linter/native_type_aware/plain_module.rs": 2,
  "linter/native_type_aware/plain_module/syntax.rs": 1,
  "linter/native_type_aware/script_options.rs": 1,
  "linter/native_type_aware/strict_boolean.rs": 1,
  "linter/native_type_aware/strict_boolean/template.rs": 1,
  "linter/native_type_aware/template_queries/calls.rs": 2,
  "linter/script_rules.rs": 1,
  "rules/musea.rs": 1,
  "rules/musea/require_component.rs": 1,
  "rules/musea/require_title.rs": 1,
  "rules/opinionated/vue/no_multiple_objects_in_class.rs": 1,
  "rules/opinionated/vue/require_component_registration.rs": 1,
  "rules/opinionated/vue/require_component_registration/async_setup.rs": 1,
  "rules/opinionated/vue/this_in_template/expressions.rs": 3,
  "rules/opinionated/vue/warn_custom_directive.rs": 1,
  "rules/petite_vue/valid_v_scope.rs": 1,
  "rules/script.rs": 1,
  "rules/script/linter.rs": 1,
  "rules/script/no_deprecated_destroyed_lifecycle.rs": 1,
  "rules/script/no_duplicate_attr_inheritance/template.rs": 2,
  "rules/script/no_with_defaults.rs": 1,
  "rules/script/prefer_computed/template.rs": 1,
  "rules/script/template_component_bindings.rs": 7,
  "rules/script/template_scan/calls.rs": 1,
  "rules/ssr/no_browser_globals_in_ssr/script_reads.rs": 1,
  "rules/ssr/no_browser_globals_in_ssr/template_identifiers.rs": 1,
  "rules/vue/no_deprecated_filter/assertion_types.rs": 1,
  "rules/vue/no_deprecated_filter/location.rs": 1,
  "rules/vue/no_mutating_props/handlers.rs": 1,
  "rules/vue/no_mutating_props/script_mutations.rs": 1,
  "rules/vue/no_unsafe_url.rs": 1,
  "rules/vue/no_unused_components/dynamic_is.rs": 1,
  "rules/vue/no_unused_components/script_setup_refs.rs": 1,
  "rules/vue/no_unused_properties/usage.rs": 1,
  "rules/vue/prop_name_casing/declarations.rs": 1,
  "rules/vue/require_v_for_key.rs": 1,
  "rules/vue/valid_v_for.rs": 1,
  "rules/vue/valid_v_model.rs": 1,
};

const root = fileURLToPath(new URL("../../", import.meta.url));
const source = join(root, "crates/vize_patina/src");
const isTest = (path: string) => /(^|\/)tests?(\/|\.rs$)|_tests?\.rs$/.test(path);

/** oxc `Parser` constructions plus Croquis source-text script analyses. */
export function parseSites(text: string): number {
  const qualified = text.match(/oxc_parser::Parser::new\(/g)?.length ?? 0;
  // Names an oxc `Parser` import binds: `Parser` itself or its alias.
  const names = new Set<string>();
  for (const [, list] of text.matchAll(/use oxc_parser::(\{[^}]*\}|Parser(?: as \w+)?)/g)) {
    for (const item of (list ?? "").replace(/[{}]/g, "").split(",")) {
      const binding = item.trim().match(/^Parser(?: as (\w+))?$/);
      if (binding) names.add(binding[1] ?? "Parser");
    }
  }
  let bound = 0;
  for (const name of names) {
    bound += text.match(new RegExp(`(?<![:\\w])${name}::new\\(`, "g"))?.length ?? 0;
  }
  const croquis = text.includes("vize_croquis::script_parser")
    ? (text.match(/(?<!fn )(?<![\w])(?:script_parser::)?parse_script\w*\(/g)?.length ?? 0)
    : 0;
  return qualified + bound + croquis;
}

function scan(dir: string, sites: Record<string, number>): void {
  for (const name of readdirSync(dir)) {
    const path = join(dir, name);
    const file = relative(source, path);
    if (statSync(path).isDirectory()) scan(path, sites);
    else if (file.endsWith(".rs") && !isTest(file)) {
      const count = parseSites(readFileSync(path, "utf8"));
      if (count > 0) sites[file] = count;
    }
  }
}

void test("rule-local JavaScript parses in vize_patina only shrink", () => {
  const sites: Record<string, number> = {};
  scan(source, sites);
  for (const [file, count] of Object.entries(sites)) {
    assert.ok(
      count <= (LEDGER[file] ?? 0),
      `${file} parses source the lint pass already holds (${count} > ${LEDGER[file] ?? 0}); read LintContext::sfc_scripts or the retained js_ast`,
    );
  }
  for (const [file, count] of Object.entries(LEDGER)) {
    assert.equal(sites[file] ?? 0, count, `${file} dropped parse sites; shrink its ledger entry`);
  }
});

void test("the site counter sees qualified, imported and Croquis parses only", () => {
  assert.equal(parseSites("let p = oxc_parser::Parser::new(a, s, t).parse();"), 1);
  assert.equal(parseSites("use oxc_parser::{Parser, ParseOptions};\nParser::new(a, s, t);"), 1);
  assert.equal(parseSites("use vize_armature::Parser;\nParser::new(a, s);"), 0);
  assert.equal(
    parseSites("use oxc_parser::Parser as Oxc;\nuse vize_armature::Parser;\nParser::new(a, s);"),
    0,
  );
  assert.equal(parseSites("use oxc_parser::Parser as Oxc;\nOxc::new(a, s, t);"), 1);
  assert.equal(parseSites("use oxc_parser::{ParseOptions, Parser as Js};\nJs::new(a, s, t);"), 1);
  assert.equal(
    parseSites(
      "use vize_croquis::script_parser::parse_script_setup;\nlet r = parse_script_setup(s);",
    ),
    1,
  );
  assert.equal(parseSites("fn parse_script_block(s: &str) {}"), 0);
});
