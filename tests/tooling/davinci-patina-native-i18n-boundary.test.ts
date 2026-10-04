import assert from "node:assert/strict";
import fs from "node:fs";
import { test } from "node:test";

import { withoutI18nHostImports } from "./support/davinci-i18n-host-imports.ts";

const callers = [
  "crates/vize_patina/tests/native_syntax_img_alt.rs",
  "crates/vize_patina/tests/native_syntax_img_alt/support.rs",
];
const declaration = "use vize_carton::i18n::{Locale, translator};";

test("native img-alt differential tests use the actual reviewed host catalog", () => {
  for (const file of callers) {
    const source = fs.readFileSync(new URL(`../../${file}`, import.meta.url), "utf8");
    assert.ok(source.includes(declaration), file);
    assert.doesNotMatch(withoutI18nHostImports(source, file), /\bvize_carton::/u);
  }
});

test("native catalog registration does not admit storage or changed imports", () => {
  for (const file of callers) {
    for (const unreviewed of [
      "use vize_carton::CompactString;",
      "use vize_carton::{Allocator, Span};",
      "use vize_carton::i18n::{Locale, translator, Translator};",
      "use vize_carton::i18n::Translator;",
    ]) {
      assert.equal(withoutI18nHostImports(unreviewed, file), unreviewed);
    }
    const mixed = `${declaration}\nuse vize_carton::Allocator;`;
    assert.match(withoutI18nHostImports(mixed, file), /use vize_carton::Allocator;/u);
  }
});

test("catalog test registration cannot authorize native production or foreign paths", () => {
  for (const file of [
    "crates/vize_patina/src/native.rs",
    "crates/vize_patina/src/native/img_alt.rs",
    "crates/vize_patina/tests/foreign.rs",
    "davinci/vize_l1/src/lib.rs",
  ]) {
    assert.equal(withoutI18nHostImports(declaration, file), declaration);
  }
});

test("the configured SFC reporter admits only its exact original catalog host import", () => {
  const file = "crates/vize_patina/src/native/sfc/context.rs";
  const host = "use vize_carton::i18n::{Locale, t, t_fmt};";
  const source = fs.readFileSync(new URL(`../../${file}`, import.meta.url), "utf8");
  assert.ok(source.includes(host), file);
  assert.doesNotMatch(withoutI18nHostImports(source, file), /\bvize_carton::/u);
  for (const unreviewed of [
    "use vize_carton::Allocator;",
    "use vize_carton::CompactString;",
    "use vize_carton::i18n::{Locale, t, t_fmt, Translator};",
  ]) {
    assert.equal(withoutI18nHostImports(unreviewed, file), unreviewed);
  }
  for (const foreign of [
    "crates/vize_patina/src/native/sfc/owner.rs",
    "crates/vize_patina/src/native/sfc/driver.rs",
    "crates/vize_patina/src/native/sfc/tests/support.rs",
    "davinci/vize_l2/src/lib.rs",
  ]) {
    assert.equal(withoutI18nHostImports(host, foreign), host);
  }
});
