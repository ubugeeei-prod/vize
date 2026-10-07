// Run --move-only and commit the two byte-exact renames before --integrate.
// On a conflict, rerun the modes on fresh main instead of hand-merging paths.
import assert from "node:assert/strict";
import { existsSync, mkdirSync, readFileSync, renameSync, writeFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../../../..");
const level = "davinci/vize_l0/src/source_io";
const host = "crates/vize_carton/src/source_io";
const planned = new Map<string, string>();
const read = (file: string): string =>
  planned.get(file) ?? readFileSync(path.join(root, file), "utf8");
const write = (file: string, text: string): void => {
  planned.set(file, text);
};

function replace(file: string, before: string, after: string, count = 1): void {
  const source = read(file);
  const found = source.split(before).length - 1;
  const integrated = source.split(after).length - 1;
  if (found === 0 && integrated === count) return;
  assert.equal(found, count, `${file}: unexpected source boundary`);
  assert.equal(integrated, 0, `${file}: mixed source ownership`);
  write(file, source.replaceAll(before, after));
}

function moveOnly(): void {
  if (
    existsSync(path.join(root, `${host}.rs`)) &&
    read(`${host}.rs`).includes("pub use vize_l0::source_io::decode_utf8;")
  ) {
    for (const file of [`${level}.rs`, `${level}/tests.rs`, `${host}.rs`, `${host}/tests.rs`]) {
      assert.ok(existsSync(path.join(root, file)), `missing integrated source owner: ${file}`);
    }
    return;
  }
  const moves: [string, string][] = [];
  for (const suffix of [".rs", "/tests.rs"]) {
    const from = path.join(root, level + suffix);
    const to = path.join(root, host + suffix);
    if (existsSync(to)) {
      assert.ok(!existsSync(from), `both source owners exist before integration: ${suffix}`);
      continue;
    }
    assert.ok(existsSync(from), `missing source: ${suffix}`);
    moves.push([from, to]);
  }
  for (const [from, to] of moves) {
    mkdirSync(path.dirname(to), { recursive: true });
    renameSync(from, to);
  }
}

function integrate(): void {
  const source = read(`${host}.rs`);
  if (source.includes("pub use vize_l0::source_io::decode_utf8;")) {
    assert.ok(existsSync(path.join(root, `${level}.rs`)), "missing restored L0 decoder");
    assert.ok(existsSync(path.join(root, `${level}/tests.rs`)), "missing restored L0 decoder laws");
    const decoder = read(`${level}.rs`);
    const laws = read(`${level}/tests.rs`);
    assert.ok(decoder.includes("pub fn decode_utf8(bytes: &[u8]) -> Result<&str, Utf8Error>"));
    assert.ok(
      !/read_to_string|FileString|std::fs|vize_carton/u.test(decoder),
      "mixed L0 source ownership",
    );
    assert.ok(laws.includes("use super::decode_utf8;") && !/read_to_string|std::fs/u.test(laws));
    for (const law of [
      "all_one_and_two_byte_inputs_match_the_standard_decoder",
      "unicode_truncation_overlong_surrogates_and_block_boundaries_match_std",
      "deterministic_random_external_buffers_match_std",
    ])
      assert.ok(laws.includes(`fn ${law}()`), `missing restored decoder law: ${law}`);
  } else {
    const decodeStart = source.indexOf("/// Validate source bytes,");
    const hostStart = source.indexOf("/// Read a UTF-8 file");
    const testsStart = source.indexOf('#[cfg(all(test, not(target_arch = "wasm32")))]');
    assert.ok(decodeStart > 0 && hostStart > decodeStart && testsStart > hostStart);
    const decode = source.slice(decodeStart, hostStart);
    const readFile = source.slice(hostStart, testsStart);
    write(
      `${level}.rs`,
      "//! Borrowed UTF-8 validation for authored source bytes.\n//! Validation never normalizes or repairs bytes, preserving exact offsets.\n\n" +
        "use core::str::Utf8Error;\n\n" +
        decode +
        source.slice(testsStart),
    );
    write(
      `${host}.rs`,
      "//! Owned source-file buffers at the host filesystem boundary.\n//! Authored bytes and standard filesystem errors are preserved.\n\n" +
        '#[cfg(not(target_arch = "wasm32"))]\nuse std::{io, path::Path};\n\n' +
        "#[expect(\n    clippy::disallowed_types,\n" +
        '    reason = "Preserve the std file-reader\'s owned buffer without a copy"\n)]\n' +
        '#[cfg(not(target_arch = "wasm32"))]\nuse std::string::String as FileString;\n\n' +
        "pub use vize_l0::source_io::decode_utf8;\n\n" +
        readFile +
        '#[cfg(all(test, not(target_arch = "wasm32")))]\n' +
        "#[expect(\n    clippy::disallowed_methods,\n" +
        '    reason = "The filesystem law compares the existing std diagnostic spelling"\n)]\n' +
        "mod tests;\n",
    );
    const laws = read(`${host}/tests.rs`);
    const fileLawStart = laws.indexOf(
      "#[test]\nfn file_reader_preserves_contents_and_standard_error_contracts()",
    );
    assert.ok(fileLawStart > 0, "missing original filesystem law");
    write(
      `${level}/tests.rs`,
      laws
        .slice(0, fileLawStart)
        .replace("use super::{decode_utf8, read_to_string};", "use super::decode_utf8;")
        .trimEnd() + "\n",
    );
    write(`${host}/tests.rs`, "use super::read_to_string;\n\n" + laws.slice(fileLawStart));
  }
  replace(
    "crates/vize_carton/src/lib.rs",
    'pub mod profile_export;\n\n#[cfg(not(target_arch = "wasm32"))]',
    'pub mod profile_export;\n\npub mod source_io;\n\n#[cfg(not(target_arch = "wasm32"))]',
  );
  for (const [file, count] of [
    ["crates/vize/src/commands/build/runner/compile.rs", 3],
    ["crates/vize/src/commands/build/runner/compile_stats.rs", 1],
    ["crates/vize_vitrine/src/napi/sfc/batch.rs", 1],
  ] as const) {
    replace(
      file,
      "vize_l0::source_io::read_to_string",
      "vize_carton::source_io::read_to_string",
      count,
    );
  }
  for (const file of ["crates/vize/src/commands/fmt.rs", "crates/vize/src/commands/lint.rs"]) {
    replace(file, "use vize_l0::source_io as fs;", "use vize_carton::source_io as fs;");
  }
  replace(
    "crates/vize_maestro/src/server/state/global_tag_names.rs",
    "use vize_l0::{String, ToCompactString, source_io as fs};",
    "use vize_carton::source_io as fs;\nuse vize_l0::{String, ToCompactString};",
  );
}

const mode = process.argv[2];
assert.ok(
  mode === "--move-only" || mode === "--integrate",
  "usage: node tools/support/compat/levels/move-source-io-host.ts --move-only | --integrate",
);
if (mode === "--move-only") moveOnly();
else {
  integrate();
  // No integration writes occur until every actual caller has passed preflight.
  for (const [file, text] of planned) {
    const target = path.join(root, file);
    mkdirSync(path.dirname(target), { recursive: true });
    writeFileSync(target, text);
  }
}
