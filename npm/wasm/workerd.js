/**
 * Cloudflare Workers adapter for the focused Vize compiler artifact.
 *
 * Instantiate inside `fetch()` because workerd provides the imported `.wasm`
 * file as a precompiled `WebAssembly.Module`. The cached promise keeps warm
 * requests on the same initialized binding.
 */

import initWasm, {
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

const binding = Object.freeze({
  Compiler,
  compile,
  compileCss,
  compileSfc,
  compileVapor,
  parseCssAst,
  parseSfc,
  parseTemplate,
  printCssAst,
});

let bindingPromise;

/**
 * Initialize Vize from a workerd `CompiledWasm` module.
 *
 * @param {WebAssembly.Module} module
 * @returns {Promise<typeof binding>}
 */
export function instantiate(module) {
  bindingPromise ??= initWasm({ module_or_path: module })
    .then(() => binding)
    .catch((error) => {
      bindingPromise = undefined;
      throw error;
    });
  return bindingPromise;
}
