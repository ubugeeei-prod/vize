import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { sha256 } from "./manifest.mjs";
import {
  retainedFormatterFunction,
  validatePreservedFormatterWitness,
} from "./formatter-history-source-artifact.ts";
export { retainedFormatterFunction } from "./formatter-history-source-artifact.ts";

// Main #7382 moved script formatting and retained these eleven test bodies.
// #7681 production setup changes retain all ten script bodies and the original
// suppression-placement law. Only complete current-owner hashes change here.
// The original audit and captured source pins remain unchanged. These exact
// owner transitions admit retained laws, never fresh output/execution credit.
// Each function hash was verified against both complete original/current Git
// owners before publication; source gates require no historical Git objects.
const ORIGINAL = "cc87bb5960ea9e49e82672205df919de58bb4b24";
const OWNERS: Record<
  string,
  { originalSha256: string; actualMainSha256: string; functions: Record<string, string> }
> = {
  "crates/vize_glyph/src/script/block_identity.rs": {
    originalSha256: "e0cf30415eb6585ecd6688da72effc7b560868a0e31acf042507810d468c8eff",
    actualMainSha256: "70b4c8e7db28eece8dc4f22a4e34cdfeea5da293f57fe5a2a53533d6a6f1c926",
    functions: {
      classifies_only_comment_free_empty_statements:
        "aeb192094f63391b90c1c078b0c17e49e85702af5daa9122e77c8edcacdff928",
    },
  },
  "crates/vize_glyph/src/script.rs": {
    originalSha256: "a205174795bb6d993e84b0cc29dfc5d10c36602dfb6216f1db2278a982b2e0ab",
    actualMainSha256: "553ea84d9f4a87c842735b0c4399b2ea6a4fcddd5c37201395c1176371b63a69",
    functions: {
      test_format_tsx_component_script:
        "d108e897e74d4289b8fdf44555eed4b7ad551aff2ac7673b5a80ca8601220d9b",
      test_format_jsx_component_script:
        "310ba1b4c74a4d5bb9952aca1cecc0828ae9cac00ad7938ca299920370ac8ff7",
      test_format_js_expression_simple:
        "04e1429c6391d2032f5c537cdbcfce4b35d2ad9ced9d0c8fbeeeaa4eeca8cd3b",
      test_format_js_expression_with_optional_chaining:
        "8b2def3c19322e4dba7597cfdc8ce44f5849593b94f16533f3ba4bf8c26772f3",
      test_format_js_expression_empty:
        "e424ad71e7b89482e11fddca37413c6c3a9e1879c4c9ca0c47d5b014695da080",
      test_format_simple_script: "0bd325dc4c1f7d8f96ceb6bb017278b48377e7ff45e63b9ab02851869a783c76",
      test_format_with_imports: "82305229468aa40941a032afc2c0fb60f1480c10b571f1520369e96742c24769",
      test_format_object: "20fdf3e1770a2f0bddc1b39c7a3dca536edcf37935c486da2952e241eff758de",
      test_format_empty_source: "517a6c6ae4e6449b6c4464e959aeb76f401ccb389c304472b76cf24cf2c5c3f1",
      test_format_whitespace_only:
        "958b802bade6ac87002b32dff6a6d36a24a9ace640147c100898a7665c6a9a1a",
    },
  },
  "crates/vize_glyph/src/template/formatter/suppression.rs": {
    originalSha256: "94b99ca1dcf0833d5706b73f95e41c9cdf280160e1dccdb751340ad86607b675",
    actualMainSha256: "6ed649ce147cb3979b3ee76bbc9eb90d4e5848eb252a487d4660b2ff8c5dc10e",
    functions: {
      ranges_track_pragma_placement:
        "8120d76a61ca7b815e73c633c93431ba5399f1227d8274a699d09c7d2a5a5400",
    },
  },
};

export function validateCurrentFormatterWitness(root: string, entry: any, name: string) {
  if (validatePreservedFormatterWitness(root, entry, name)) return;
  const bytes = fs.readFileSync(path.join(root, entry.path));
  const actual = sha256(bytes);
  const owner = Object.hasOwn(OWNERS, entry.path) ? OWNERS[entry.path]! : undefined;
  if (owner) {
    assert.equal(entry.sha256, owner.originalSha256, "original Rust law pin changed");
    assert(entry.revisions.includes(ORIGINAL), "original Rust law revision changed");
  }
  if (actual === entry.sha256) {
    assert(bytes.toString().includes(`fn ${name}(`), "retained Rust law is missing");
    return;
  }
  assert(owner, "current witness source changed");
  assert.equal(actual, owner.actualMainSha256, "current witness source changed");
  assert(Object.hasOwn(owner.functions, name), "unregistered retained Rust law transition");
  const currentBody = retainedFormatterFunction(bytes, name);
  assert.equal(sha256(currentBody), owner.functions[name], "retained Rust law body changed");
}
