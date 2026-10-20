import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { retainedFormatterFunction } from "./formatter-history-source-artifact.ts";
import { sha256 } from "./manifest.mjs";
import { stripRust } from "../../tools/support/compat/davinci/lib/rust-source.ts";

// These exact retained laws own configuration input, never an output oracle.
export function assertOriginalSortingConfig(fixture, owner, initialFiles, root) {
  const name = fixture.witness.function;
  const liveLaw = retainedFormatterFunction(owner, name);
  const preserved = fs.readFileSync(
    path.join(
      root,
      "tests/_fixtures/differential/formatter-history/source-witnesses/fmt_import_sorting_cli.pre-vue-config.txt",
    ),
  );
  assert.equal(
    sha256(preserved),
    "4e249c4cf14dacc1cdda3e3010b00350cdee9bd4d09b0548ffba9d58dfe88d0d",
  );
  const currentLaws = {
    malformed_sort_settings_fail_closed_and_no_config_bypasses_them:
      "9ac7d29c94070b76406a2f74387ccb446e4980434cbc4b39ccf5ec056287bb3b",
    contradictory_sort_settings_cannot_write_authored_sources:
      "8516ebc84bc6beccc9b68a299b940b99e27640c548eaa60e151b951fbcec727b",
  };
  const originalLaw = retainedFormatterFunction(preserved, name);
  const law = sha256(liveLaw) === fixture.witness.functionSha256 ? liveLaw : originalLaw;
  if (law !== liveLaw)
    assert.equal(sha256(liveLaw), currentLaws[name], "current migrated config law changed");
  assert.equal(sha256(law), fixture.witness.functionSha256, "original config law changed");
  const raw = law.toString();
  let inputs;
  if (name === "malformed_sort_settings_fail_closed_and_no_config_bypasses_them") {
    const clean = stripRust(raw);
    const start = clean.match(/\bfor\s+invalid\s+in\s*\[/);
    assert(start, "original invalid configuration array missing");
    const open = clean.indexOf("[", start.index),
      close = clean.indexOf("]", open + 1);
    assert(close > open);
    const list = raw.slice(open + 1, close);
    assert.match(stripRust(list), /^[\s,]*$/, "original config array changed shape");
    inputs = list.match(/r#"[\s\S]*?"#|"(?:\\.|[^"\\])*"/g);
    assert.equal(inputs?.length, 7, "original malformed configuration set changed");
  } else {
    assert.equal(name, "contradictory_sort_settings_cannot_write_authored_sources");
    inputs = raw.match(/r#"[\s\S]*?"#/g);
    assert.equal(inputs?.length, 1, "original contradictory configuration changed");
  }
  const { file, literal } = fixture.historicalConfig;
  assert(inputs.includes(literal), "literal is not an original configuration input");
  assert(liveLaw.toString().includes(literal), "migrated law lost an original config input");
  const authored =
    literal.startsWith('r#"') && literal.endsWith('"#')
      ? literal.slice(3, -2)
      : JSON.parse(literal);
  assert.equal(typeof authored, "string");
  assert(initialFiles[file]?.equals(Buffer.from(authored)), "original config bytes changed");
}
