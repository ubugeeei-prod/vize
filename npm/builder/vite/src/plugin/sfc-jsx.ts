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

// Matches wherever a `jsx` or `tsx` lang attribute could be, and in more places
// besides, so a source it does not match cannot hold a JSX script block.
const JSX_LANG = /\blang\s*=\s*["']?[jt]sx\b/;
const SCRIPT_OPEN = /<script(?=\s)/g;
// One attribute of an opening tag, read as the SFC parser reads it. A quote
// opens a value only right after `=`, and a `>` inside a quoted value does not
// end the tag. A stray `=` before a name is skipped, as the parser skips it.
const TAG_ATTRIBUTE = /[\s=]*([^\s=>/]+)(?:\s*=\s*(?:"([^"]*)"?|'([^']*)'?|([^\s>/]*)))?/y;

/** The `lang` of the opening tag whose attributes start at `from`. */
function openingTagLang(source: string, from: number): string | undefined {
  let lang: string | undefined;
  TAG_ATTRIBUTE.lastIndex = from;
  for (let match = TAG_ATTRIBUTE.exec(source); match; match = TAG_ATTRIBUTE.exec(source)) {
    // The parser keeps the last of repeated attributes.
    if (match[1] === "lang") lang = match[2] ?? match[3] ?? match[4];
  }
  return lang;
}

/** A JSX-bearing script block: its language and the line its tag opens on. */
export interface SfcJsxScript {
  lang: "jsx" | "tsx";
  line: number;
}

/**
 * The SFC's JSX-bearing script block, or `null`. One pattern test is all an
 * SFC without such a block pays; only a source it matches has its script tags
 * read. Vue rejects script blocks of different languages, so the first block
 * found names the language of both.
 */
export function findSfcJsxScript(source: string): SfcJsxScript | null {
  if (!JSX_LANG.test(source)) return null;
  for (const tag of source.matchAll(SCRIPT_OPEN)) {
    const lang = openingTagLang(source, tag.index + "<script".length);
    if (lang === "jsx" || lang === "tsx") {
      return { lang, line: countNewlines(source.slice(0, tag.index)) + 1 };
    }
  }
  return null;
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

  // The binding reports no spans, so point at the block the JSX came from.
  const where = `in <script lang="${script.lang}"> (line ${script.line})`;
  let lowered: ReturnType<typeof compileJsxModule>;
  try {
    // Client builds use the same options, in the same precedence, as a
    // standalone `.tsx` module. The SFC's server render renders VNodes, and
    // Vapor JSX output needs the DOM, so SSR always takes the VDOM output.
    const fileOptions = state.fileCompilerOptions?.(filePath) ?? state.mergedOptions;
    lowered = compileJsxModule(filePath, masked, {
      jsxMode: ssr ? "vdom" : fileOptions.jsxMode,
      jsxCompat: fileOptions.jsxCompat,
      vapor: ssr ? false : (fileOptions.vapor ?? false),
      ssr,
      lang: script.lang,
      sourceMap: emitted.map !== null,
    });
  } catch (error) {
    if (!(error instanceof VizeSfcCompileError)) throw error;
    throw new VizeSfcCompileError(
      filePath,
      error.diagnostics.map((diagnostic) => `${diagnostic} ${where}`),
    );
  }

  // What remains of this pattern is the JSX compiler's helper preamble.
  const imported = new Map<string, string>();
  for (const [, list] of lowered.code.matchAll(HELPER_IMPORT)) {
    for (const [specifier, local] of helperSpecifiers(list) ?? []) imported.set(local, specifier);
  }
  // A same-line substitution, so the lowered map still holds.
  const code = lowered.code.replace(HELPER_PLACEHOLDER, (_, index: string) => {
    const kept: string[] = [];
    for (const [specifier, local] of setAside[Number(index)] ?? []) {
      const preamble = imported.get(local);
      if (preamble === undefined) kept.push(specifier);
      // An authored import can share a helper's local name, such as
      // `ref as _createElementVNode`. Dropping it would rebind every authored
      // use to the helper, so only the identical import is merged.
      else if (preamble !== specifier) {
        throw new VizeSfcCompileError(filePath, [
          `\`${specifier}\` from "vue" binds \`${local}\`, which JSX lowering imports as \`${preamble}\`; rename the local ${where}`,
        ]);
      }
    }
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
