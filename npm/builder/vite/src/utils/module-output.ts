import { parseSync } from "vite";

const OUTPUT_PARSE_ID = "vize-output.tsx";
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
  hasNamedRenderExport: boolean;
  hasNamedSsrRenderExport: boolean;
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

function getExportedNames(statement: AstNode): string[] {
  const declaration = isNode(statement.declaration) ? statement.declaration : null;
  const names: string[] = [];

  if (declaration?.type === "FunctionDeclaration" || declaration?.type === "ClassDeclaration") {
    const name = isNode(declaration.id) ? getNodeName(declaration.id) : null;
    if (name) names.push(name);
  }

  if (declaration?.type === "VariableDeclaration") {
    names.push(...getVariableDeclarationNames(declaration));
  }

  const specifiers = Array.isArray(statement.specifiers) ? statement.specifiers : [];
  for (const specifier of specifiers) {
    if (!isNode(specifier)) {
      continue;
    }

    const exported = isNode(specifier.exported) ? specifier.exported : null;
    const local = isNode(specifier.local) ? specifier.local : null;
    const name = getNodeName(exported) ?? getNodeName(local);
    if (name) names.push(name);
  }

  return names;
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
  const exportedNames = body
    .filter((statement) => statement.type === "ExportNamedDeclaration")
    .flatMap(getExportedNames);

  return {
    hasDefaultExport: defaultExport != null,
    hasSfcMainDefined: body.some((statement) => {
      return (
        statement.type === "VariableDeclaration" &&
        getVariableDeclarationNames(statement).includes(SFC_MAIN_NAME)
      );
    }),
    hasNamedRenderExport: exportedNames.includes("render"),
    hasNamedSsrRenderExport: exportedNames.includes("ssrRender"),
  };
}

export function rewriteDefaultExportToSfcMain(code: string): string {
  const defaultExport = findDefaultExport(parseProgram(code));
  const exportStart = getNodeStart(defaultExport);
  const keywordEnd = defaultExport ? getExportDefaultKeywordEnd(code, defaultExport) : null;
  if (exportStart == null || keywordEnd == null) {
    return code;
  }

  return `${code.slice(0, exportStart)}const ${SFC_MAIN_NAME} =${code.slice(keywordEnd)}`;
}

export function insertBeforeSfcMainDefaultExport(
  code: string,
  insertion: string,
  options: { normalizeSemicolon?: boolean } = {},
): string {
  const defaultExport = findDefaultExport(parseProgram(code));
  const declaration = isNode(defaultExport?.declaration) ? defaultExport.declaration : null;
  const exportStart = getNodeStart(defaultExport);
  const exportEnd = typeof defaultExport?.end === "number" ? defaultExport.end : null;
  if (!isIdentifierNamed(declaration, SFC_MAIN_NAME) || exportStart == null) {
    return code;
  }

  if (options.normalizeSemicolon && exportEnd != null) {
    const suffixStart = code[exportEnd] === ";" ? exportEnd + 1 : exportEnd;
    return `${code.slice(0, exportStart)}${insertion}\nexport default ${SFC_MAIN_NAME};${code.slice(suffixStart)}`;
  }

  return `${code.slice(0, exportStart)}${insertion}\n${code.slice(exportStart)}`;
}
