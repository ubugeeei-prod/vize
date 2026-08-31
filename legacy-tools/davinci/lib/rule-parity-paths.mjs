// Repository locations and the shared rule vocabulary of the rule-parity
// matrix generator's stages. Keeping the artifact paths in one place keeps the
// regeneration command quoted in the artifact header in sync with the command
// the CLI actually accepts.

import path from "node:path";
import { fileURLToPath } from "node:url";

export const repoRoot = path.resolve(
  path.dirname(fileURLToPath(import.meta.url)),
  "..",
  "..",
  "..",
);
export const PATINA_SRC = path.join(repoRoot, "crates", "vize_patina", "src");
export const RULES_DIR = path.join(PATINA_SRC, "rules");
export const ARTIFACT_REL = "davinci-road/plan/rule-parity.md";
export const ARTIFACT = path.join(repoRoot, ARTIFACT_REL);
export const OVERRIDES_REL = "davinci-road/plan/rule-parity-overrides.toml";
export const OVERRIDES = path.join(repoRoot, OVERRIDES_REL);
export const REGEN_COMMAND = "rust-script tools/commands/davinci/rule-parity.rs --write";

/** `static META` type -> rule family, the partition every count is grouped by. */
export const META_KINDS = new Map([
  ["RuleMeta", "template-family"],
  ["ScriptRuleMeta", "script"],
  ["CssRuleMeta", "css"],
  ["MuseaRuleMeta", "musea"],
]);
export const RULE_TRAITS = ["Rule", "MarkupRule", "ScriptRule", "CssRule", "MuseaRule"];
export const CLASSIFICATIONS = ["neutral-core-candidate", "vue-dialect-bound", "container-bound"];
