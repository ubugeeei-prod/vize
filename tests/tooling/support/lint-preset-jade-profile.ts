import assert from "node:assert/strict";

import { canonical, identity, jsonBytes } from "./lint-preset-source-successors.ts";

const membershipPath =
  "crates/vize_patina/src/preset/snapshots/vize_patina__preset__tests__lint_preset_rule_membership.snap";
const happyPathPath = "crates/vize_patina/src/rule/happy_path.rs";

// #13262 adds vue/prefer-pug-lang to the happy-path preset and to the presets
// that extend it.
const jadeRule = "vue/prefer-pug-lang";
const jadePresets = ["ecosystem", "happy_path", "nuxt", "opinionated"];
const jadeRegistration = "        registry.register(Box::new(crate::rules::vue::PreferPugLang));\n";

// Remove only those exact registration and membership additions before the
// older ledgers replay. Whole-file identities keep any later edit fatal until
// it receives its own finite successor.
export function inverseJadeProfile(name: string, body: Buffer): Buffer {
  if (name === happyPathPath) {
    identity(body, {
      bytes: 8941,
      sha256: "7e00806c2694eeb5c871328ebf0f4ca508aa7ddde902e29e1ac2c2caba1e28db",
    });
    const text = body.toString();
    assert.equal(text.split(jadeRegistration).length, 2);
    const predecessor = Buffer.from(text.replace(jadeRegistration, ""));
    identity(predecessor, {
      bytes: 8870,
      sha256: "162f7c1336380e6cdb4b917f512052d0da49e7747fa749559a0754f1f95b7e80",
    });
    return predecessor;
  }
  if (name !== membershipPath) return body;
  identity(body, {
    bytes: 39916,
    sha256: "87ca2b8cd7d533045bcbb09e79e2cf4eba5f9b11f30b39a63b8fad0ef423b51c",
  });
  const text = body.toString();
  const at = text.indexOf("{");
  const table = canonical(Buffer.from(text.slice(at)));
  for (const [preset, rules] of Object.entries(table)) {
    if (!Array.isArray(rules)) continue;
    const count = rules.filter((rule) => rule === jadeRule).length;
    assert.equal(count, jadePresets.includes(preset) ? 1 : 0, preset);
    if (count === 1) rules.splice(rules.indexOf(jadeRule), 1);
  }
  const predecessor = Buffer.concat([Buffer.from(text.slice(0, at)), jsonBytes(table)]);
  identity(predecessor, {
    bytes: 39808,
    sha256: "abee0c1b443a26a723be362e702d086cff8fac9fab46ee48247573a7cee3e948",
  });
  return predecessor;
}
