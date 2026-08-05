/**
 * One shared `<script setup>` instance for an art file's variants.
 *
 * `scriptSetupIsolated: false` opts an art file out of per-variant isolation:
 * every variant sees the same setup instance, so state written in one variant is
 * visible in the next. Compiling each variant as its own SFC (#3857) would give
 * each one its own setup and silently drop that, so the shared case hoists the
 * setup into this module and the variants import its bindings instead of
 * declaring them.
 *
 * A dedicated module rather than the art module itself, because the art module
 * imports the variants — routing the shared state through it would make that a
 * cycle.
 */

import path from "node:path";

import { parseScriptSetupForArt, rewriteRelativeImportStatement } from "./art-module.js";
import type { ArtFileInfo } from "./types/index.js";

type ParsedScriptSetup = {
  imports: string[];
  setupBody: string[];
  returnNames: string[];
};

/**
 * The binding names a variant SFC should import from the shared module.
 *
 * `<script setup>` resolves template identifiers from its own bindings, and an
 * import is a binding, so importing each name by hand is what makes the shared
 * state reachable from the variant's template.
 */
export function sharedBindingNames(parsed: ParsedScriptSetup): string[] {
  return [...new Set(parsed.returnNames)].filter((name) => /^[A-Za-z_$][\w$]*$/.test(name));
}

export function generateSharedSetupModule(art: ArtFileInfo, filePath: string): string {
  const parsed = parseSharedScriptSetup(art);
  if (!parsed) return "export {};\n";

  const artDir = path.dirname(filePath);
  const imports = parsed.imports
    .map((statement) => rewriteRelativeImportStatement(statement, artDir))
    .join("\n");
  const names = sharedBindingNames(parsed);

  // Evaluated once at module scope, so every variant importing these bindings
  // observes the same instance.
  return [
    imports,
    parsed.setupBody.join("\n"),
    names.length > 0 ? `export { ${names.join(", ")} };` : "export {};",
    "",
  ].join("\n");
}

function parseSharedScriptSetup(art: ArtFileInfo): ParsedScriptSetup | null {
  if (!art.scriptSetupContent) return null;
  return parseScriptSetupForArt(art.scriptSetupContent);
}
