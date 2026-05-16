// Re-export all types
export type * from "./compiler";
export type * from "./croquis";
export type * from "./features";
export type * from "./analysis";

import type {
  CompilerOptions,
  CompileResult,
  CssCompileOptions,
  CssCompileResult,
  SfcDescriptor,
  SfcCompileResult,
} from "./compiler";
import type { CroquisOptions, CroquisResult } from "./croquis";
import type {
  ArtParseOptions,
  ArtDescriptor,
  CsfOutput,
  LintOptions,
  LintResult,
  LintRule,
  LocaleInfo,
  FormatOptions,
  FormatResult,
} from "./features";
import type {
  TypeCheckOptions,
  TypeCheckResult,
  TypeCheckCapabilities,
  CrossFileInput,
  CrossFileOptions,
  CrossFileResult,
} from "./analysis";

export interface WasmModule {
  compile: (template: string, options: CompilerOptions) => CompileResult;
  compileVapor: (template: string, options: CompilerOptions) => CompileResult;
  compileCss: (css: string, options: CssCompileOptions) => CssCompileResult;
  parseTemplate: (template: string, options: CompilerOptions) => object;
  parseSfc: (source: string, options: CompilerOptions) => SfcDescriptor;
  compileSfc: (source: string, options: CompilerOptions) => SfcCompileResult;
  // Analysis functions
  analyzeSfc: (source: string, options: CroquisOptions) => CroquisResult;
  analyzeCrossFile: (files: CrossFileInput[], options: CrossFileOptions) => CrossFileResult;
  // Musea functions
  parseArt: (source: string, options: ArtParseOptions) => ArtDescriptor;
  artToCsf: (source: string, options: ArtParseOptions) => CsfOutput;
  // Patina (Linter) functions
  lintTemplate: (source: string, options: LintOptions) => LintResult;
  lintSfc: (source: string, options: LintOptions) => LintResult;
  getLintRules: () => LintRule[];
  getLocales: () => LocaleInfo[];
  // Glyph (Formatter) functions
  formatSfc: (source: string, options: FormatOptions) => FormatResult;
  formatTemplate: (source: string, options: FormatOptions) => FormatResult;
  formatScript: (source: string, options: FormatOptions) => FormatResult;
  // Canon (TypeCheck) functions
  typeCheck: (source: string, options: TypeCheckOptions) => TypeCheckResult;
  getTypeCheckCapabilities: () => TypeCheckCapabilities;
  Compiler: new () => {
    compile: (template: string, options: CompilerOptions) => CompileResult;
    compileVapor: (template: string, options: CompilerOptions) => CompileResult;
    compileCss: (css: string, options: CssCompileOptions) => CssCompileResult;
    parse: (template: string, options: CompilerOptions) => object;
    parseSfc: (source: string, options: CompilerOptions) => SfcDescriptor;
    compileSfc: (source: string, options: CompilerOptions) => SfcCompileResult;
    analyzeSfc: (source: string, options: CroquisOptions) => CroquisResult;
  };
}
