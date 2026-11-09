import type { JsxCompileResultNapi, StyleBlockNapi } from "@vizejs/native";

/** JSX output defaults; component directive prologues can select another mode. */
export interface VizeBabelOptions {
  /** JSX lowering mode used unless a component directive overrides it. @default "vdom" */
  jsxMode?: "vdom" | "vapor";
  /** Compatibility strategy used for Babel-owned JSX transforms. @default "native" */
  jsxCompat?: "native" | "babel";
  /** Emit server-rendering output. @default false */
  ssr?: boolean;
  /** Explicitly hand extracted CSS to a bundler through Babel file metadata. @default "error" */
  scopedStyles?: "error" | "metadata";
  /** How standalone Babel transforms handle Vue SFC style blocks. @default "inject" */
  sfcStyles?: "inject" | "metadata" | "error";
  /** Root used to make SFC scope IDs stable across machines. @default process.cwd() */
  root?: string;
  /** Enables production SFC output and content-sensitive production scope IDs. @default NODE_ENV === "production" */
  isProduction?: boolean;
}

/** Metadata recorded for a native JSX transform. */
export interface VizeBabelMetadata {
  defaultMode: "vdom" | "vapor";
  ssr: boolean;
  nativePasses: 1;
  warnings: string[];
  scopedStyles: JsxCompileResultNapi["scopedStyles"];
}

/** Metadata recorded for a native SFC transform. */
export interface VizeBabelSfcMetadata {
  defaultMode: "vdom" | "vapor";
  ssr: boolean;
  nativePasses: 1;
  warnings: string[];
  css: string | null;
  styles: StyleBlockNapi[];
  dependencies: string[];
}

// The plugin is structurally typed across the two host AST versions. Babel
// owns these objects; no second Babel installation is loaded by the plugin.
/** Smallest AST node shape shared by supported Babel host versions. */
export interface AstNode {
  type: string;
  [key: string]: unknown;
}

/** Structurally typed Babel file AST. */
export interface FileAst extends AstNode {
  type: "File";
  program: AstNode;
}

/** Parser controls consumed by the structurally typed Babel adapter. */
export interface ParserOptions {
  /** Current input filename. @default undefined */
  filename?: string;
  /** Authored source filename used in emitted maps. @default undefined */
  sourceFilename?: string;
  /** Legacy authored source filename alias. @default undefined */
  sourceFileName?: string;
  /** Babel parser plugins enabled for the input. @default [] */
  plugins?: (string | [string, Record<string, unknown>])[];
  [key: string]: unknown;
}

/** Babel lexical-scope operations required by native recompilation. */
export interface Scope {
  crawl(): void;
  bindings: Record<string, unknown>;
  uids: Record<string, boolean>;
  uidsSet?: Set<string>;
  generateUid(name: string): string;
  rename(name: string, replacement: string): void;
  getBinding(name: string): unknown;
}

/** Babel file operations required by the adapter. */
export interface BabelFile {
  ast: FileAst;
  code: string;
  metadata: Record<string, unknown>;
  opts: Record<string, unknown> & {
    filename?: string;
    sourceFileName?: string;
    sourceRoot?: string;
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

/** Host parser function used to materialize compiled source as a Babel AST. */
export type BabelParse = (code: string, options?: Record<string, unknown>) => FileAst;

/** Printed Babel program and optional source map. */
export interface SerializedAst {
  code: string | null;
  map: object | null;
}

/** Babel host operations consumed by the Vize plugin. */
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
