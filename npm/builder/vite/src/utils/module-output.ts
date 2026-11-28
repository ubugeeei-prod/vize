const SFC_MAIN_NAME = "_sfc_main";
const EXPORT_DEFAULT = "export default";

/** The native `moduleShape`: offsets are UTF-8 bytes into the module. */
export type ModuleOutputInfo = {
  hasDefaultExport: boolean;
  hasSfcMainDefined: boolean;
  hasNamedRenderExport: boolean;
  hasNamedSsrRenderExport: boolean;
  defaultExportKeywordEnd: number | null;
  defaultExportStart: number | null;
  /** End offset of the whole `export default ...` statement, or `null`. */
  defaultExportEnd: number | null;
  /** Whether the default export's declaration is exactly the `_sfc_main` identifier. */
  defaultExportIsSfcMain: boolean;
};

const NO_EXPORTS: ModuleOutputInfo = {
  hasDefaultExport: false,
  hasSfcMainDefined: false,
  hasNamedRenderExport: false,
  hasNamedSsrRenderExport: false,
  defaultExportKeywordEnd: null,
  defaultExportStart: null,
  defaultExportEnd: null,
  defaultExportIsSfcMain: false,
};

/** String indices count UTF-16 units; the native shape counts UTF-8 bytes. */
function byteOffset(code: string, index: number): number {
  return Buffer.byteLength(code.slice(0, index));
}

/**
 * Find a lone `export default` by scanning. Only a module without `_sfc_main`,
 * a named render export or an `export {` list qualifies, so the match cannot
 * be a component the module assembles some other way.
 */
export function analyzeFastDefaultOutput(code: string): ModuleOutputInfo | null {
  if (
    code.includes(SFC_MAIN_NAME) ||
    code.includes("export function render") ||
    code.includes("export function ssrRender") ||
    code.includes("export {")
  ) {
    return null;
  }

  const defaultExportStart = code.indexOf(EXPORT_DEFAULT);
  if (defaultExportStart === -1 || defaultExportStart !== code.lastIndexOf(EXPORT_DEFAULT)) {
    return null;
  }

  const before = defaultExportStart === 0 ? "\n" : code[defaultExportStart - 1];
  if (!before || !/\s|;/.test(before)) {
    return null;
  }

  return {
    hasDefaultExport: true,
    hasSfcMainDefined: false,
    hasNamedRenderExport: false,
    hasNamedSsrRenderExport: false,
    defaultExportStart: byteOffset(code, defaultExportStart),
    defaultExportKeywordEnd: byteOffset(code, defaultExportStart + EXPORT_DEFAULT.length),
    // The scan only matches when `_sfc_main` does not occur anywhere in the
    // module, so the default export cannot be that identifier and no caller
    // needs the statement's end offset.
    defaultExportEnd: null,
    defaultExportIsSfcMain: false,
  };
}

/**
 * The shape of a module the native compiler reported none for.
 *
 * The native compiler parses every module it emits as TypeScript, then as TSX,
 * and reports no shape only when both fail. Nothing here could parse that
 * module either, so only the scan's unambiguous `export default` is bound and
 * any other module is left as it is.
 */
export function unparsedModuleOutput(code: string): ModuleOutputInfo {
  return analyzeFastDefaultOutput(code) ?? { ...NO_EXPORTS };
}
