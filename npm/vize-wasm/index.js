/**
 * Vize - WASM bindings
 *
 * This module provides WebAssembly bindings for the Vue compiler implemented in Rust.
 */

import initWasm, {
  Compiler as WasmCompiler,
  compile as wasmCompile,
  compileVapor as wasmCompileVapor,
  parseTemplate as wasmParseTemplate,
  parseSfc as wasmParseSfc,
  compileSfc as wasmCompileSfc,
  compileCss as wasmCompileCss,
} from "./vize_vitrine.js";

let initialized = false;
let initPromise = null;

/**
 * Initialize the WASM module.
 * Must be called before using any other functions.
 *
 * @param {RequestInfo | URL | Response | BufferSource | WebAssembly.Module} [moduleOrPath]
 * @returns {Promise<void>}
 */
export async function init(moduleOrPath) {
  if (initialized) {
    return;
  }

  if (initPromise) {
    return initPromise;
  }

  initPromise = initWasm(moduleOrPath).then(() => {
    initialized = true;
  });

  return initPromise;
}

/**
 * Check if the WASM module is initialized.
 *
 * @returns {boolean}
 */
export function isInitialized() {
  return initialized;
}

/**
 * Ensure WASM is initialized, throwing if not.
 */
function ensureInitialized() {
  if (!initialized) {
    throw new Error(
      "WASM module not initialized. Call `await init()` first."
    );
  }
}

/**
 * WASM Compiler class wrapper.
 */
export class Compiler {
  #inner;

  constructor() {
    ensureInitialized();
    this.#inner = new WasmCompiler();
  }

  /**
   * Compile template to VDom render function.
   *
   * @param {string} template
   * @param {object} [options]
   * @returns {object}
   */
  compile(template, options = {}) {
    return this.#inner.compile(template, options);
  }

  /**
   * Compile template to Vapor mode.
   *
   * @param {string} template
   * @param {object} [options]
   * @returns {object}
   */
  compileVapor(template, options = {}) {
    return this.#inner.compileVapor(template, options);
  }

  /**
   * Parse template to AST.
   *
   * @param {string} template
   * @param {object} [options]
   * @returns {object}
   */
  parse(template, options = {}) {
    return this.#inner.parse(template, options);
  }

  /**
   * Parse SFC (.vue file).
   *
   * @param {string} source
   * @param {object} [options]
   * @returns {object}
   */
  parseSfc(source, options = {}) {
    return this.#inner.parseSfc(source, options);
  }

  /**
   * Compile SFC (.vue file).
   *
   * @param {string} source
   * @param {object} [options]
   * @returns {object}
   */
  compileSfc(source, options = {}) {
    return this.#inner.compileSfc(source, options);
  }

  /**
   * Compile CSS with LightningCSS.
   *
   * @param {string} css
   * @param {object} [options]
   * @returns {object}
   */
  compileCss(css, options = {}) {
    return this.#inner.compileCss(css, options);
  }

  /**
   * Free the WASM memory.
   */
  free() {
    this.#inner.free();
  }

  /**
   * Disposable interface.
   */
  [Symbol.dispose]() {
    this.free();
  }
}

/**
 * Compile template to VDom render function.
 *
 * @param {string} template
 * @param {object} [options]
 * @returns {object}
 */
export function compile(template, options = {}) {
  ensureInitialized();
  return wasmCompile(template, options);
}

/**
 * Compile template to Vapor mode.
 *
 * @param {string} template
 * @param {object} [options]
 * @returns {object}
 */
export function compileVapor(template, options = {}) {
  ensureInitialized();
  return wasmCompileVapor(template, options);
}

/**
 * Parse template to AST.
 *
 * @param {string} template
 * @param {object} [options]
 * @returns {object}
 */
export function parseTemplate(template, options = {}) {
  ensureInitialized();
  return wasmParseTemplate(template, options);
}

/**
 * Parse SFC (.vue file).
 *
 * @param {string} source
 * @param {object} [options]
 * @returns {object}
 */
export function parseSfc(source, options = {}) {
  ensureInitialized();
  return wasmParseSfc(source, options);
}

/**
 * Compile SFC (.vue file).
 *
 * @param {string} source
 * @param {object} [options]
 * @returns {object}
 */
export function compileSfc(source, options = {}) {
  ensureInitialized();
  return wasmCompileSfc(source, options);
}

/**
 * Compile CSS with LightningCSS.
 *
 * @param {string} css
 * @param {object} [options]
 * @returns {object}
 */
export function compileCss(css, options = {}) {
  ensureInitialized();
  return wasmCompileCss(css, options);
}

export default init;
