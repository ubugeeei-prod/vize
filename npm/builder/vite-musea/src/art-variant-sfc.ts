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

import { assertStaticExpression, loadNative } from "./native-loader.js";
import { resolveArtComponent } from "./art-component.js";
import {
  importDeclaresName,
  parseScriptSetupForArt,
  rewriteRelativeImportStatement,
} from "./art-module.js";
import { serializeScriptValue } from "./security.js";
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
    componentImportStatement?: string;
    artFilePath?: string;
    /**
     * Bindings to import from the art file's shared setup module instead of
     * declaring locally. `scriptSetupIsolated: false` means every variant sees
     * one setup instance, which per-variant SFCs would otherwise each
     * re-evaluate — losing the shared state that opt-in exists for.
     */
    sharedBindings?: { moduleId: string; names: string[] };
    /** Typed function expression authored by `<variant :play="…">`. */
    play?: string;
    props?: Record<string, unknown>;
    attrs?: Record<string, unknown>;
  } = {},
): string {
  const playRegistration = createPlayRegistration(options.play);
  const variantContracts = createVariantContracts(
    options.props,
    options.attrs,
    options.componentBindingName,
  );
  const boundTemplate = injectVariantBindings(
    variantTemplate,
    options.componentBindingName,
    Boolean(options.props || options.attrs),
  );
  if (options.sharedBindings && options.sharedBindings.names.length > 0) {
    const { moduleId, names } = options.sharedBindings;
    const escapedShared = variantName.replace(/"/g, "&quot;");
    // The demonstrated component is not necessarily a shared binding: an art
    // file can name it through `defineArt`/`component` without importing it in
    // `<script setup>`. Import it here too, unless the shared module already
    // exports that name, or the authored target tag cannot resolve.
    const sharedComponentImport =
      options.componentBindingName && !names.includes(options.componentBindingName)
        ? componentImportCode(options)
        : "";
    return (
      `<script setup lang="ts">\n` +
      sharedComponentImport +
      `import { ${names.join(", ")} } from ${JSON.stringify(moduleId)}\n` +
      variantContracts +
      playRegistration +
      `</script>\n` +
      `<template><div data-variant="${escapedShared}">${boundTemplate}</div></template>\n`
    );
  }
  const authoredScriptSetup = options.scriptSetup ?? art.scriptSetupContent ?? "";
  const parsedScriptSetup = authoredScriptSetup.trim()
    ? parseScriptSetupForArt(authoredScriptSetup)
    : null;
  const scriptSetup = rebaseScriptSetup(
    authoredScriptSetup,
    path.dirname(options.artFilePath ?? art.path),
  );
  // The demonstrated component is imported into the variant's own setup scope so
  // The authored target tag resolves through the SFC's binding resolution
  // rather than a `components` option.
  const componentAlreadyImported =
    options.componentBindingName &&
    parsedScriptSetup?.imports.some((statement) =>
      importDeclaresName(statement, options.componentBindingName!),
    );
  const componentImport =
    options.componentImportStatement || componentAlreadyImported
      ? ""
      : componentImportCode(options);
  // Always emit the block, even empty. Without a script a template-only SFC
  // compiles to a bare `export function render`, with no default export for the
  // art module to import — the variant would resolve to `undefined`.
  const body =
    `${componentImport}${scriptSetup.trim()}${variantContracts}${playRegistration}`.trim();
  const script = `<script setup lang="ts">\n${body}\n</script>\n`;
  // The wrapper carries the variant name for the gallery's DOM queries. It is
  // part of the template so the compiler sees a single root.
  const escapedName = variantName.replace(/"/g, "&quot;");
  return `${script}<template><div data-variant="${escapedName}">${boundTemplate}</div></template>\n`;
}

function componentImportCode(options: {
  componentImportPath?: string;
  componentBindingName?: string;
  componentImportStatement?: string;
  artFilePath?: string;
}): string {
  if (options.componentImportStatement) {
    return `${rewriteRelativeImportStatement(
      options.componentImportStatement,
      path.dirname(options.artFilePath ?? "."),
    )}\n`;
  }
  return options.componentImportPath && options.componentBindingName
    ? `import ${options.componentBindingName} from ${JSON.stringify(options.componentImportPath)}\n`
    : "";
}

function createPlayRegistration(play: string | undefined): string {
  if (!play) return "";
  const expression = assertStaticExpression(play, "play");
  return `
const __museaVariantPlay: MuseaPlay = (${expression});
if (typeof window !== "undefined") { (window as any).__museaPlay = __museaVariantPlay; }
`;
}

function createVariantContracts(
  props: Record<string, unknown> | undefined,
  attrs: Record<string, unknown> | undefined,
  componentBindingName: string | undefined,
): string {
  const propType = componentBindingName
    ? `MuseaComponentProps<typeof ${componentBindingName}>`
    : "Record<string, unknown>";
  const propsCheck = props
    ? `\nconst __museaVariantProps = ${serializeScriptValue(props)} satisfies Partial<${propType}>;\n`
    : "";
  const attrsCheck = attrs
    ? `\nconst __museaVariantAttrs = ${serializeScriptValue(attrs)} satisfies ${componentBindingName ? `MuseaComponentAttrs<typeof ${componentBindingName}>` : "MuseaAttrs"};\n`
    : "";
  const bindings =
    props || attrs
      ? `\nconst __museaVariantBindings = { ...${attrs ? "__museaVariantAttrs" : "{}"}, ...${props ? "__museaVariantProps" : "{}"} };\n`
      : "";
  return propsCheck + attrsCheck + bindings;
}

function injectVariantBindings(
  template: string,
  componentBindingName: string | undefined,
  hasBindings: boolean,
): string {
  if (!componentBindingName || !hasBindings) return template;
  const escaped = componentBindingName.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
  return template.replace(
    new RegExp(`<${escaped}(?=[\\s/>])`),
    `<${componentBindingName} v-bind="__museaVariantBindings"`,
  );
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
    play?: string;
    props?: Record<string, unknown>;
    attrs?: Record<string, unknown>;
  } = {},
): VariantSfcResult {
  const component = resolveArtComponent(art, filename, options.parsedScriptSetup ?? null, {
    root: options.root,
    scanRoots: options.scanRoots,
  });
  const source = buildVariantSfcSource(art, variantTemplate, variantName, {
    scriptSetup: options.scriptSetup,
    componentImportPath: component.componentImportPath,
    componentBindingName: component.componentBindingName,
    componentImportStatement: component.componentImportStatement,
    artFilePath: filename,
    sharedBindings: options.sharedBindings,
    play: options.play,
    props: options.props,
    attrs: options.attrs,
  });
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
