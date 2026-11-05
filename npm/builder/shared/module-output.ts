import { parseSync } from "oxc-parser";
import { transformSync } from "oxc-transform";

const OUTPUT_PARSE_ID = "vize-rspack-output.tsx";

type AstNode = {
  type?: string;
  start?: number;
  end?: number;
  [key: string]: unknown;
};

function isNode(value: unknown): value is AstNode {
  return value != null && typeof value === "object" && typeof (value as AstNode).type === "string";
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

const GENERATED_PREFIXES = ["__vize_", "_imports_"];

/** Reserve decoded identifiers only when generated names or escapes can collide. */
export function createGeneratedNameAllocator(code: string, onSemanticParse?: () => void) {
  const names = new Set<string>();
  if (code.includes("\\u") || GENERATED_PREFIXES.some((prefix) => code.includes(prefix))) {
    onSemanticParse?.();
    const program = parseProgram(code);
    if (!program) throw new Error("[vize] Cannot safely allocate generated module bindings.");
    const pending: unknown[] = [program];
    while (pending.length) {
      const value = pending.pop();
      if (Array.isArray(value)) {
        for (const child of value) pending.push(child);
      } else if (value !== null && typeof value === "object") {
        const node = value as AstNode;
        if (node.type === "Identifier" && typeof node.name === "string") names.add(node.name);
        pending.push(...Object.values(node));
      }
    }
  }
  return (base: string): string => {
    if (!GENERATED_PREFIXES.some((prefix) => base.startsWith(prefix))) {
      throw new Error("[vize] Unknown generated binding family.");
    }
    let name = base;
    while (names.has(name)) name += "_";
    names.add(name);
    return name;
  };
}

function getProgramBody(program: AstNode | null): AstNode[] {
  if (!program || !Array.isArray(program.body)) {
    return [];
  }

  return program.body.filter(isNode);
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
