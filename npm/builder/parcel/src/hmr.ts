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

/** Record actual module evaluation without adding exports or wrapping authored ESM. */
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
  const names = new Set<string>();
  const visit = (value: unknown): void => {
    if (Array.isArray(value)) {
      for (const child of value) visit(child);
      return;
    }
    if (!node(value)) return;
    if (typeof value.name === "string") names.add(value.name);
    if (typeof value.value === "string") names.add(value.value);
    for (const child of Object.values(value)) if (typeof child === "object") visit(child);
  };
  visit(program);
  const unique = (base: string) => {
    let name = base;
    while (names.has(name)) name += "_";
    names.add(name);
    return name;
  };
  const remember = unique("__vize_parcel_remember");
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
  return `import { remember as ${remember} } from ${JSON.stringify(runtime)};\n${emitted}\n${remember}(${target}, ${JSON.stringify(identity)});\n`;
}
