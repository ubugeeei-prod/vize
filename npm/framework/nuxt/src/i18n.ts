import { parseSync } from "vite";

const I18N_FN_MAP: Record<string, string> = {
  $t: "t: $t",
  $rt: "rt: $rt",
  $d: "d: $d",
  $n: "n: $n",
  $tm: "tm: $tm",
  $te: "te: $te",
};

const I18N_HELPER_LOCALS = new Set(Object.keys(I18N_FN_MAP));
const DEFAULT_PARSE_ID = "vize-nuxt-i18n-bridge.tsx";

type AstNode = {
  type?: string;
  start?: number;
  end?: number;
  [key: string]: unknown;
};

type ExistingUseI18nDestructure = {
  declaration: AstNode;
  pattern: AstNode;
  locals: Set<string>;
};

function getLocalAlias(specifier: string): string {
  const colon = specifier.indexOf(":");
  return (colon === -1 ? specifier : specifier.slice(colon + 1)).trim();
}

function isNode(value: unknown): value is AstNode {
  return value != null && typeof value === "object" && typeof (value as AstNode).type === "string";
}

function getNodeStart(node: AstNode): number | null {
  return typeof node.start === "number" ? node.start : null;
}

function getNodeEnd(node: AstNode): number | null {
  return typeof node.end === "number" ? node.end : null;
}

function getNodeName(node: AstNode | null | undefined): string | null {
  return isNode(node) && typeof node.name === "string" ? node.name : null;
}

function getChildNodes(node: AstNode): AstNode[] {
  const children: AstNode[] = [];

  for (const [key, value] of Object.entries(node)) {
    if (key === "parent") {
      continue;
    }

    if (Array.isArray(value)) {
      for (const item of value) {
        if (isNode(item)) {
          children.push(item);
        }
      }
      continue;
    }

    if (isNode(value)) {
      children.push(value);
    }
  }

  return children;
}

function walkNode(node: AstNode, visit: (node: AstNode) => void): void {
  visit(node);

  for (const child of getChildNodes(node)) {
    walkNode(child, visit);
  }
}

function parseProgram(code: string, id: string): AstNode | null {
  try {
    const result = parseSync(normalizeParseId(id), code) as unknown;
    if (isNode(result)) {
      return result;
    }

    if (result != null && typeof result === "object") {
      const program = (result as { program?: unknown }).program;
      if (isNode(program)) {
        return program;
      }
    }

    return null;
  } catch {
    return null;
  }
}

function normalizeParseId(id: string): string {
  const withoutNullPrefix = id.startsWith("\0") ? id.slice(1) : id;
  const withoutQuery = withoutNullPrefix.split(/[?#]/, 1)[0];
  return withoutQuery || DEFAULT_PARSE_ID;
}

function isSetupFunction(node: AstNode): boolean {
  if (node.type !== "Property") {
    return false;
  }

  const key = isNode(node.key) ? node.key : null;
  const value = isNode(node.value) ? node.value : null;
  if (getNodeName(key) !== "setup" || !value) {
    return false;
  }

  if (value.type !== "FunctionExpression" && value.type !== "ArrowFunctionExpression") {
    return false;
  }

  const params = Array.isArray(value.params) ? value.params : [];
  return getNodeName(params[0] as AstNode | undefined) === "__props";
}

function findSetupFunctionBody(program: AstNode): AstNode | null {
  let setupBody: AstNode | null = null;

  walkNode(program, (node) => {
    if (setupBody || !isSetupFunction(node)) {
      return;
    }

    const value = isNode(node.value) ? node.value : null;
    const body = value && isNode(value.body) ? value.body : null;
    if (body?.type === "BlockStatement") {
      setupBody = body;
    }
  });

  return setupBody;
}

function isUseI18nCall(node: AstNode | null | undefined): boolean {
  if (!isNode(node) || node.type !== "CallExpression") {
    return false;
  }

  return getNodeName(isNode(node.callee) ? node.callee : null) === "useI18n";
}

function collectPatternBindingNames(pattern: AstNode, locals: Set<string>): void {
  if (pattern.type === "Identifier" && typeof pattern.name === "string") {
    locals.add(pattern.name);
    return;
  }

  if (pattern.type === "AssignmentPattern" && isNode(pattern.left)) {
    collectPatternBindingNames(pattern.left, locals);
    return;
  }

  if (pattern.type === "RestElement" && isNode(pattern.argument)) {
    collectPatternBindingNames(pattern.argument, locals);
    return;
  }

  if (pattern.type === "Property" && isNode(pattern.value)) {
    collectPatternBindingNames(pattern.value, locals);
    return;
  }

  for (const child of getChildNodes(pattern)) {
    collectPatternBindingNames(child, locals);
  }
}

function findExistingUseI18nDestructure(setupBody: AstNode): ExistingUseI18nDestructure | null {
  const statements = Array.isArray(setupBody.body) ? setupBody.body : [];

  for (const statement of statements) {
    if (!isNode(statement) || statement.type !== "VariableDeclaration") {
      continue;
    }

    const declarations = Array.isArray(statement.declarations) ? statement.declarations : [];
    for (const declaration of declarations) {
      if (
        !isNode(declaration) ||
        !isNode(declaration.id) ||
        declaration.id.type !== "ObjectPattern"
      ) {
        continue;
      }
      if (!isUseI18nCall(isNode(declaration.init) ? declaration.init : null)) {
        continue;
      }

      const locals = new Set<string>();
      collectPatternBindingNames(declaration.id, locals);
      return { declaration: statement, pattern: declaration.id, locals };
    }
  }

  return null;
}

function collectUsedI18nSpecifiers(setupBody: AstNode): {
  firstUseIndex: number;
  specifiers: string[];
} {
  const used = new Map<string, number>();

  walkNode(setupBody, (node) => {
    if (node.type !== "CallExpression") {
      return;
    }

    const callee = isNode(node.callee) ? node.callee : null;
    const fnName = getNodeName(callee);
    const callStart = getNodeStart(node);
    if (!fnName || callStart == null || !I18N_HELPER_LOCALS.has(fnName)) {
      return;
    }

    if (!used.has(fnName)) {
      used.set(fnName, callStart);
    }
  });

  const helpers = Array.from(used.entries()).sort((a, b) => a[1] - b[1]);
  return {
    firstUseIndex: helpers[0]?.[1] ?? Number.POSITIVE_INFINITY,
    specifiers: helpers.map(([name]) => I18N_FN_MAP[name]).filter((value) => value != null),
  };
}

export function injectNuxtI18nHelpers(code: string, id = DEFAULT_PARSE_ID): string {
  const program = parseProgram(code, id);
  if (!program) {
    return code;
  }

  const setupBody = findSetupFunctionBody(program);
  const setupBodyStart = setupBody ? getNodeStart(setupBody) : null;
  if (!setupBody || setupBodyStart == null) {
    return code;
  }

  const insertAt = setupBodyStart + 1;
  const { firstUseIndex, specifiers: usedSpecifiers } = collectUsedI18nSpecifiers(setupBody);
  if (usedSpecifiers.length === 0) {
    return code;
  }

  const existing = findExistingUseI18nDestructure(setupBody);

  if (existing) {
    const missingSpecifiers = usedSpecifiers.filter((specifier) => {
      return !existing.locals.has(getLocalAlias(specifier));
    });

    if (missingSpecifiers.length === 0) {
      return code;
    }

    const declarationStart = getNodeStart(existing.declaration);
    const declarationEnd = getNodeEnd(existing.declaration);
    const patternStart = getNodeStart(existing.pattern);
    const patternEnd = getNodeEnd(existing.pattern);
    if (
      declarationStart == null ||
      declarationEnd == null ||
      patternStart == null ||
      patternEnd == null
    ) {
      return code;
    }

    if (declarationStart > firstUseIndex) {
      return (
        code.slice(0, insertAt) +
        `\nconst { ${missingSpecifiers.join(", ")} } = useI18n();\n` +
        code.slice(insertAt)
      );
    }

    const merged = code.slice(patternStart + 1, patternEnd - 1).trim();
    const nextDestructure = merged
      ? `${merged}, ${missingSpecifiers.join(", ")}`
      : missingSpecifiers.join(", ");

    return (
      code.slice(0, declarationStart) +
      `const { ${nextDestructure} } = useI18n();` +
      code.slice(declarationEnd)
    );
  }

  return (
    code.slice(0, insertAt) +
    `\nconst { ${usedSpecifiers.join(", ")} } = useI18n();\n` +
    code.slice(insertAt)
  );
}
