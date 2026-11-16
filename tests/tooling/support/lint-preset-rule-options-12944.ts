import assert from "node:assert/strict";

import { canonical, identity, jsonBytes } from "./lint-preset-source-successors.ts";

const ruleMapPath = "tests/_fixtures/patina-eslint-vue-rule-map.json";
const membershipPath =
  "crates/vize_patina/src/preset/snapshots/vize_patina__preset__tests__lint_preset_rule_membership.snap";

// #12946 ports the opt-in vue/prefer-single-event-payload, and #12944 maps
// vue/v-on-handler-style, whose eslint-plugin-vue styles are now options.
const portedRule = "vue/prefer-single-event-payload";
const handlerStyle = "vue/v-on-handler-style";
const handlerStyleDivergence =
  "Patina's opinionated policy permits only method references, while eslint-plugin-vue always permits two of method, inline, and inline-function handlers; no upstream option has the same acceptance surface.";

// Remove only the exact #12944 and #12946 rule-map and membership additions
// before the older ledgers replay. Whole-file identities keep any later edit
// fatal until it receives its own finite successor.
export function inverseRuleOptions12944(name: string, body: Buffer): Buffer {
  if (name === ruleMapPath) {
    identity(body, {
      bytes: 55313,
      sha256: "9db2b7af4e34df904ac5d53727d9e80726a9f07b42c0c618d61aeb578a98f15e",
    });
    const value = canonical(body);
    assert.deepEqual(value.summary, { intentionalDivergence: 2, mapped: 200, unimplemented: 54 });
    assert.deepEqual(value.entries[portedRule], {
      patinaPresets: [],
      patinaRule: portedRule,
      patinaSeverity: "warning",
      status: "mapped",
    });
    assert.deepEqual(value.entries[handlerStyle], {
      patinaPresets: ["nuxt", "opinionated"],
      patinaRule: handlerStyle,
      patinaSeverity: "warning",
      status: "mapped",
    });
    value.entries[portedRule] = { issue: 3223, status: "unimplemented" };
    value.entries[handlerStyle] = {
      reason: handlerStyleDivergence,
      status: "intentional-divergence",
    };
    value.summary = { intentionalDivergence: 3, mapped: 198, unimplemented: 55 };
    const predecessor = jsonBytes(value);
    identity(predecessor, {
      bytes: 55310,
      sha256: "819787315f085049e4185333df685e950bb7fcdd94c1e1e0a99b556890f6be33",
    });
    return predecessor;
  }
  if (name === membershipPath) {
    identity(body, {
      bytes: 39808,
      sha256: "abee0c1b443a26a723be362e702d086cff8fac9fab46ee48247573a7cee3e948",
    });
    const text = body.toString();
    const at = text.indexOf("{");
    const table = canonical(Buffer.from(text.slice(at)));
    assert.equal(table.opt_in.filter((entry: string) => entry === portedRule).length, 1);
    table.opt_in.splice(table.opt_in.indexOf(portedRule), 1);
    const predecessor = Buffer.concat([Buffer.from(text.slice(0, at)), jsonBytes(table)]);
    identity(predecessor, {
      bytes: 39769,
      sha256: "d093ed93fcd632850cf06003918a7d67ab073ea582bfe6f8cbd0ccb562152df4",
    });
    return predecessor;
  }
  return body;
}
