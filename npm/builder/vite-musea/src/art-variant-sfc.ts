/**
 * Compile an `<art>` variant through the SFC pipeline.
 *
 * Variants used to be emitted as `template:` strings for Vue's runtime
 * compiler, which never sees the SFC pipeline. Any template expression relying
 * on SFC-time compilation therefore failed at render time — most importantly
 * TypeScript, even though `.art.vue` files are authored with
 * `<script setup lang="ts">` (#3857).
 *
 * Compiling the raw template with the template compiler is not enough: it
 * leaves TypeScript in place *and* its identifier prefixer gives up on TS
 * syntax, so `items[0]!.isValid` stays unprefixed and throws at runtime. Only
 * the SFC pipeline handles both, which is what the issue asks for — the variant
 * is compiled the same way the enclosing SFC is:
 *
 *     :disabled="items[0]!.isValid"   ->   disabled: $setup.items[0].isValid
 *     @u="(f: File | null) => …"      ->   onU: (f) => $setup.file = f
 *
 * The generated module render function receives setup state through Vue's
 * proxy-unwrapped `$setup` argument, so refs intentionally do not retain an
 * explicit `.value` access in this output mode.
 */

import path from "node:path";

import { loadNative } from "./native-loader.js";
import { expandSelfTag, resolveArtComponent } from "./art-component.js";
import { parseScriptSetupForArt, rewriteRelativeImportStatement } from "./art-module.js";
import type { ArtFileInfo } from "./types/index.js";

/**
 * Rebase the art file's `<script setup>` for a virtual module.
 *
 * A variant compiles into a virtual module, and a relative specifier cannot be
 * resolved from one — the art module rebases its own imports for exactly this
 * reason, and the variant needs the same treatment or every relative import in
 * an art file fails to resolve.
 */
function rebaseScriptSetup(scriptSetup: string, artDir: string): string {
  if (!scriptSetup.trim()) return "";
  const parsed = parseScriptSetupForArt(scriptSetup);
  const imports = parsed.imports.map((statement) =>
    rewriteRelativeImportStatement(statement, artDir),
  );
  return [...imports, ...parsed.setupBody].join("\n").trim();
}

export type VariantSfcResult = {
  code: string;
  errors: string[];
};

/**
 * Build the synthetic SFC source for one variant.
 *
 * The art file's own `<script setup>` becomes the variant's setup block, so
 * bindings resolve exactly as they do in the authored file, and `lang="ts"` is
 * carried over so the pipeline strips TypeScript from template expressions.
 */
export function buildVariantSfcSource(
  art: ArtFileInfo,
  variantTemplate: string,
  variantName: string,
  options: {
    scriptSetup?: string | null;
    componentImportPath?: string;
    componentBindingName?: string;
    artFilePath?: string;
    /**
     * Bindings to import from the art file's shared setup module instead of
     * declaring locally. `scriptSetupIsolated: false` means every variant sees
     * one setup instance, which per-variant SFCs would otherwise each
     * re-evaluate — losing the shared state that opt-in exists for.
     */
    sharedBindings?: { moduleId: string; names: string[] };
  } = {},
): string {
  if (options.sharedBindings && options.sharedBindings.names.length > 0) {
    const { moduleId, names } = options.sharedBindings;
    const escapedShared = variantName.replace(/"/g, "&quot;");
    // The demonstrated component is not necessarily a shared binding: an art
    // file can name it through `defineArt`/`component` without importing it in
    // `<script setup>`. Import it here too, unless the shared module already
    // exports that name, or `<Self>` expands to a tag the variant cannot resolve.
    const sharedComponentImport =
      options.componentImportPath &&
      options.componentBindingName &&
      !names.includes(options.componentBindingName)
        ? `import ${options.componentBindingName} from ${JSON.stringify(options.componentImportPath)}\n`
        : "";
    return (
      `<script setup lang="ts">\n` +
      sharedComponentImport +
      `import { ${names.join(", ")} } from ${JSON.stringify(moduleId)}\n` +
      `</script>\n` +
      `<template><div data-variant="${escapedShared}">${variantTemplate}</div></template>\n`
    );
  }
  const scriptSetup = rebaseScriptSetup(
    options.scriptSetup ?? art.scriptSetupContent ?? "",
    path.dirname(options.artFilePath ?? art.path),
  );
  // The demonstrated component is imported into the variant's own setup scope so
  // `<Self>` — and any tag the art file referenced — resolves through the SFC's
  // binding resolution rather than a `components` option.
  const componentImport =
    options.componentImportPath && options.componentBindingName
      ? `import ${options.componentBindingName} from ${JSON.stringify(options.componentImportPath)}\n`
      : "";
  // Always emit the block, even empty. Without a script a template-only SFC
  // compiles to a bare `export function render`, with no default export for the
  // art module to import — the variant would resolve to `undefined`.
  const body = `${componentImport}${scriptSetup.trim()}`.trim();
  const script = `<script setup lang="ts">\n${body}\n</script>\n`;
  // The wrapper carries the variant name for the gallery's DOM queries. It is
  // part of the template so the compiler sees a single root.
  const escapedName = variantName.replace(/"/g, "&quot;");
  return `${script}<template><div data-variant="${escapedName}">${variantTemplate}</div></template>\n`;
}

/**
 * Compile a variant to a self-contained ES module exporting the component.
 *
 * Each variant compiles to its own module because `compileSfc` emits a complete
 * module with its own `export default` and imports; concatenating several into
 * one art module would redeclare bindings and collide on the default export.
 */
export function compileVariantSfc(
  art: ArtFileInfo,
  variantTemplate: string,
  variantName: string,
  filename: string,
  options: {
    scriptSetup?: string | null;
    parsedScriptSetup?: Parameters<typeof resolveArtComponent>[2];
    root?: string;
    scanRoots?: string[];
    sharedBindings?: { moduleId: string; names: string[] };
  } = {},
): VariantSfcResult {
  const component = resolveArtComponent(art, filename, options.parsedScriptSetup ?? null, {
    root: options.root,
    scanRoots: options.scanRoots,
  });
  const source = buildVariantSfcSource(
    art,
    expandSelfTag(variantTemplate, component.componentTagName),
    variantName,
    {
      scriptSetup: options.scriptSetup,
      componentImportPath: component.componentImportPath,
      componentBindingName: component.componentBindingName,
      artFilePath: filename,
      sharedBindings: options.sharedBindings,
    },
  );
  const result = loadNative().compileSfc(source, { filename });
  return {
    code: result.code ?? "",
    errors: (result.errors ?? []).map((error) =>
      typeof error === "string"
        ? error
        : ((error as { message?: string }).message ?? JSON.stringify(error)),
    ),
  };
}
