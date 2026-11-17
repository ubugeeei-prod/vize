/**
 * `.tsrx` modules in the bundler plugins (#6858, #13140).
 *
 * Every plugin compiles a TSRX module through the native `compileTsrx`, the
 * entry `vize build` also uses. A refused module throws one error that lists
 * every diagnostic at its authored line and column; there is never partial
 * output.
 */

/** Options of the native `compileTsrx`. */
export interface TsrxCompileOptions {
  filename?: string;
  target?: "dom" | "ssr" | "vapor";
  sourceMap?: boolean;
  eraseTypes?: boolean;
}

/** One diagnostic of a refused module. */
export interface TsrxDiagnostic {
  /** `TSRX2001`, a shared `TS1012`, or `unsupported`. */
  code: string;
  message: string;
  /** UTF-8 byte offsets into the source. */
  start: number;
  end: number;
  /** 1-based line of `start`. */
  line: number;
  /** 0-based UTF-16 column of `start`. */
  column: number;
}

/** Result of the native `compileTsrx`. */
export interface TsrxCompileResult {
  code?: string | null;
  map?: string | null;
  errors: TsrxDiagnostic[];
}

export type CompileTsrxFn = (source: string, options?: TsrxCompileOptions) => TsrxCompileResult;

/** The ids a plugin compiles as TSRX unless its `exclude` matches them. */
export const TSRX_INCLUDE_PATTERN = /\.tsrx$/;

export function isTsrxFile(filePath: string): boolean {
  return TSRX_INCLUDE_PATTERN.test(filePath);
}

/** A refused module, in the shape Rollup, Vite and webpack report with a location. */
export class TsrxCompileError extends Error {
  readonly id: string;
  readonly diagnostics: readonly TsrxDiagnostic[];
  readonly loc: { file: string; line: number; column: number };

  constructor(filePath: string, diagnostics: readonly TsrxDiagnostic[]) {
    const lines = diagnostics.map(
      (diagnostic) =>
        `  - ${filePath}:${diagnostic.line}:${diagnostic.column + 1} [${diagnostic.code}] ${diagnostic.message}`,
    );
    super(`[vize] TSRX compilation failed in ${filePath}:\n${lines.join("\n")}`);
    this.name = "TsrxCompileError";
    this.id = filePath;
    this.diagnostics = diagnostics;
    const [first] = diagnostics;
    this.loc = { file: filePath, line: first?.line ?? 1, column: first?.column ?? 0 };
  }
}

export interface TsrxModuleOptions {
  ssr?: boolean;
  vapor?: boolean;
  /** Return JavaScript; otherwise the module is TypeScript without JSX. */
  eraseTypes?: boolean;
}

/** The target a plugin request selects: SSR wins over Vapor, as in `vize build`. */
export function tsrxTarget(
  options: Pick<TsrxModuleOptions, "ssr" | "vapor">,
): "dom" | "ssr" | "vapor" {
  if (options.ssr) return "ssr";
  return options.vapor ? "vapor" : "dom";
}

/**
 * Compile one `.tsrx` module.
 *
 * @throws {TsrxCompileError} with every diagnostic when the module is refused.
 */
export function compileTsrxModule(
  compileTsrx: CompileTsrxFn,
  filePath: string,
  source: string,
  options: TsrxModuleOptions = {},
): { code: string; map: string | null } {
  const result = compileTsrx(source, {
    filename: filePath,
    target: tsrxTarget(options),
    // The DOM target records no span links yet, so the entry refuses source
    // maps (#13131); requesting one would refuse every module.
    sourceMap: false,
    eraseTypes: options.eraseTypes ?? false,
  });
  if (result.errors.length > 0 || typeof result.code !== "string") {
    throw new TsrxCompileError(filePath, result.errors);
  }
  return { code: result.code, map: null };
}
