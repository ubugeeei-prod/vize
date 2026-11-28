import { createRequire } from "node:module";

/** A finding located by 1-based line and column, as both linters report it. */
export interface Finding {
  ruleId: string;
  line: number | bigint;
  column: number | bigint;
  endLine: number | bigint;
  endColumn: number | bigint;
  message: string;
}
export interface Node {
  type: string;
  [key: string]: unknown;
}
interface Block {
  content: string;
  lang?: string;
  loc: { start: { offset: number } };
}
export interface TemplateNode {
  type: number;
  tag?: string;
  props?: TemplateNode[];
  children?: TemplateNode[];
  name?: string;
  value?: { content: string } | null;
  arg?: { content?: string } | null;
  loc: { start: { offset: number } };
}
interface Compiler {
  parse(source: string): {
    errors: unknown[];
    descriptor: {
      script: Block | null;
      scriptSetup: Block | null;
      template: (Block & { ast?: TemplateNode | null }) | null;
    };
  };
  babelParse(source: string, options: { sourceType: string; plugins: string[] }): { program: Node };
}

const requireFromTests = createRequire(new URL("../../../../tests/package.json", import.meta.url));
const requireFromBench = createRequire(
  new URL("../../../benchmarks/scripts/package.json", import.meta.url),
);
function compiler(): Compiler {
  // Both execution scopes already pin compiler-sfc to the same catalog entry.
  // Unit checks install tests; corpus qualification installs benchmark tools.
  try {
    return requireFromBench("@vue/compiler-sfc") as Compiler;
  } catch {
    return requireFromTests("@vue/compiler-sfc") as Compiler;
  }
}

export function node(value: unknown): Node | null {
  return typeof value === "object" &&
    value !== null &&
    "type" in value &&
    typeof value.type === "string"
    ? (value as Node)
    : null;
}
export function offset(
  source: string,
  line: number | bigint,
  column: number | bigint,
): number | null {
  const row = Number(line),
    col = Number(column),
    lines = source.split("\n");
  if (
    !Number.isSafeInteger(row) ||
    !Number.isSafeInteger(col) ||
    row < 1 ||
    col < 1 ||
    row > lines.length ||
    col > lines[row - 1].length + 1
  )
    return null;
  return lines.slice(0, row - 1).reduce((total, text) => total + text.length + 1, 0) + col - 1;
}
export function nodes(root: Node): Node[] {
  const result: Node[] = [];
  const visit = (value: unknown) => {
    if (Array.isArray(value)) {
      for (const item of value) visit(item);
      return;
    }
    const current = node(value);
    if (current === null) return;
    result.push(current);
    for (const [key, child] of Object.entries(current))
      if (!["parent", "comments", "tokens", "loc"].includes(key)) visit(child);
  };
  visit(root);
  return result;
}
export function statements(program: Node): Node[] {
  return (Array.isArray(program.body) ? program.body : []).flatMap((value) => node(value) ?? []);
}

export interface SetupScript {
  /** The `<script setup>` program; offsets are relative to `start`. */
  program: Node;
  /** Offset of the setup block's content in the SFC source. */
  start: number;
  /** Content of a sibling `<script>` block, if the SFC has one. */
  sibling: string | null;
}
/** Parse the `<script setup>` block of a valid SFC, or `null` when there is none. */
export function setupScript(source: string): SetupScript | null {
  const sfc = compiler();
  const parsed = sfc.parse(source),
    script = parsed.descriptor.scriptSetup;
  if (parsed.errors.length || script === null) return null;
  const plugins = script.lang?.endsWith("x") ? ["typescript", "jsx"] : ["typescript"];
  return {
    program: sfc.babelParse(script.content, { sourceType: "module", plugins }).program,
    start: script.loc.start.offset,
    sibling: parsed.descriptor.script?.content ?? null,
  };
}
/** The parsed template root of a valid SFC; node offsets are SFC offsets. */
export function templateRoot(source: string): TemplateNode | null {
  const parsed = compiler().parse(source);
  return parsed.errors.length ? null : (parsed.descriptor.template?.ast ?? null);
}

/**
 * Whether `callee` calls Vue's export `exported` (`ref`, `useTemplateRef`, …):
 * through a value import from `vue` under any local name, or as the bare
 * auto-imported name when neither script block declares or imports it.
 */
export function callsVueExport(setup: SetupScript, callee: unknown, exported: string[]): boolean {
  const name = node(callee);
  if (name?.type !== "Identifier" || typeof name.name !== "string") return false;
  const local = name.name;
  let imported: boolean | null = null;
  for (const statement of statements(setup.program)) {
    if (statement.type !== "ImportDeclaration" || !Array.isArray(statement.specifiers)) continue;
    for (const value of statement.specifiers) {
      const specifier = node(value);
      if (node(specifier?.local)?.name !== local) continue;
      const source = node(statement.source)?.value,
        name = node(specifier?.imported)?.name;
      imported =
        source === "vue" &&
        statement.importKind !== "type" &&
        specifier?.type === "ImportSpecifier" &&
        specifier.importKind !== "type" &&
        typeof name === "string" &&
        exported.includes(name);
    }
  }
  if (imported !== null) return imported;
  // An auto-imported name: no binding of that name anywhere in the setup
  // block, and no sibling block that could import or declare it.
  if (!exported.includes(local)) return false;
  if (setup.sibling !== null && new RegExp(`\\b${local}\\b`, "u").test(setup.sibling)) return false;
  return nodes(setup.program).every(
    (current) =>
      !(
        ["VariableDeclarator", "FunctionDeclaration", "ClassDeclaration"].includes(current.type) &&
        node(current.id)?.name === local
      ) &&
      !(
        Array.isArray(current.params) && current.params.some((param) => node(param)?.name === local)
      ),
  );
}
