import type { JsxCompileResultNapi } from "@vizejs/native";

/** JSX output defaults; component directive prologues can select another mode. */
export interface VizeBabelOptions {
  jsxMode?: "vdom" | "vapor";
  jsxCompat?: "native" | "babel";
  ssr?: boolean;
  /** Explicitly hand extracted CSS to a bundler through Babel file metadata. */
  scopedStyles?: "error" | "metadata";
}

export interface VizeBabelMetadata {
  defaultMode: "vdom" | "vapor";
  ssr: boolean;
  nativePasses: 1;
  warnings: string[];
  scopedStyles: JsxCompileResultNapi["scopedStyles"];
}

// The plugin is structurally typed across the two host AST versions. Babel
// owns these objects; no second Babel installation is loaded by the plugin.
export interface AstNode {
  type: string;
  [key: string]: unknown;
}

export interface FileAst extends AstNode {
  type: "File";
  program: AstNode;
}

export interface ParserOptions {
  plugins?: (string | [string, Record<string, unknown>])[];
  [key: string]: unknown;
}

export interface Scope {
  crawl(): void;
  bindings: Record<string, unknown>;
  uids: Record<string, boolean>;
  uidsSet?: Set<string>;
  generateUid(name: string): string;
  rename(name: string, replacement: string): void;
  getBinding(name: string): unknown;
}

export interface BabelFile {
  ast: FileAst;
  code: string;
  metadata: Record<string, unknown>;
  opts: Record<string, unknown> & {
    filename?: string;
    parserOpts?: ParserOptions;
    sourceMaps?: boolean | "inline" | "both";
  };
  inputMap: { toObject(): object } | null;
  path: {
    replaceWith(node: AstNode): void;
    scope: Scope;
    traverse(visitor: { Scopable(path: { scope: Scope }): void }): void;
  };
  scope: Scope;
  get(key: symbol): unknown;
  set(key: symbol, value: unknown): void;
}

export interface SerializedAst {
  code: string | null;
  map: object | null;
}

export interface BabelApi {
  assertVersion(range: string): void;
  parseSync(code: string, options: Record<string, unknown>): FileAst | null;
  transformFromAstSync(
    ast: FileAst,
    code: string,
    options: Record<string, unknown>,
  ): SerializedAst | null;
  types: { VISITOR_KEYS: Record<string, readonly string[]> };
}
