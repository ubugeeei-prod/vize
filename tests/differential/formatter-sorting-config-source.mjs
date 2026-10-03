import assert from "node:assert/strict";
import { retainedFormatterFunction } from "./formatter-history-source-artifact.ts";
import { sha256 } from "./manifest.mjs";
import { stripRust } from "../../tools/support/compat/davinci/lib/rust-source.mjs";

// These exact retained laws own configuration input, never an output oracle.
export function assertOriginalSortingConfig(fixture, owner, initialFiles) {
  const name = fixture.witness.function;
  const law = retainedFormatterFunction(owner, name);
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
  const authored =
    literal.startsWith('r#"') && literal.endsWith('"#')
      ? literal.slice(3, -2)
      : JSON.parse(literal);
  assert.equal(typeof authored, "string");
  assert(initialFiles[file]?.equals(Buffer.from(authored)), "original config bytes changed");
}
