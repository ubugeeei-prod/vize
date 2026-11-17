// Rules the Vue 2 dialect turns off in Patina.
//
// A Vue 2 or 2.7 project is linted with its dialect (matrix_rule_options.ts),
// which turns off Vue 3 migration rules such as vue/no-deprecated-filter:
// they flag code that is correct in Vue 2. The eslint-plugin-vue baseline
// still runs them, so their findings in such a project are classified with
// this reason. The list is read from `Linter::with_vue_version`, so it cannot
// drift from Patina.

import fs from "node:fs";
import path from "node:path";

export const vue2DialectReason =
  "Vize lints this Vue 2 project with the Vue 2 dialect, which turns off this Vue 3 migration rule; the eslint-plugin-vue baseline still applies it to Vue 2 code.";

/** The Patina rules `with_vue_version` disables for a legacy Vue version. */
export function vue2DialectDisabledRules(root: string): Set<string> {
  const file = path.join(root, "crates/vize_patina/src/linter/compatibility.rs");
  const source = fs.readFileSync(file, "utf8");
  const open = source.indexOf("{", source.indexOf("if version.is_some_and(VueVersion::is_legacy)"));
  if (open < 0 || !source.includes("if version.is_some_and(VueVersion::is_legacy)"))
    throw new Error(`${file} no longer gates rules on a legacy Vue version`);
  let depth = 0,
    close = open;
  for (; close < source.length; close++) {
    if (source[close] === "{") depth++;
    else if (source[close] === "}" && --depth === 0) break;
  }
  const rules = new Set(
    [...source.slice(open, close).matchAll(/String::from\("([^"]+)"\)/gu)].map(
      (match) => match[1] ?? "",
    ),
  );
  if (rules.size === 0) throw new Error(`${file} disables no rule for a legacy Vue version`);
  return rules;
}
