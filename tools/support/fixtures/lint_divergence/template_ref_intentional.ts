import { createRequire } from "node:module";

interface Finding {
  ruleId: string;
  line: number | bigint;
  column: number | bigint;
  endLine: number | bigint;
  endColumn: number | bigint;
}
interface Node {
  type: string;
  [key: string]: unknown;
}
const requireFromTests = createRequire(new URL("../../../../tests/package.json", import.meta.url));

function node(value: unknown): Node | null {
  return typeof value === "object" &&
    value !== null &&
    "type" in value &&
    typeof value.type === "string"
    ? (value as Node)
    : null;
}
function offset(source: string, line: number | bigint, column: number | bigint): number | null {
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
function nodes(root: Node): Node[] {
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

function vueRefCall(program: Node, call: Node): boolean {
  const callee = node(call.callee);
  if (callee?.type !== "Identifier" || !Array.isArray(program.body)) return false;
  return program.body.some((value) => {
    const statement = node(value);
    return (
      statement?.type === "ImportDeclaration" &&
      node(statement.source)?.value === "vue" &&
      Array.isArray(statement.specifiers) &&
      statement.specifiers.some((value) => {
        const specifier = node(value),
          imported = node(specifier?.imported);
        return (
          specifier?.type === "ImportSpecifier" &&
          node(specifier.local)?.name === callee.name &&
          (imported?.name === "ref" || imported?.name === "shallowRef")
        );
      })
    );
  });
}

/** Refuse ambiguous bindings: only a proven write makes this migration unsafe. */
export function mutableTemplateRefReason(finding: Finding, source: string): string | null {
  if (finding.ruleId !== "vue/prefer-use-template-ref") return null;
  const start = offset(source, finding.line, finding.column);
  const end = offset(source, finding.endLine, finding.endColumn);
  if (start === null || end === null) return null;
  try {
    const sfc = requireFromTests("@vue/compiler-sfc") as {
      parse(source: string): {
        errors: unknown[];
        descriptor: {
          scriptSetup: {
            content: string;
            lang?: string;
            loc: { start: { offset: number } };
          } | null;
        };
      };
    };
    const parsed = sfc.parse(source),
      script = parsed.descriptor.scriptSetup;
    if (parsed.errors.length || script === null) return null;
    const parser = requireFromTests("@babel/parser") as {
      parse(source: string, options: { sourceType: string; plugins: string[] }): { program: Node };
    };
    const plugins = script.lang?.endsWith("x") ? ["typescript", "jsx"] : ["typescript"];
    const program = parser.parse(script.content, { sourceType: "module", plugins }).program;
    const declarations = (Array.isArray(program.body) ? program.body : [])
      .map(node)
      .filter(
        (statement) => statement?.type === "VariableDeclaration" && statement.kind === "const",
      )
      .flatMap((statement) =>
        Array.isArray(statement?.declarations) ? statement.declarations : [],
      )
      .map(node)
      .filter((declaration) => {
        const init = node(declaration?.init);
        return (
          init?.type === "CallExpression" &&
          vueRefCall(program, init) &&
          init.start === start - script.loc.start.offset &&
          init.end === end - script.loc.start.offset
        );
      });
    if (declarations.length !== 1) return null;
    const binding = node(declarations[0]?.id);
    if (binding?.type !== "Identifier" || typeof binding.name !== "string") return null;
    const all = nodes(program),
      allowed = new Set<Node>([binding]);
    let written = false;
    for (const current of all) {
      if (current.type !== "MemberExpression") continue;
      const object = node(current.object),
        property = node(current.property);
      if (object?.type !== "Identifier" || object.name !== binding.name) continue;
      allowed.add(object);
      const value = current.computed
        ? property?.type === "StringLiteral" && property.value === "value"
        : property?.type === "Identifier" && property.name === "value";
      if (
        value &&
        all.some(
          (parent) =>
            (parent.type === "AssignmentExpression" && parent.left === current) ||
            (parent.type === "UpdateExpression" && parent.argument === current),
        )
      )
        written = true;
    }
    if (
      !written ||
      all.some(
        (current) =>
          current.type === "Identifier" && current.name === binding.name && !allowed.has(current),
      )
    )
      return null;
    return "Patina retains a mutable template ref whose .value is assigned by its unshadowed setup binding; useTemplateRef() returns a readonly ref, so the upstream migration suggestion would introduce a TypeScript error.";
  } catch {
    // Unavailable parsers or invalid source cannot establish an exception.
    return null;
  }
}
