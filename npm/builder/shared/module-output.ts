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

/** One native-reported import binding that can participate in HMR snapshots. */
export type ModuleImportInfo = {
  /** Module specifier used by the emitted import. */
  source: string;
  /** Local binding name in the emitted module. */
  local: string;
  /** Whether the binding represents an import namespace. */
  namespace: boolean;
};

/** Native module-shape facts needed to decide whether state can be preserved. */
export type HmrModuleShape = {
  /** Imported bindings whose current values are snapshotted before replacement. */
  imports?: ModuleImportInfo[];
  /** Whether a bare side-effect import makes preservation unsafe. */
  hasSideEffectImport?: boolean;
};

/** Snapshot native-reported imported values before a hot replacement. */
export function hmrImportSnapshot(
  shape: HmrModuleShape | undefined,
  hostImports: readonly ModuleImportInfo[] = [],
): string | null {
  if (!shape?.imports || shape.hasSideEffectImport) return null;
  const entries: string[] = [];
  for (const { source, local, namespace } of [...shape.imports, ...hostImports]) {
    // Vue helpers can change with the template. CSS has its own HMR boundary.
    if (source === "vue" || source.includes("?vue&type=style&")) continue;
    const key = JSON.stringify(`${source}:${local}`);
    entries.push(
      namespace
        ? `...Object.keys(${local}).sort().map(key => [${key} + ':' + key, ${local}[key]])`
        : `[${key}, ${local}]`,
    );
  }
  return `[${entries.join(", ")}]`;
}
