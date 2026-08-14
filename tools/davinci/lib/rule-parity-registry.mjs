// Registration surfaces: which rules the dispatch paths can actually reach.
// Every `register…(Box::new(…))` site under vize_patina/src is collected
// mechanically, and the script/css block registries are read from their own
// name tables, so an unregistered rule surfaces instead of being assumed live.

import { readFileSync } from "node:fs";
import path from "node:path";

import { walkRustFiles } from "./crates.mjs";
import { PATINA_SRC } from "./rule-parity-paths.mjs";
import { stripRustComments } from "./rule-parity-rust-text.mjs";
import { stripRust } from "./rust-source.mjs";

/** Type names registered into a RuleRegistry anywhere in vize_patina/src. */
export function collectRegisteredRuleTypes() {
  const types = new Set();
  const patterns = [
    // registry.register(Box::new(Type…)) / registry.add(Box::new(Type…))
    /\.\s*(?:register|add)\s*\(\s*Box::new\s*\(\s*((?:[A-Za-z_][A-Za-z0-9_]*::)*)([A-Z][A-Za-z0-9_]*)/g,
    // register_if_missing(registry, Box::new(Type…))
    /register_if_missing\s*\(\s*registry\s*,\s*Box::new\s*\(\s*((?:[A-Za-z_][A-Za-z0-9_]*::)*)([A-Z][A-Za-z0-9_]*)/g,
  ];
  for (const abs of walkRustFiles(PATINA_SRC)) {
    const stripped = stripRust(readFileSync(abs, "utf8"));
    for (const re of patterns) {
      re.lastIndex = 0;
      let m;
      while ((m = re.exec(stripped)) !== null) types.add(m[2]);
    }
  }
  return types;
}

/** Rule names in the built-in script registry the SFC path dispatches. */
export function collectScriptRegistryNames() {
  const namesRs = stripRustComments(
    readFileSync(path.join(PATINA_SRC, "linter", "script_rules", "registry", "names.rs"), "utf8"),
  );
  const consts = new Map();
  for (const m of namesRs.matchAll(/const\s+([A-Z_0-9]+)\s*:\s*&str\s*=\s*"([^"]+)"/g)) {
    consts.set(m[1], m[2]);
  }
  const rulesRs = stripRustComments(
    readFileSync(path.join(PATINA_SRC, "linter", "script_rules", "registry", "rules.rs"), "utf8"),
  );
  const registered = new Set();
  for (const m of rulesRs.matchAll(/rule_name:\s*([A-Z_0-9]+)/g)) {
    const value = consts.get(m[1]);
    if (!value) throw new Error(`script registry rule_name const ${m[1]} not found in names.rs`);
    registered.add(value);
  }
  // Cross-check against the ALL list in names.rs (count only; both live there).
  const allList = /ALL_BUILTIN_SCRIPT_RULE_NAMES\s*:\s*&\[&str\]\s*=\s*&\[([^\]]*)\]/.exec(namesRs);
  const allCount = allList ? [...allList[1].matchAll(/[A-Z_0-9]+/g)].length : -1;
  return { registered, allCount };
}

/** Rule names in the built-in css registry the SFC path dispatches. */
export function collectCssRegistryNames() {
  const cssRs = stripRustComments(
    readFileSync(path.join(PATINA_SRC, "linter", "css_rules.rs"), "utf8"),
  );
  const consts = new Map();
  for (const m of cssRs.matchAll(/const\s+(RULE_[A-Z_0-9]+)\s*:\s*&str\s*=\s*"([^"]+)"/g)) {
    consts.set(m[1], m[2]);
  }
  const allList = /ALL_BUILTIN_CSS_RULE_NAMES\s*:\s*&\[&str\]\s*=\s*&\[([^\]]*)\]/.exec(cssRs);
  if (!allList) throw new Error("ALL_BUILTIN_CSS_RULE_NAMES not found in css_rules.rs");
  const registered = new Set();
  for (const m of allList[1].matchAll(/RULE_[A-Z_0-9]+/g)) {
    const value = consts.get(m[0]);
    if (!value) throw new Error(`css registry const ${m[0]} has no string value`);
    registered.add(value);
  }
  return registered;
}
