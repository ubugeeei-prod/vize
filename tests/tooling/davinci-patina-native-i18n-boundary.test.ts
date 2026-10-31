import assert from "node:assert/strict";
import fs from "node:fs";
import { test } from "node:test";

import { withoutI18nHostImports } from "./support/davinci-i18n-host-imports.ts";

const callers = [
  "crates/vize_patina/tests/native_syntax_img_alt.rs",
  "crates/vize_patina/tests/native_syntax_img_alt/support.rs",
  "crates/vize_patina/tests/native_canonical_tabindex/support.rs",
  "crates/vize_patina/tests/native_canonical_tabindex/refusals.rs",
  "crates/vize_patina/tests/native_canonical_img_alt/support.rs",
  "crates/vize_patina/tests/native_canonical_img_alt/refusals.rs",
  "crates/vize_patina/examples/image_alt_measurement/mod.rs",
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

test("template-only SFC reporter and whole-output oracle keep exact catalog host boundaries", () => {
  const hosts = [
    [
      "crates/vize_patina/src/native/sfc/template/context.rs",
      "use vize_carton::i18n::{Locale, t, t_fmt};",
    ],
    [
      "crates/vize_patina/tests/native_sfc_template_lint/support.rs",
      "use vize_carton::i18n::translator;",
    ],
  ];
  for (const [file, host] of hosts) {
    const source = fs.readFileSync(new URL(`../../${file}`, import.meta.url), "utf8");
    assert.ok(source.includes(host), file);
    assert.doesNotMatch(withoutI18nHostImports(source, file), /\bvize_carton::/u);
    for (const unreviewed of [
      "use vize_carton::Allocator;",
      "use vize_carton::CompactString;",
      "use vize_carton::{String, i18n::Locale};",
      "use vize_carton::i18n::{Locale, t, t_fmt, Translator};",
      "use vize_carton::i18n::{translator, String};",
      "use vize_carton::i18n::*;",
    ]) {
      assert.equal(withoutI18nHostImports(unreviewed, file), unreviewed);
    }
    assert.match(
      withoutI18nHostImports(`${host}\nuse vize_carton::Allocator;`, file),
      /use vize_carton::Allocator;/u,
    );
    for (const foreign of [
      "crates/vize_patina/src/native/sfc/template/owner.rs",
      "crates/vize_patina/src/native/sfc/template/driver.rs",
      "crates/vize_patina/tests/foreign.rs",
      "davinci/vize_l2/src/lib.rs",
    ]) {
      assert.equal(withoutI18nHostImports(host, foreign), host);
    }
  }
});
