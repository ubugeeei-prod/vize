import { createHash } from "node:crypto";
import type { ModuleShapeNapi } from "@vizejs/native";

interface Node {
  type: string;
  name?: string;
  value?: unknown;
  [key: string]: unknown;
}

function node(value: unknown): value is Node {
  return typeof value === "object" && value !== null && typeof (value as Node).type === "string";
}

/** Private per-component state, with no global registry or public component properties. */
export const hmrRuntime = `
const records = new WeakMap();
const namespace = (value, source) => Object.keys(value).sort().map(key => [source + ':' + key, value[key]]);
export function remember(component, signature, collect) {
  if ((typeof component !== 'object' || component === null) && typeof component !== 'function') return;
  let imports = null;
  try { imports = collect(namespace); } catch {}
  records.set(component, { signature, imports });
}
export function read(component) { return records.get(component); }
export function same(before, after) {
  return before && after && before.signature === after.signature &&
    before.imports && after.imports && before.imports.length === after.imports.length &&
    after.imports.every((entry, i) => entry[0] === before.imports[i][0] && Object.is(entry[1], before.imports[i][1]));
}
`;

/**
 * Capture actual imported values before hot replacement. Equal module text alone
 * is insufficient: a script dependency can change while the SFC text stays equal.
 * Failed analysis, side effects, dynamic imports and cyclic TDZs disable reuse.
 */
export async function instrumentHmr(
  code: string,
  shape: ModuleShapeNapi | undefined,
  runtime: string,
  identity: string,
): Promise<string> {
  // Production and SSR never initialize a second parser or native binding.
  const { parseSync } = await import("oxc-parser");
  const parsed = parseSync("vize-parcel-output.js", code);
  if (parsed.errors.length) return code;
  const program = parsed.program as unknown as Node;
  if (!Array.isArray(program.body)) return code;
  const body = program.body.filter(node);
  if (
    body.some(
      (statement) =>
        (statement.type === "ExportAllDeclaration" ||
          statement.type === "ExportNamedDeclaration") &&
        statement.source,
    )
  )
    return code;
  const names = new Set<string>();
  let dynamic = false;
  const visit = (value: unknown): void => {
    if (Array.isArray(value)) {
      for (const child of value) visit(child);
      return;
    }
    if (!node(value)) return;
    if (typeof value.name === "string") names.add(value.name);
    if (typeof value.value === "string") names.add(value.value);
    if (value.type === "ImportExpression") dynamic = true;
    for (const child of Object.values(value)) if (typeof child === "object") visit(child);
  };
  visit(program);
  if (dynamic) return code;
  const unique = (base: string) => {
    let name = base;
    while (names.has(name)) name += "_";
    names.add(name);
    return name;
  };
  const remember = unique("__vize_parcel_remember");
  const namespace = unique("__vize_parcel_namespace");
  const imports: string[] = [];
  for (const statement of body) {
    if (statement.type !== "ImportDeclaration") continue;
    const source = node(statement.source) ? statement.source.value : null;
    if (
      typeof source !== "string" ||
      !Array.isArray(statement.specifiers) ||
      !statement.specifiers.length
    )
      return code;
    for (const specifier of statement.specifiers) {
      if (!node(specifier) || !node(specifier.local) || typeof specifier.local.name !== "string")
        return code;
      const local = specifier.local.name;
      const key = JSON.stringify(`${source}:${local}`);
      imports.push(
        specifier.type === "ImportNamespaceSpecifier"
          ? `...${namespace}(${local}, ${key})`
          : `[${key}, ${local}]`,
      );
    }
  }
  let target: string;
  let emitted = code;
  if (shape?.hasDefaultExport) {
    const statement = body.find((statement) => statement.type === "ExportDefaultDeclaration");
    if (!statement || !node(statement.declaration)) return code;
    const declaration = statement.declaration;
    if (declaration.type === "Identifier" && typeof declaration.name === "string")
      target = declaration.name;
    else if (
      (declaration.type === "FunctionDeclaration" || declaration.type === "ClassDeclaration") &&
      node(declaration.id) &&
      typeof declaration.id.name === "string"
    )
      target = declaration.id.name;
    else {
      target = unique("__vize_parcel_component");
      // OXC's JS API reports UTF-16 offsets, including after astral text.
      // Native byte offsets refer to the pre-assembly output, so do not reuse
      // them after asset rewrites or import insertion.
      if (typeof statement.start !== "number" || typeof statement.end !== "number") return code;
      const start = statement.start;
      const keyword = /^export\s+default\b/.exec(code.slice(start));
      if (!keyword) return code;
      const keywordEnd = start + keyword[0].length;
      const end = statement.end;
      emitted = `${code.slice(0, start)}const ${target} =${code.slice(keywordEnd, end)};${code.slice(end)}\nexport default ${target};`;
    }
  } else if (shape?.hasNamedRenderExport) target = "render";
  else return code;
  const signature = createHash("sha256").update(code).update(identity).digest("hex");
  return `import { remember as ${remember} } from ${JSON.stringify(runtime)};\n${emitted}\n${remember}(${target}, ${JSON.stringify(signature)}, (${namespace}) => [${imports.join(",")}]);\n`;
}
