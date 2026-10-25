import type { AstNode, BabelApi, FileAst, ParserOptions, Scope } from "./types.ts";

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
export function rejectForeignRuntime(ast: FileAst, plugins: unknown, scope: Scope): void {
  if (Array.isArray(plugins)) {
    for (const plugin of plugins) {
      if (typeof plugin !== "object" || plugin === null || !("key" in plugin)) continue;
      if (
        typeof plugin.key === "string" &&
        /^(?:@babel\/plugin-)?transform-react-jsx(?:-development|\/development)?$/u.test(plugin.key)
      ) {
        rejectReact();
      }
    }
  }
  const body = ast.program.body;
  if (Array.isArray(body)) {
    for (const node of body) {
      if (isNode(node) && node.type === "TSImportEqualsDeclaration" && node.importKind !== "type") {
        const owner = node.moduleReference;
        if (isNode(owner) && isNode(owner.expression) && foreignName(owner.expression.value))
          rejectReact();
      }
      if (isNode(node) && node.type === "VariableDeclaration" && !scope.getBinding("require")) {
        const declarations = node.declarations;
        if (Array.isArray(declarations))
          for (const declaration of declarations) {
            if (!isNode(declaration) || !isNode(declaration.init)) continue;
            const call = declaration.init;
            if (
              call.type === "CallExpression" &&
              isNode(call.callee) &&
              call.callee.type === "Identifier" &&
              call.callee.name === "require" &&
              Array.isArray(call.arguments) &&
              call.arguments.length === 1 &&
              isNode(call.arguments[0]) &&
              foreignName(call.arguments[0].value)
            )
              rejectReact();
          }
      }
      if (!isNode(node) || node.type !== "ImportDeclaration" || node.importKind === "type")
        continue;
      const specifiers = node.specifiers;
      if (
        Array.isArray(specifiers) &&
        specifiers.length > 0 &&
        specifiers.every((specifier) => isNode(specifier) && specifier.importKind === "type")
      )
        continue;
      const source = node.source;
      if (isNode(source) && typeof source.value === "string" && foreignName(source.value)) {
        rejectReact();
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

function foreignName(value: unknown): boolean {
  return typeof value === "string" && /^react(?:\/|$)/u.test(value);
}

function rejectReact(): never {
  throw new Error("Vize compiles Vue JSX; scope this plugin to Vue files using Babel overrides");
}
