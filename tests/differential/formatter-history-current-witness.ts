import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { sha256 } from "./manifest.mjs";
import { validateDeclarationSnapshotAuthority } from "./formatter-declaration-reference.ts";
import { validateExpressionWidthWitness } from "./formatter-expression-width-witness.ts";
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
  // #7826 shares the existing lexer helpers; the original escape law is unchanged.
  "crates/vize_glyph/src/style/comment_scan.rs": {
    originalSha256: "1b1dc3ae107887fb740a4139778a38852be910e4299f530269a337ddaf832cb6",
    actualMainSha256: "ada9e0e8ec0f25f7a3e20cb0e50200054932aac32bb85ced7906f63f19f79102",
    functions: {
      css_escapes_do_not_affect_comment_depth:
        "74fd329fbcaa5a624c497e676c21072aff763aaa477339cc87c2708bd61e8ac0",
    },
  },
  // #7704 line endings and #3295 private boundary registration retain all eight original laws.
  "crates/vize_glyph/src/style.rs": {
    originalSha256: "8e13635eab09e08a32282d372ef5379c22210cc4117fbccf05d164df76233a61",
    actualMainSha256: "ae84ca8f74dd923440bfdb57a26bc1e10367363acdaa178c6950a5bf21bc942f",
    functions: {
      test_style_numbers_match_standalone_css_leading_zeroes:
        "768c1b2fee61409a7034b99c2a7a277d7ac19699302a94b36d58dd54912937cb",
      test_format_preserves_nested_block_comments_by_leaving_block_raw:
        "d75832be3024ed959e9f8449a327f9c378acceefde338e6d20755d7e9aae5237",
      test_background_position_shorthand_stays_authored_and_stable:
        "2789c0f1a1ee365e550be6ef1743142c02939a4a40d932a6a014da1883363556",
      test_format_nested_css_at_rule:
        "8e0bf6f220ae5a4835ee7f70023c216a1a4f1dea16990f1cda52af29e9b998e8",
      test_format_simple_css: "85da41d455cec5e0ef4256603b9846a0cc583b4cee9eafd3f773b1c8d7436089",
      test_format_empty_css: "9e1b0a77d8008bcfa0475213d4776c295055865d2f0d306c43f2ecee757bf1cb",
      test_format_css_whitespace_only:
        "af60fef48d17c6a45c60399972414080f9b18faec296752525242dd4026866b2",
      style_block_keeps_box_values_and_implicit_nested_selectors:
        "e3653ec69d33d2b1063140d14b8fc065e339a6d258c23052f2f986847227fa3e",
    },
  },
  // #7704 changes production line endings; complete original law bodies are retained.
  "crates/vize_glyph/src/formatter/block_indent.rs": {
    originalSha256: "3fb14d8c80972c6222de66e01196e7b0072039688a75722c3ea76bf0addec023",
    actualMainSha256: "6fd2362a08932f4ecbe3ce9607787f960a48111cf4106260f854bc0242a8ffaf",
    functions: {
      ordinary_lines_are_indented:
        "2db24947771836ab456fd330963996617b6ef559d76d3c6014b34941c9859979",
      template_literal_continuation_lines_stay_verbatim:
        "51748688a59c07b5545a56b1fca1c88d6d306fb4305a60f0588eebba2946b405",
      indentation_resumes_after_the_literal_closes:
        "f5ca6057e2fdbc1dbdd7c2645dd0988747d92e53dc46d5824e36323739a172cd",
      escaped_backticks_do_not_toggle_the_literal:
        "2a81f12c65e4d68cb5f31cd4e6425d4bd10fc303bf8d00bd22e9b1e3558c9060",
      nested_literals_close_correctly:
        "d19b0bda1e6c2a69f688d2705ec93e38f4a16d7ed966723352203f116ae8081c",
      blank_lines_are_never_indented:
        "a8d287daa40a7bc176830e013de3bcdda567d1883b32b1f4c5dbba67a01a1660",
      crlf_content_emits_the_configured_newline_once:
        "027a642da3fbafafc2060c882ea6c86824a71ec964fe901acb2494a6e382f2f4",
    },
  },
  // #7697 changes CRLF production emission; all four original LF/raw laws
  // remain byte-identical to their complete cc87 source bodies.
  "crates/vize_glyph/src/formatter/template_indent.rs": {
    originalSha256: "4e610e8497708dc137ae90209526526875cc70154c376a4ee6e8098f7caccef6",
    actualMainSha256: "dd465917c25d364f94298f984a8491ecf75ceb152cb3b622fff08aa007ea7e3a",
    functions: {
      ordinary_templates_bypass_the_raw_line_mask:
        "252d13f191f01a540b78024c0d9e21603dc9a1a14b4c92cc56594ac3fb4cc115",
      every_raw_continuation_shape_keeps_the_full_lexer:
        "0588d2d49af44e13ec3c98ed1c9fa38bd68b1cee068f1b22be09a67ec63dffa5",
      opaque_templates_rebase_only_the_shared_prefix:
        "1e351dc6ca35052214415b656b0718d1d276a027136cc3a87f62f1dfe6c2bcb1",
      empty_opaque_templates_emit_no_phantom_body_line:
        "4996ca42deefa69e5fbc9dc5061e720a204ecfad2bd96a5714c57ba5c9d7c883",
    },
  },
  "crates/vize_glyph/src/script/block_identity.rs": {
    originalSha256: "e0cf30415eb6585ecd6688da72effc7b560868a0e31acf042507810d468c8eff",
    actualMainSha256: "70b4c8e7db28eece8dc4f22a4e34cdfeea5da293f57fe5a2a53533d6a6f1c926",
    functions: {
      classifies_only_comment_free_empty_statements:
        "aeb192094f63391b90c1c078b0c17e49e85702af5daa9122e77c8edcacdff928",
    },
  },
  // #7880/#8087 and #7876 expression extraction retain all ten complete original script laws.
  "crates/vize_glyph/src/script.rs": {
    originalSha256: "a205174795bb6d993e84b0cc29dfc5d10c36602dfb6216f1db2278a982b2e0ab",
    actualMainSha256: "30ece91fbfcd338dbe999a9fb1ba5d2d8d65dfee78f5a07c032a4685e666eb78",
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
  // #7876 shares a read-only matched-close query; the original placement law is unchanged.
  "crates/vize_glyph/src/template/formatter/suppression.rs": {
    originalSha256: "94b99ca1dcf0833d5706b73f95e41c9cdf280160e1dccdb751340ad86607b675",
    actualMainSha256: "df1db7ef1b3a4ac0f854e3228a2b2895e3bb45e3768118f79e2e1b9d9aecdcfd",
    functions: {
      ranges_track_pragma_placement:
        "8120d76a61ca7b815e73c633c93431ba5399f1227d8274a699d09c7d2a5a5400",
    },
  },
};

export function validateCurrentFormatterWitness(root: string, entry: any, name: string) {
  if (validateExpressionWidthWitness(root, entry, name)) return;
  if (
    entry.path === "crates/vize_glyph/src/style.rs" &&
    name === "test_format_nested_css_at_rule"
  ) {
    validateDeclarationSnapshotAuthority(root);
  }
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
