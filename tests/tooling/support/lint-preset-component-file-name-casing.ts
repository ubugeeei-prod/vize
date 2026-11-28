import assert from "node:assert/strict";

import { canonical, identity, jsonBytes } from "./lint-preset-source-successors.ts";

const membershipPath =
  "crates/vize_patina/src/preset/snapshots/vize_patina__preset__tests__lint_preset_rule_membership.snap";
const happyPathPath = "crates/vize_patina/src/rule/happy_path.rs";
const ruleMapPath = "tests/_fixtures/patina-eslint-vue-rule-map.json";
const definitionRule = "vue/component-definition-name-casing";

// The SFC file-name check moves from vue/component-definition-name-casing,
// which now checks definition names, to vue/component-file-name-casing in
// every preset that ran it.
const rule = "vue/component-file-name-casing";
const presets = ["ecosystem", "happy_path", "nuxt", "opinionated"];
const registration = [
  `        registry.register(Box::new(
            crate::rules::vue::ComponentDefinitionNameCasing::default(),
        ));
        registry.register(Box::new(crate::rules::vue::ComponentFileNameCasing));
`,
  `        registry.register(Box::new(crate::rules::vue::ComponentDefinitionNameCasing));
`,
] as const;

// Remove only that exact registration and membership before the older ledgers
// replay. Whole-file identities keep any later edit fatal until it receives
// its own finite successor.
export function inverseComponentFileNameCasing(name: string, body: Buffer): Buffer {
  if (name === ruleMapPath) {
    identity(body, {
      bytes: 56118,
      sha256: "2e5bc588164b260718fd288a46a4cad2123497ae454da8e9947c739b7bc9d565",
    });
    const value = canonical(body);
    assert.deepEqual(value.entries[definitionRule], {
      patinaPresets: ["ecosystem", "general-recommended", "nuxt", "opinionated"],
      patinaRule: definitionRule,
      patinaSeverity: "warning",
      status: "mapped",
    });
    value.entries[definitionRule] = {
      reason:
        "Patina's rule checks SFC file-name casing; eslint-plugin-vue checks the component definition name, so their findings are not comparable.",
      status: "intentional-divergence",
    };
    value.summary.mapped -= 1;
    value.summary.intentionalDivergence += 1;
    const predecessor = jsonBytes(value);
    identity(predecessor, {
      bytes: 56071,
      sha256: "0d218a073ef430f92fb55996f772166c6449fcdd0e1d0a67963a7c9148feed6b",
    });
    return predecessor;
  }
  if (name === happyPathPath) {
    identity(body, {
      bytes: 9056,
      sha256: "9a34d1db072900b0f63f9900a5e56fd446edf367b7a2c06390c1a2bda0228287",
    });
    const text = body.toString();
    const [after, before] = registration;
    assert.equal(text.split(after).length, 2);
    const predecessor = Buffer.from(text.replace(after, before));
    identity(predecessor, {
      bytes: 8941,
      sha256: "7e00806c2694eeb5c871328ebf0f4ca508aa7ddde902e29e1ac2c2caba1e28db",
    });
    return predecessor;
  }
  if (name !== membershipPath) return body;
  identity(body, {
    bytes: 40106,
    sha256: "720fee52a74a32d5685d5a2fa77b9d65c6cbfd08ddc7d1424ac050d7749621ed",
  });
  const text = body.toString();
  const at = text.indexOf("{");
  const table = canonical(Buffer.from(text.slice(at)));
  for (const [preset, rules] of Object.entries(table)) {
    if (!Array.isArray(rules)) continue;
    const count = rules.filter((entry) => entry === rule).length;
    assert.equal(count, presets.includes(preset) ? 1 : 0, preset);
    if (count === 1) rules.splice(rules.indexOf(rule), 1);
  }
  const predecessor = Buffer.concat([Buffer.from(text.slice(0, at)), jsonBytes(table)]);
  identity(predecessor, {
    bytes: 39954,
    sha256: "4e27911d86ffe7fa03b7dfa8b183f352536b9b10892d83468d0f9581efaafcf3",
  });
  return predecessor;
}
