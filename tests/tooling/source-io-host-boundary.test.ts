import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";
import { withoutHostRuntimeReferences } from "./support/davinci-host-imports.ts";
import {
  sourceIoAliasReads,
  sourceIoDirectReads,
} from "./support/davinci-source-io-host-imports.ts";

const read = (file: string) => readFileSync(new URL("../../" + file, import.meta.url), "utf8");
const forbidden = /\bvize_carton::|use vize_carton\b/u;
const hosts = [...Object.keys(sourceIoDirectReads), ...sourceIoAliasReads];

test("only actual filesystem callers receive the exact host ownership exception", () => {
  for (const file of hosts) {
    const source = read(file);
    const directCount = [...source.matchAll(/vize_carton::source_io::read_to_string\(/gu)].length;
    assert.equal(directCount, sourceIoDirectReads[file] ?? 0, file);
    for (const path of [file, file.replaceAll("/", "\\")]) {
      assert.doesNotMatch(withoutHostRuntimeReferences(source, path), forbidden, file);
    }
    assert.match(
      withoutHostRuntimeReferences(source, "davinci/vize_l0/src/source_io.rs"),
      forbidden,
      file,
    );
    assert.match(
      withoutHostRuntimeReferences(source, "crates/vize/src/commands/unreviewed.rs"),
      forbidden,
      file,
    );
  }
});

test("extra reads, storage imports and alternate source APIs remain forbidden", () => {
  for (const file of hosts) {
    const source = read(file);
    for (const extra of [
      "vize_carton::source_io::read_to_string(path);",
      "use vize_carton::String;",
      "use vize_carton::{String, source_io};",
      "use vize_carton::source_io::*;",
      "use vize_carton::source_io::{read_to_string, decode_utf8};",
      "use vize_carton::source_io as files;",
      "vize_carton::source_io::decode_utf8(bytes);",
      "vize_carton::source_io::read_to_string_extra(path);",
    ]) {
      assert.match(withoutHostRuntimeReferences(source + "\n" + extra, file), forbidden, extra);
    }
  }
  for (const file of sourceIoAliasReads) {
    const source = read(file);
    for (const extra of [
      "use vize_carton::source_io as fs;",
      "fs::read_to_string(path);",
      "fs::decode_utf8(bytes);",
      "fs :: unreviewed_reader(path);",
      "use fs::*;",
      "use fs::{read_to_string};",
    ]) {
      assert.match(withoutHostRuntimeReferences(source + "\n" + extra, file), forbidden, extra);
    }
  }
});

test("L0 retains the borrowed decoder and all filesystem declarations belong to Carton", () => {
  const level = read("davinci/vize_l0/src/source_io.rs");
  const host = read("crates/vize_carton/src/source_io.rs");
  assert.doesNotMatch(level, /\b(?:FileString|read_to_string|vize_carton)\b|std::fs/u);
  assert.match(level, /pub fn decode_utf8\(bytes: &\[u8\]\) -> Result<&str, Utf8Error>/u);
  assert.match(host, /^pub use vize_l0::source_io::decode_utf8;$/mu);
  assert.match(host, /#\[cfg\(not\(target_arch = "wasm32"\)\)\]\npub fn read_to_string/u);
  assert.doesNotMatch(host, /pub fn decode_utf8/u);
  assert.doesNotMatch(read("davinci/vize_l0/src/source_io/tests.rs"), /std::fs|read_to_string/u);
  assert.match(
    read("crates/vize_carton/src/source_io/tests.rs"),
    /fn file_reader_preserves_contents_and_standard_error_contracts/u,
  );
});
