/**
 * Post-processes generated TypeScript types from JSON Schema.
 *
 * Adds TS-specific type unions that cannot be expressed in Pkl/JSON Schema:
 * - VitePluginConfig.include/exclude: adds RegExp union
 * - Fixes naming inconsistencies from json-schema-to-typescript
 * - Removes $schema from VizeConfig (internal only)
 * - Adds shared lint type aliases
 */

import * as fs from "node:fs";
import * as path from "node:path";

const GENERATED_PATH = path.resolve(import.meta.dirname, "../npm/vize/src/types/generated.ts");

let content = fs.readFileSync(GENERATED_PATH, "utf-8");

// Fix MuseaA11YConfig -> MuseaA11yConfig (json2ts capitalizes "11Y")
content = content.replaceAll("MuseaA11YConfig", "MuseaA11yConfig");

// Add RegExp union to VitePluginConfig include/exclude
content = content.replace(
  /include\?\s*:\s*string\s*\|\s*string\[\]/g,
  "include?: string | RegExp | (string | RegExp)[]",
);
content = content.replace(
  /exclude\?\s*:\s*string\s*\|\s*string\[\]/g,
  "exclude?: string | RegExp | (string | RegExp)[]",
);

// Remove $schema property from VizeConfig (not part of user-facing API)
content = content.replace(
  /  \/\*\*\n   \* JSON Schema reference for editor autocompletion\n   \*\/\n  \$schema\?: string;\n/,
  "",
);

// Add shared lint type aliases after the generated header
// The header ends with "regenerate this file.\n */\n"
const marker = "regenerate this file.\n */\n";
const markerIdx = content.indexOf(marker);
if (markerIdx !== -1) {
  const insertPos = markerIdx + marker.length;
  const typeAliases = `
export type LintPreset = "happy-path" | "opinionated" | "essential" | "incremental" | "nuxt";

export type RuleSeverity = "off" | "warn" | "error";

export type RuleCategory = "correctness" | "suspicious" | "style" | "perf" | "a11y" | "security";
`;
  content = content.slice(0, insertPos) + typeAliases + content.slice(insertPos);
}

fs.writeFileSync(GENERATED_PATH, content);
console.log("Post-processed generated types:", GENERATED_PATH);
