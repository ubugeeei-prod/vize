import { parseSync } from "oxc-parser";

const OUTPUT_PARSE_ID = "vize-output.tsx";
const SFC_MAIN_NAME = "_sfc_main";
const EXPORT_DEFAULT = "export default";

type AstNode = {
  type?: string;
  start?: number;
  end?: number;
  [key: string]: unknown;
};

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

/** oxc and string indices count UTF-16 units; the native shape counts UTF-8 bytes. */
function byteOffset(code: string, index: number | null): number | null {
  return index == null ? null : Buffer.byteLength(code.slice(0, index));
}

function isNode(value: unknown): value is AstNode {
  return value != null && typeof value === "object" && typeof (value as AstNode).type === "string";
}

function getNodeStart(node: AstNode | null | undefined): number | null {
  return typeof node?.start === "number" ? node.start : null;
}

function getNodeEnd(node: AstNode | null | undefined): number | null {
  return typeof node?.end === "number" ? node.end : null;
}

function getNodeName(node: AstNode | null | undefined): string | null {
  return isNode(node) && typeof node.name === "string" ? node.name : null;
}

function isSfcMainDefaultExport(defaultExport: AstNode | null | undefined): boolean {
  const declaration = isNode(defaultExport?.declaration) ? defaultExport.declaration : null;
  return isIdentifierNamed(declaration, SFC_MAIN_NAME);
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

function analyzeFastDefaultOutput(code: string): ModuleOutputInfo | null {
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
    // The fast path is only taken when `_sfc_main` does not occur anywhere in
    // the module, so the default export cannot be that identifier and no caller
    // needs the statement's end offset.
    defaultExportEnd: null,
    defaultExportIsSfcMain: false,
  };
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
  const fastOutput = analyzeFastDefaultOutput(code);
  if (fastOutput) {
    return fastOutput;
  }

  const program = parseProgram(code);
  const body = getProgramBody(program);
  const defaultExport = findDefaultExport(program);
  const defaultExportStart = getNodeStart(defaultExport);
  const defaultExportKeywordEnd = defaultExport
    ? getExportDefaultKeywordEnd(code, defaultExport)
    : null;
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
    defaultExportKeywordEnd: byteOffset(code, defaultExportKeywordEnd),
    defaultExportStart: byteOffset(code, defaultExportStart),
    defaultExportEnd: byteOffset(code, getNodeEnd(defaultExport)),
    defaultExportIsSfcMain: isSfcMainDefaultExport(defaultExport),
  };
}
