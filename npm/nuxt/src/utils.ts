import type { VizeNuxtCompilerOptions } from "./compiler-options.ts";
import { createHash } from "node:crypto";
import fs from "node:fs";

export const NUXT_OG_IMAGE_RENDERER_SFC_EXCLUDE = /\.takumi\.vue(?:\?|$)/;

function normalizeUrlPrefix(value: string): string {
  const withLeadingSlash = value.startsWith("/") ? value : `/${value}`;
  return withLeadingSlash.endsWith("/") ? withLeadingSlash : `${withLeadingSlash}/`;
}

export function buildNuxtDevAssetBase(baseURL = "/", buildAssetsDir = "/_nuxt/"): string {
  const normalizedBase = normalizeUrlPrefix(baseURL);
  const normalizedAssetsDir = normalizeUrlPrefix(buildAssetsDir);
  return normalizedBase === "/"
    ? normalizedAssetsDir
    : normalizeUrlPrefix(`${normalizedBase}${normalizedAssetsDir.replace(/^\//, "")}`);
}

export function buildNuxtCompilerOptions(
  rootDir: string,
  baseURL = "/",
  buildAssetsDir = "/_nuxt/",
  overrides: VizeNuxtCompilerOptions = {},
): VizeNuxtCompilerOptions {
  const defaults: VizeNuxtCompilerOptions = {
    devUrlBase: buildNuxtDevAssetBase(baseURL, buildAssetsDir),
    root: rootDir,
    scanPatterns: [],
  };

  if (overrides.customRenderer === true && overrides.exclude !== undefined) {
    defaults.exclude = overrides.exclude;
  } else if (overrides.customRenderer !== true) {
    defaults.exclude = mergeNuxtCompilerPatterns(
      NUXT_OG_IMAGE_RENDERER_SFC_EXCLUDE,
      overrides.exclude,
    );
  }

  for (const [key, value] of Object.entries(overrides) as Array<
    [keyof VizeNuxtCompilerOptions, VizeNuxtCompilerOptions[keyof VizeNuxtCompilerOptions]]
  >) {
    if (key === "exclude") {
      continue;
    }
    if (value !== undefined) {
      defaults[key] = value as never;
    }
  }

  return defaults;
}

function mergeNuxtCompilerPatterns(
  defaultPattern: NonNullable<VizeNuxtCompilerOptions["exclude"]>,
  userPattern: VizeNuxtCompilerOptions["exclude"],
): VizeNuxtCompilerOptions["exclude"] {
  if (userPattern == null) {
    return defaultPattern;
  }

  return [defaultPattern, ...(Array.isArray(userPattern) ? userPattern : [userPattern])];
}

export function isVizeVirtualVueModuleId(id: string): boolean {
  return id.startsWith("\0") && /\.vue\.ts(?:\?|$)/.test(id);
}

export function isVizeGeneratedVueModuleId(id: string): boolean {
  let normalized = id;
  if (normalized.startsWith("/@id/__x00__")) {
    normalized = normalized.slice("/@id/__x00__".length);
  } else if (normalized.startsWith("__x00__")) {
    normalized = normalized.slice("__x00__".length);
  }
  return /\.vue\.ts(?:\?|$)/.test(normalized);
}

/**
 * Recognize raw `.jsx`/`.tsx` Vue component modules compiled by Vize.
 *
 * Unlike `.vue` files, JSX/TSX modules are transformed in place by the
 * underlying Vite plugin (the original `.jsx`/`.tsx` id is preserved, no
 * `\0`-prefixed `.vue.ts` virtual id is created). Nuxt's auto-import,
 * component, and i18n transforms still need to run on these modules, so the
 * Nuxt transform bridge keys off this predicate in addition to
 * `isVizeGeneratedVueModuleId`.
 *
 * A bare query suffix (e.g. `?vue`) is ignored so dev-server requests still
 * match, but `?raw`/`?url`/`?worker` asset imports are rejected since those
 * are not compiled component modules.
 */
export function isVizeJsxModuleId(id: string): boolean {
  const queryIndex = id.indexOf("?");
  const pathPart = queryIndex === -1 ? id : id.slice(0, queryIndex);
  if (!/\.(?:jsx|tsx)$/.test(pathPart)) {
    return false;
  }

  if (queryIndex === -1) {
    return true;
  }

  const params = new URLSearchParams(id.slice(queryIndex + 1));
  return !(
    params.has("raw") ||
    params.has("url") ||
    params.has("worker") ||
    params.has("sharedworker")
  );
}

export function normalizeVizeVirtualVueModuleId(id: string): string {
  const withoutPrefix = id.startsWith("\0vize-ssr:") ? id.slice("\0vize-ssr:".length) : id.slice(1);
  return withoutPrefix.replace(/\.ts(?=\?|$)/, "");
}

export function normalizeVizeGeneratedVueModuleId(id: string): string {
  if (isVizeVirtualVueModuleId(id)) {
    return normalizeVizeVirtualVueModuleId(id);
  }

  return id
    .replace(/^\/@id\/__x00__/, "")
    .replace(/^__x00__/, "")
    .replace(/\.ts(?=\?|$)/, "");
}

const NUXT_INJECTED_MARKER = "/* nuxt-injected */";
const NUXT_INJECTED_KEY_RE = /'\$[^']+'\s+\/\* nuxt-injected \*\//g;
const NUXT_FETCH_COMPOSABLE_RE = /\b(?:useFetch|useLazyFetch)\s*\(/g;

function buildStableNuxtKey(id: string, index: number): string {
  return createHash("sha256")
    .update(id)
    .update(":")
    .update(String(index))
    .digest("base64url")
    .slice(0, 10);
}

export function normalizeNuxtInjectedKeysForVizeVirtualModule(code: string, id: string): string {
  const normalizedId = normalizeVizeGeneratedVueModuleId(id).replace(/\?.*$/, "");
  let index = 0;
  return code.replace(NUXT_INJECTED_KEY_RE, () => {
    index += 1;
    return `'$${buildStableNuxtKey(normalizedId, index)}' ${NUXT_INJECTED_MARKER}`;
  });
}

export function stabilizeNuxtInjectedKeysForVizeVirtualModule(code: string, id: string): string {
  return normalizeNuxtInjectedKeysForVizeVirtualModule(injectMissingNuxtFetchKeys(code), id);
}

function injectMissingNuxtFetchKeys(code: string): string {
  let output = "";
  let cursor = 0;

  for (const match of code.matchAll(NUXT_FETCH_COMPOSABLE_RE)) {
    const matchIndex = match.index ?? 0;
    const openParenIndex = matchIndex + match[0].length - 1;
    if (openParenIndex < cursor) {
      continue;
    }

    const closeParenIndex = findMatchingParen(code, openParenIndex);
    if (closeParenIndex === -1) {
      continue;
    }

    const args = code.slice(openParenIndex + 1, closeParenIndex);
    if (args.includes(NUXT_INJECTED_MARKER)) {
      continue;
    }

    output += code.slice(cursor, closeParenIndex);
    output += `${args.trim().length === 0 ? "" : ", "}'$__vize_nuxt_key__' ${NUXT_INJECTED_MARKER}`;
    cursor = closeParenIndex;
  }

  return cursor === 0 ? code : output + code.slice(cursor);
}

function findMatchingParen(code: string, openParenIndex: number): number {
  let depth = 0;
  let quote: "'" | '"' | "`" | null = null;
  let escaped = false;
  let lineComment = false;
  let blockComment = false;

  for (let index = openParenIndex; index < code.length; index += 1) {
    const char = code[index]!;
    const next = code[index + 1];

    if (lineComment) {
      if (char === "\n" || char === "\r") {
        lineComment = false;
      }
      continue;
    }

    if (blockComment) {
      if (char === "*" && next === "/") {
        blockComment = false;
        index += 1;
      }
      continue;
    }

    if (quote) {
      if (escaped) {
        escaped = false;
      } else if (char === "\\") {
        escaped = true;
      } else if (char === quote) {
        quote = null;
      }
      continue;
    }

    if (char === "/" && next === "/") {
      lineComment = true;
      index += 1;
      continue;
    }
    if (char === "/" && next === "*") {
      blockComment = true;
      index += 1;
      continue;
    }
    if (char === "'" || char === '"' || char === "`") {
      quote = char;
      continue;
    }
    if (char === "(") {
      depth += 1;
      continue;
    }
    if (char === ")") {
      depth -= 1;
      if (depth === 0) {
        return index;
      }
    }
  }

  return -1;
}

type NamedImportSpecifier = {
  imported: string;
  local: string;
  raw: string;
};

type NamedImportStatement = {
  end: number;
  quote: string;
  source: string;
  specifiers: NamedImportSpecifier[];
  start: number;
};

const NAMED_IMPORT_RE = /^import\s*\{([\s\S]*?)\}\s*from\s*(['"])(vue|#imports|#entry)\2\s*;?/gm;

function parseNamedImportSpecifiers(specifierSource: string): NamedImportSpecifier[] {
  return specifierSource
    .split(",")
    .map((specifier) => specifier.trim())
    .filter(Boolean)
    .flatMap((specifier) => {
      const withoutType = specifier.replace(/^type\s+/, "").trim();
      if (!withoutType) {
        return [];
      }
      const match = withoutType.match(/^([A-Za-z_$][\w$]*)(?:\s+as\s+([A-Za-z_$][\w$]*))?$/);
      if (!match) {
        return [];
      }
      const imported = match[1]!;
      const local = match[2] ?? imported;
      return [{ imported, local, raw: withoutType }];
    });
}

function collectNamedImports(code: string): NamedImportStatement[] {
  const imports: NamedImportStatement[] = [];
  for (const match of code.matchAll(NAMED_IMPORT_RE)) {
    imports.push({
      start: match.index ?? 0,
      end: (match.index ?? 0) + match[0].length,
      quote: match[2]!,
      source: match[3]!,
      specifiers: parseNamedImportSpecifiers(match[1] ?? ""),
    });
  }
  return imports;
}

function renderNamedImport(
  specifiers: NamedImportSpecifier[],
  source: string,
  quote: string,
): string {
  return `import { ${specifiers.map((specifier) => specifier.raw).join(", ")} } from ${quote}${source}${quote};`;
}

export function preserveExplicitVueImportsFromNuxtAutoImports(
  originalCode: string,
  injectedCode: string,
): string {
  const originalVueSpecifiers = new Map<string, NamedImportSpecifier>();
  for (const statement of collectNamedImports(originalCode)) {
    if (statement.source !== "vue") {
      continue;
    }
    for (const specifier of statement.specifiers) {
      originalVueSpecifiers.set(specifier.local, specifier);
    }
  }

  if (originalVueSpecifiers.size === 0) {
    return injectedCode;
  }

  const restoredSpecifiers = new Map<string, NamedImportSpecifier>();
  const replacements: Array<{ start: number; end: number; text: string }> = [];

  for (const statement of collectNamedImports(injectedCode)) {
    if (statement.source !== "#imports" && statement.source !== "#entry") {
      continue;
    }

    const keep: NamedImportSpecifier[] = [];
    let changed = false;
    for (const specifier of statement.specifiers) {
      const original = originalVueSpecifiers.get(specifier.local);
      if (original) {
        restoredSpecifiers.set(specifier.local, original);
        changed = true;
      } else {
        keep.push(specifier);
      }
    }

    if (!changed) {
      continue;
    }

    replacements.push({
      start: statement.start,
      end: statement.end,
      text: keep.length > 0 ? renderNamedImport(keep, statement.source, statement.quote) : "",
    });
  }

  if (replacements.length === 0) {
    return injectedCode;
  }

  let output = injectedCode;
  for (const replacement of replacements.reverse()) {
    output = output.slice(0, replacement.start) + replacement.text + output.slice(replacement.end);
  }

  const currentVueLocals = new Set<string>();
  for (const statement of collectNamedImports(output)) {
    if (statement.source !== "vue") {
      continue;
    }
    for (const specifier of statement.specifiers) {
      currentVueLocals.add(specifier.local);
    }
  }

  const missing = [...restoredSpecifiers.values()].filter(
    (specifier) => !currentVueLocals.has(specifier.local),
  );
  if (missing.length > 0) {
    output = `${renderNamedImport(missing, "vue", '"')}\n${output}`;
  }

  return output.replace(/\n{3,}/g, "\n\n");
}

export function preserveExplicitVueImportsFromVizeModuleSource(id: string, code: string): string {
  if (!isVizeVirtualVueModuleId(id) && !isVizeGeneratedVueModuleId(id)) {
    return code;
  }

  const normalizedId = normalizeVizeGeneratedVueModuleId(id);
  const sourcePath = normalizedId.replace(/\?.*$/, "");
  if (!sourcePath.endsWith(".vue") || !fs.existsSync(sourcePath)) {
    return code;
  }

  return preserveExplicitVueImportsFromNuxtAutoImports(fs.readFileSync(sourcePath, "utf-8"), code);
}
