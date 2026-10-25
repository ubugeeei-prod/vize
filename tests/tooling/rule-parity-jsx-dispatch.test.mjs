import assert from "node:assert/strict";
import fs from "node:fs";
import test from "node:test";
import { validateJsxDispatch } from "../../tools/support/compat/davinci/lib/rule-parity-dispatch.ts";
import { matchBraceBlock } from "../../tools/support/compat/davinci/lib/rule-parity-rust-text.ts";
import { stripRust } from "../../tools/support/compat/davinci/lib/rust-source.ts";

const engine = fs.readFileSync(
  new URL("../../crates/vize_patina/src/linter/engine.rs", import.meta.url),
  "utf8",
);

function bodyRange(source, name) {
  const stripped = stripRust(source);
  const found = new RegExp(`\\bfn\\s+${name}\\b`, "u").exec(stripped);
  assert.ok(found);
  const open = stripped.indexOf("{", found.index);
  return [open + 1, matchBraceBlock(stripped, open)];
}

function changeBody(source, name, change) {
  const [start, end] = bodyRange(source, name);
  const before = source.slice(start, end);
  const after = change(before);
  assert.notEqual(after, before, `negative control must modify ${name}`);
  return source.slice(0, start) + after + source.slice(end);
}

void test("actual JSX public wrapper and owned markup phase retain the dispatch graph", () => {
  assert.doesNotThrow(() => validateJsxDispatch(engine));
  const [start, end] = bodyRange(engine, "lint_jsx_without_types");
  const inline = changeBody(engine, "lint_jsx", () => engine.slice(start, end));
  assert.doesNotThrow(() => validateJsxDispatch(inline));
});

void test("wrapper delegation and type completion must occur exactly once in source order", () => {
  for (const change of [
    (body) =>
      body.replace(
        "self.lint_jsx_without_types(source, filename, lang)",
        "self.unknown(source, filename, lang)",
      ),
    (body) => `${body}\n self.lint_jsx_without_types(source, filename, lang);`,
    (body) => body.replace("append_plain_module_type_diagnostics", "missing_type_completion"),
    (body) =>
      body.replace(
        /self,\s*source,\s*filename,\s*&mut result/u,
        "self, other_source, filename, &mut result",
      ),
    (body) =>
      body.replace(
        "self.apply_diagnostic_policy(result)",
        "self.apply_diagnostic_policy(other_result)",
      ),
    (body) =>
      `${body}\n super::native_type_aware::plain_module::append_plain_module_type_diagnostics(self, source, filename, &mut result);`,
    (body) => {
      const delegate = "let mut result = self.lint_jsx_without_types(source, filename, lang);";
      return `${body.replace(delegate, "")}\n${delegate}`;
    },
  ]) {
    const modified = changeBody(engine, "lint_jsx", change);
    assert.throws(() => validateJsxDispatch(modified), /dispatch model drift/u);
  }
});

void test("every real markup partition remains required after the helper extraction", () => {
  for (const anchor of [
    "as_markup_rule",
    "jsx_needs_lowering",
    "legacy_keep_mask",
    "lint_jsx_over_ir",
    "lint_jsx_lowered_markup_root",
    "lint_jsx_fallback_root",
  ]) {
    const modified = changeBody(engine, "lint_jsx_without_types", (body) =>
      body.replaceAll(anchor, "missing_partition"),
    );
    assert.throws(() => validateJsxDispatch(modified), /JSX markup phase lost anchor/u);
  }
  const duplicateScript = changeBody(
    engine,
    "lint_jsx_without_types",
    (body) => `${body}\n self.lint_jsx_script(source, filename, &mut result);`,
  );
  assert.throws(() => validateJsxDispatch(duplicateScript), /script rules exactly once/u);
});

void test("SFC CSS and direct checker dispatch remain forbidden in the markup phase", () => {
  for (const forbidden of [
    "run_on_sfc",
    "css_rules",
    "native_type_aware",
    "CorsaTypeAwareSession",
    "with_corsa_session",
  ]) {
    const markup = changeBody(
      engine,
      "lint_jsx_without_types",
      (body) => `${body}\n ${forbidden}();`,
    );
    assert.throws(() => validateJsxDispatch(markup), /JSX markup phase now references/u);
    const wrapper = changeBody(engine, "lint_jsx", (body) => `${body}\n ${forbidden}();`);
    assert.throws(() => validateJsxDispatch(wrapper), /JSX wrapper gained unsupported/u);
  }
});

void test("comment and literal text cannot counterfeit dispatch edges or forbidden calls", () => {
  const comments = changeBody(
    engine,
    "lint_jsx_without_types",
    (body) =>
      `${body}\n // native_type_aware::run_on_sfc css_rules\n let text = r#"CorsaTypeAwareSession with_corsa_session"#;`,
  );
  assert.doesNotThrow(() => validateJsxDispatch(comments));
  const counterfeit = changeBody(engine, "lint_jsx", (body) =>
    body.replace(
      "self.lint_jsx_without_types(source, filename, lang)",
      '"self.lint_jsx_without_types(source, filename, lang)"',
    ),
  );
  assert.throws(() => validateJsxDispatch(counterfeit), /dispatch model drift/u);
  assert.throws(
    () => validateJsxDispatch(`${engine}\n fn lint_jsx_without_types() {}`),
    /expected one lint_jsx_without_types/u,
  );
});
