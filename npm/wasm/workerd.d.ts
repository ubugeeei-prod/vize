import {
  Compiler,
  compile,
  compileCss,
  compileSfc,
  compileVapor,
  parseCssAst,
  parseSfc,
  parseTemplate,
  printCssAst,
} from "./vize_workerd.js";

export interface VizeWorkerdBinding {
  readonly Compiler: typeof Compiler;
  readonly compile: typeof compile;
  readonly compileCss: typeof compileCss;
  readonly compileSfc: typeof compileSfc;
  readonly compileVapor: typeof compileVapor;
  readonly parseCssAst: typeof parseCssAst;
  readonly parseSfc: typeof parseSfc;
  readonly parseTemplate: typeof parseTemplate;
  readonly printCssAst: typeof printCssAst;
}

/** Initialize Vize from a workerd `CompiledWasm` module. */
export declare function instantiate(
  module: WebAssembly.Module,
): Promise<Readonly<VizeWorkerdBinding>>;
