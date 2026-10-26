import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";

const read = (path: string) => readFileSync(new URL(`../../${path}`, import.meta.url), "utf8");
const rust = read("crates/vize_vitrine/src/napi/language_code.rs");
const declarations = read("npm/native/index.d.ts");

/** The `///` lines directly above `marker`, skipping its attributes. */
function rustDoc(marker: string): string {
  const end = rust.indexOf(marker);
  assert.ok(end > 0, marker);
  const doc: string[] = [];
  for (const line of rust.slice(0, end).trimEnd().split("\n").reverse()) {
    const text = line.trim();
    if (text.startsWith("#[")) continue;
    if (!text.startsWith("///")) break;
    doc.unshift(text.slice(3).trim());
  }
  return doc.join(" ");
}

/** The single-line JSDoc directly above `marker`. */
function declarationDoc(marker: string): string {
  const end = declarations.indexOf(marker);
  assert.ok(end > 0, marker);
  const line = declarations.slice(0, end).trimEnd().split("\n").at(-1)!.trim();
  return /^\/\*\* (.*) \*\/$/.exec(line)?.[1] ?? "";
}

void test("the editor projection binding converts panics and matches its declarations", () => {
  // tsserver hosts this binding: a Canon panic must not abort the editor.
  assert.match(rust, /#\[napi\(js_name = "generateVueLanguageCode", catch_unwind\)\]/);
  for (const [item, declaration] of [
    ["pub struct VueLanguageCodeOptions", "export interface VueLanguageCodeOptions"],
    ["pub config_file_name", "configFileName?:"],
    ["pub fn generate_vue_language_code", "export declare function generateVueLanguageCode"],
  ]) {
    const doc = rustDoc(item);
    assert.notEqual(doc, "", item);
    assert.equal(declarationDoc(declaration), doc, item);
  }
});
