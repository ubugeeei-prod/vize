import assert from "node:assert/strict";

import { canonical, identity, jsonBytes } from "./lint-preset-source-successors.ts";

const membershipPath =
  "crates/vize_patina/src/preset/snapshots/vize_patina__preset__tests__lint_preset_rule_membership.snap";
const opinionatedVuePath = "crates/vize_patina/src/rules/opinionated/vue.rs";

// #12878 adds the script/template unused-binding pair, and #12879 adds the
// typescript-eslint type-aware checks, to the opinionated and Nuxt presets.
const scriptUnusedRules = ["vue/no-unused-script-vars", "vue/prefer-const"] as const;
const typeAwareRules = [
  "type/await-thenable",
  "type/no-array-delete",
  "type/no-base-to-string",
  "type/no-confusing-void-expression",
  "type/no-deprecated",
  "type/no-duplicate-type-constituents",
  "type/no-for-in-array",
  "type/no-generated-empty-object-type",
  "type/no-implied-eval",
  "type/no-meaningless-void-operator",
  "type/no-misused-promises",
  "type/no-misused-spread",
  "type/no-mixed-enums",
  "type/no-redundant-type-constituents",
  "type/no-unnecessary-boolean-literal-compare",
  "type/no-unnecessary-condition",
  "type/no-unnecessary-template-expression",
  "type/no-unnecessary-type-arguments",
  "type/no-unnecessary-type-assertion",
  "type/no-unnecessary-type-conversion",
  "type/no-unsafe-argument",
  "type/no-unsafe-assignment",
  "type/no-unsafe-call",
  "type/no-unsafe-enum-assignment",
  "type/no-unsafe-enum-comparison",
  "type/no-unsafe-member-access",
  "type/no-unsafe-return",
  "type/no-unsafe-unary-minus",
  "type/only-throw-error",
  "type/prefer-promise-reject-errors",
  "type/prefer-reduce-type-parameter",
  "type/prefer-return-this-type",
  "type/related-getter-setter-pairs",
  "type/require-await",
  "type/restrict-plus-operands",
  "type/restrict-template-expressions",
  "type/return-await",
  "type/unbound-method",
  "type/use-unknown-in-catch-callback-variable",
] as const;
const addedPresets = ["nuxt", "opinionated"] as const;
// #12874 adds the opt-in vuejs-accessibility role="img" name check.
const roleImgRule = "a11y/role-img-has-name";

const opinionatedVueAdditions = [
  "mod script_unused;\n",
  "pub(crate) use script_unused::{NoUnusedScriptVars, PreferConst};\n",
  "    registry.register(Box::new(script_unused::NoUnusedScriptVars::default()));\n",
  "    registry.register(Box::new(script_unused::PreferConst::default()));\n",
] as const;

// Remove only the exact #12874, #12878 and #12879 preset and registration additions
// before the older ledgers replay. Whole-file identities keep any later edit
// fatal until it receives its own finite successor.
export function inverseScriptUnusedAndTypeAwarePresets(name: string, body: Buffer): Buffer {
  if (name === membershipPath) {
    identity(body, {
      bytes: 39683,
      sha256: "a5b5849b77cdc7f3c38e90cea2f5ac9d5311bd017c0c74108ee835a9cda68b9b",
    });
    const text = body.toString();
    const at = text.indexOf("{");
    const table = canonical(Buffer.from(text.slice(at)));
    for (const preset of addedPresets) {
      for (const rule of [...scriptUnusedRules, ...typeAwareRules]) {
        const rules: string[] = table[preset];
        assert.equal(rules.filter((entry) => entry === rule).length, 1, `${preset}: ${rule}`);
        rules.splice(rules.indexOf(rule), 1);
      }
    }
    assert.equal(table.opt_in.filter((entry: string) => entry === roleImgRule).length, 1);
    table.opt_in.splice(table.opt_in.indexOf(roleImgRule), 1);
    const predecessor = Buffer.concat([Buffer.from(text.slice(0, at)), jsonBytes(table)]);
    identity(predecessor, {
      bytes: 36763,
      sha256: "b6323fe83900c7dfe8bd59556af7792b67ccd9c98b74c18a928f63731a56fb47",
    });
    return predecessor;
  }
  if (name === opinionatedVuePath) {
    identity(body, {
      bytes: 5434,
      sha256: "6d1732edac22fb779fa9f54b177ba7c19765ded591774738eba301506cc52d85",
    });
    let text = body.toString();
    for (const addition of opinionatedVueAdditions) {
      assert.equal(text.split(addition).length, 2, addition);
      text = text.replace(addition, "");
    }
    const predecessor = Buffer.from(text);
    identity(predecessor, {
      bytes: 5199,
      sha256: "cc576d6b49b3e23e6752a05e88d2b26c269f66031866301f72a12de0957e9732",
    });
    return predecessor;
  }
  return body;
}
