// Generated binding names that cannot collide with any identifier the emitted
// module declares or reads. Kept apart from `module-output.ts` because it
// parses the module with oxc: only a builder that allocates names loads it.
import { parseSync } from "oxc-parser";

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
