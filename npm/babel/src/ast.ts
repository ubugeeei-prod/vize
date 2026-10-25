import type { AstNode, BabelApi, FileAst, ParserOptions } from "./types.ts";

export function isNode(value: unknown): value is AstNode {
  return (
    typeof value === "object" && value !== null && "type" in value && typeof value.type === "string"
  );
}

export function hasJsx(ast: AstNode, keys: BabelApi["types"]["VISITOR_KEYS"]): boolean {
  const pending: AstNode[] = [ast];
  while (pending.length > 0) {
    const node = pending.pop()!;
    if (node.type === "JSXElement" || node.type === "JSXFragment") return true;
    for (const key of keys[node.type] ?? []) {
      const child = node[key];
      if (Array.isArray(child)) {
        for (const item of child) if (isNode(item)) pending.push(item);
      } else if (isNode(child)) pending.push(child);
    }
  }
  return false;
}

export function enableSyntax(filename: string, parser: ParserOptions): void {
  const plugins = (parser.plugins ??= []);
  const has = (name: string) => plugins.some((p) => (Array.isArray(p) ? p[0] : p) === name);
  if (/\.tsx$/iu.test(filename) && !has("typescript")) plugins.push("typescript");
  if (!/\.(?:ts|mts|cts)$/iu.test(filename) && !has("jsx")) plugins.push("jsx");
}

/** Explicit foreign runtime selection is a configuration error, never Vue output. */
export function rejectForeignRuntime(ast: FileAst): void {
  const body = ast.program.body;
  if (Array.isArray(body)) {
    for (const node of body) {
      if (!isNode(node) || node.type !== "ImportDeclaration" || node.importKind === "type")
        continue;
      const source = node.source;
      if (
        isNode(source) &&
        typeof source.value === "string" &&
        /^react(?:\/|$)/u.test(source.value)
      ) {
        throw new Error(
          "Vize compiles Vue JSX; scope this plugin to Vue files using Babel overrides",
        );
      }
    }
  }
  const comments = ast.comments;
  if (Array.isArray(comments)) {
    for (const comment of comments) {
      if (!isNode(comment) || typeof comment.value !== "string") continue;
      const source = /@jsxImportSource\s+(\S+)/u.exec(comment.value)?.[1];
      if (source && source !== "vue") {
        throw new Error(`Vize cannot compile the explicitly selected JSX import source: ${source}`);
      }
      const factory = /@jsx\s+(\S+)/u.exec(comment.value)?.[1];
      if (factory)
        throw new Error("Custom JSX factory pragmas are not supported by the native Babel adapter");
    }
  }
}
