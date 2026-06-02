import type { VizeNuxtCompilerOptions } from "./compiler-options.ts";
import { createHash } from "node:crypto";
import fs from "node:fs";

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

  for (const [key, value] of Object.entries(overrides) as Array<
    [keyof VizeNuxtCompilerOptions, VizeNuxtCompilerOptions[keyof VizeNuxtCompilerOptions]]
  >) {
    if (value !== undefined) {
      defaults[key] = value as never;
    }
  }

  return defaults;
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

export function normalizeVizeVirtualVueModuleId(id: string): string {
  const withoutPrefix = id.startsWith("\0vize-ssr:") ? id.slice("\0vize-ssr:".length) : id.slice(1);
  return withoutPrefix.replace(/\.ts(?=\?|$)/, "");
}

const NUXT_INJECTED_MARKER = "/* nuxt-injected */";
const NUXT_INJECTED_KEY_RE = /'\$[^']+'\s+\/\* nuxt-injected \*\//g;

function buildStableNuxtKey(id: string, index: number): string {
  return createHash("sha256")
    .update(id)
    .update(":")
    .update(String(index))
    .digest("base64url")
    .slice(0, 10);
}

export function normalizeNuxtInjectedKeysForVizeVirtualModule(code: string, id: string): string {
  const normalizedId = normalizeVizeVirtualVueModuleId(id);
  let index = 0;
  return code.replace(NUXT_INJECTED_KEY_RE, () => {
    index += 1;
    return `'$${buildStableNuxtKey(normalizedId, index)}' ${NUXT_INJECTED_MARKER}`;
  });
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

  const normalizedId = isVizeVirtualVueModuleId(id)
    ? normalizeVizeVirtualVueModuleId(id)
    : id
        .replace(/^\/@id\/__x00__/, "")
        .replace(/^__x00__/, "")
        .replace(/\.ts(?=\?|$)/, "");
  const sourcePath = normalizedId.replace(/\?.*$/, "");
  if (!sourcePath.endsWith(".vue") || !fs.existsSync(sourcePath)) {
    return code;
  }

  return preserveExplicitVueImportsFromNuxtAutoImports(fs.readFileSync(sourcePath, "utf-8"), code);
}
