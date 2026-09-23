import { parseSync } from "oxc-parser";
import { transformSync } from "oxc-transform";
import type { MappedModule } from "./source-map.ts";

const OUTPUT_PARSE_ID = "vize-rspack-output.tsx";
const SFC_MAIN_NAME = "_sfc_main";

type AstNode = {
  type?: string;
  start?: number;
  end?: number;
  [key: string]: unknown;
};

export type ModuleOutputInfo = {
  hasDefaultExport: boolean;
  hasSfcMainDefined: boolean;
};

function isNode(value: unknown): value is AstNode {
  return value != null && typeof value === "object" && typeof (value as AstNode).type === "string";
}

function getNodeStart(node: AstNode | null | undefined): number | null {
  return typeof node?.start === "number" ? node.start : null;
}

function getNodeName(node: AstNode | null | undefined): string | null {
  return isNode(node) && typeof node.name === "string" ? node.name : null;
}

function parseProgram(code: string): AstNode | null {
  try {
    const result = parseSync(OUTPUT_PARSE_ID, code) as unknown;
    if (result != null && typeof result === "object") {
      const errors = (result as { errors?: unknown }).errors;
      if (Array.isArray(errors) && errors.length > 0) {
        return null;
      }

      const program = (result as { program?: unknown }).program;
      if (isNode(program)) {
        return program;
      }
    }

    return isNode(result) ? result : null;
  } catch {
    return null;
  }
}

function getProgramBody(program: AstNode | null): AstNode[] {
  if (!program || !Array.isArray(program.body)) {
    return [];
  }

  return program.body.filter(isNode);
}

function isIdentifierNamed(node: AstNode | null | undefined, name: string): boolean {
  return getNodeName(node) === name;
}

function getVariableDeclarationNames(statement: AstNode): string[] {
  const declarations = Array.isArray(statement.declarations) ? statement.declarations : [];
  return declarations
    .filter(isNode)
    .map((declaration) => (isNode(declaration.id) ? getNodeName(declaration.id) : null))
    .filter((name): name is string => name != null);
}

function findDefaultExport(program: AstNode | null): AstNode | null {
  return (
    getProgramBody(program).find((statement) => statement.type === "ExportDefaultDeclaration") ??
    null
  );
}

function getExportDefaultKeywordEnd(code: string, defaultExport: AstNode): number | null {
  const exportStart = getNodeStart(defaultExport);
  if (exportStart == null) {
    return null;
  }

  const match = /^export\s+default\b/.exec(code.slice(exportStart));
  return match ? exportStart + match[0].length : null;
}

export function analyzeModuleOutput(code: string): ModuleOutputInfo {
  const program = parseProgram(code);
  const body = getProgramBody(program);
  const defaultExport = findDefaultExport(program);

  return {
    hasDefaultExport: defaultExport != null,
    hasSfcMainDefined: body.some((statement) => {
      return (
        statement.type === "VariableDeclaration" &&
        getVariableDeclarationNames(statement).includes(SFC_MAIN_NAME)
      );
    }),
  };
}

/** Snapshot imported values, not live namespace objects, before a hot replacement. */
export function hmrImportSnapshot(code: string): string | null {
  // Analyze a type-erased copy only. Injecting references to a type-only import
  // would otherwise turn it into a runtime dependency in the downstream SWC pass.
  // The actual emitted code and its source map are left untouched.
  const transformed = transformSync(OUTPUT_PARSE_ID, code, { target: "esnext", jsx: "preserve" });
  if (transformed.errors.length) return null;
  const program = parseProgram(transformed.code);
  if (!program) return null;
  const entries: string[] = [];
  for (const statement of getProgramBody(program)) {
    if (statement.type !== "ImportDeclaration") continue;
    const source = isNode(statement.source) ? statement.source.value : null;
    if (typeof source !== "string") return null;
    // Vue helpers can change with the template. CSS has its own HMR boundary.
    if (source === "vue" || source.includes("?vue&type=style&")) continue;
    const specifiers = Array.isArray(statement.specifiers)
      ? statement.specifiers.filter(isNode)
      : [];
    // Side-effect imports cannot be compared safely.
    if (specifiers.length === 0) return null;
    for (const specifier of specifiers) {
      const local = isNode(specifier.local) ? getNodeName(specifier.local) : null;
      if (!local) return null;
      const key = JSON.stringify(`${source}:${local}`);
      entries.push(
        specifier.type === "ImportNamespaceSpecifier"
          ? `...Object.keys(${local}).sort().map(key => [${key} + ':' + key, ${local}[key]])`
          : `[${key}, ${local}]`,
      );
    }
  }
  return `[${entries.join(", ")}]`;
}

export function rewriteDefaultExportToSfcMain(module: MappedModule): void {
  const code = module.code;
  const defaultExport = findDefaultExport(parseProgram(code));
  const exportStart = getNodeStart(defaultExport);
  const keywordEnd = defaultExport ? getExportDefaultKeywordEnd(code, defaultExport) : null;
  if (exportStart == null || keywordEnd == null) {
    return;
  }

  module.replace(exportStart, keywordEnd, `const ${SFC_MAIN_NAME} =`);
}

export function insertBeforeSfcMainDefaultExport(
  module: MappedModule,
  insertion: string,
  options: { normalizeSemicolon?: boolean } = {},
): void {
  const code = module.code;
  const defaultExport = findDefaultExport(parseProgram(code));
  const declaration = isNode(defaultExport?.declaration) ? defaultExport.declaration : null;
  const exportStart = getNodeStart(defaultExport);
  const exportEnd = typeof defaultExport?.end === "number" ? defaultExport.end : null;
  if (!isIdentifierNamed(declaration, SFC_MAIN_NAME) || exportStart == null) {
    return;
  }

  if (
    options.normalizeSemicolon &&
    exportEnd != null &&
    code[exportEnd - 1] !== ";" &&
    code[exportEnd] !== ";"
  ) {
    module.replace(exportEnd, exportEnd, ";");
  }

  module.replace(exportStart, exportStart, `${insertion}\n`);
}
