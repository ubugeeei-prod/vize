/**
 * JSX in an SFC's `<script lang="jsx">` or `<script lang="tsx">` block.
 *
 * The SFC compiler keeps that JSX in its output, as @vue/compiler-sfc does.
 * @vitejs/plugin-vue then serves the script as its own `lang.tsx` submodule
 * for @vitejs/plugin-vue-jsx to lower, but Vize emits the whole SFC as one
 * module, so nothing downstream would lower it. The compiled module is lowered
 * here instead, with the options standalone `.jsx`/`.tsx` modules use.
 */

import { countNewlines } from "../../../shared/source-map.ts";
import { compileJsxModule, VizeSfcCompileError } from "../compiler.ts";
import { composeSourceMaps, parseSourceMap, type SourceMapV3 } from "../utils/source-map.ts";
import type { VizePluginState } from "./state.ts";

const JSX_SCRIPT = /<script\b[^>]*?\slang\s*=\s*["']?([jt]sx)\b/;

/** A JSX-bearing script block: its language and the line its tag opens on. */
export interface SfcJsxScript {
  lang: "jsx" | "tsx";
  line: number;
}

/**
 * The SFC's JSX-bearing script block, or `null`. This one pattern test is all
 * an SFC without such a block pays. Vue rejects script blocks of different
 * languages, so the first match names the language of both.
 */
export function findSfcJsxScript(source: string): SfcJsxScript | null {
  const match = JSX_SCRIPT.exec(source);
  if (!match) return null;
  return {
    lang: match[1] as SfcJsxScript["lang"],
    line: countNewlines(source.slice(0, match.index)) + 1,
  };
}

// The SFC compiler's own runtime-helper imports. The JSX compiler rejects any
// authored binding that is spelled like one of its helpers, and these are the
// same `vue` exports under the same names, so they are set aside while the
// JSX is lowered and merged with the JSX helper imports afterwards.
const HELPER_IMPORT = /^import \{ ([^{}\n]+) \} from (["'])vue\2;?[ \t]*$/gm;
const HELPER_SPECIFIER = /^[\w$]+ as (_[\w$]+)$/;
const HELPER_PLACEHOLDER = /\/\*vize-sfc-helpers:(\d+)\*\//g;

/** Each specifier with its local name, or `null` when one is not a helper. */
function helperSpecifiers(list: string): [specifier: string, local: string][] | null {
  const specifiers: [string, string][] = [];
  for (const specifier of list.split(", ")) {
    const local = HELPER_SPECIFIER.exec(specifier)?.[1];
    if (local === undefined) return null;
    specifiers.push([specifier, local]);
  }
  return specifiers;
}

/**
 * Lower the JSX in an assembled SFC module, keeping its map pointed at the
 * `.vue` file. JSX diagnostics fail the compile like SFC diagnostics do.
 */
export function lowerSfcJsx(
  state: Pick<VizePluginState, "fileCompilerOptions" | "mergedOptions" | "logger">,
  filePath: string,
  script: SfcJsxScript,
  emitted: { code: string; map: SourceMapV3 | null },
  { ssr, logWarnings = true }: { ssr: boolean; logWarnings?: boolean },
): { code: string; map: SourceMapV3 | null } {
  const setAside: [string, string][][] = [];
  // Each helper import becomes a placeholder on its own line, so the lines the
  // SFC map describes stay where they are.
  const masked = emitted.code.replace(HELPER_IMPORT, (line, list: string) => {
    const specifiers = helperSpecifiers(list);
    if (!specifiers) return line;
    setAside.push(specifiers);
    return `/*vize-sfc-helpers:${setAside.length - 1}*/`;
  });

  let lowered: ReturnType<typeof compileJsxModule>;
  try {
    // The same options, in the same precedence, as a standalone `.tsx` module.
    const fileOptions = state.fileCompilerOptions?.(filePath) ?? state.mergedOptions;
    lowered = compileJsxModule(filePath, masked, {
      jsxMode: fileOptions.jsxMode,
      jsxCompat: fileOptions.jsxCompat,
      vapor: fileOptions.vapor ?? false,
      ssr,
      lang: script.lang,
      sourceMap: emitted.map !== null,
    });
  } catch (error) {
    if (!(error instanceof VizeSfcCompileError)) throw error;
    // The binding reports no spans, so point at the block the JSX came from.
    const where = `in <script lang="${script.lang}"> (line ${script.line})`;
    throw new VizeSfcCompileError(
      filePath,
      error.diagnostics.map((diagnostic) => `${diagnostic} ${where}`),
    );
  }

  // What remains of this pattern is the JSX compiler's helper preamble.
  const imported = new Set<string>();
  for (const [, list] of lowered.code.matchAll(HELPER_IMPORT)) {
    for (const [, local] of helperSpecifiers(list) ?? []) imported.add(local);
  }
  // A same-line substitution, so the lowered map still holds.
  const code = lowered.code.replace(HELPER_PLACEHOLDER, (_, index: string) => {
    const kept = (setAside[Number(index)] ?? [])
      .filter(([, local]) => !imported.has(local))
      .map(([specifier]) => specifier);
    return kept.length > 0 ? `import { ${kept.join(", ")} } from "vue"` : "";
  });

  if (logWarnings) {
    for (const warning of lowered.warnings) state.logger.warn(`Warning in ${filePath}: ${warning}`);
  }
  const loweredMap = parseSourceMap(lowered.map);
  return {
    code,
    map: emitted.map && loweredMap ? composeSourceMaps(loweredMap, emitted.map) : null,
  };
}
