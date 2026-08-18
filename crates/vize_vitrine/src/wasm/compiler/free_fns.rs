//! Free-function aliases over the `Compiler` class bindings.
//!
//! Kept separate from `compiler.rs` so that file stays focused on the compile
//! pipeline itself.

use wasm_bindgen::prelude::*;

use super::Compiler;

/// Compile template to VDom (free function)
#[wasm_bindgen]
pub fn compile(template: &str, options: JsValue) -> Result<JsValue, JsValue> {
    Compiler::new().compile(template, options)
}

/// Compile template to Vapor mode (free function)
#[wasm_bindgen(js_name = "compileVapor")]
pub fn compile_vapor_fn(template: &str, options: JsValue) -> Result<JsValue, JsValue> {
    Compiler::new().compile_vapor(template, options)
}

/// Parse template to AST (free function)
#[wasm_bindgen(js_name = "parseTemplate")]
pub fn parse_template(template: &str, options: JsValue) -> Result<JsValue, JsValue> {
    Compiler::new().parse(template, options)
}

/// Parse SFC (free function)
#[wasm_bindgen(js_name = "parseSfc")]
pub fn parse_sfc_fn(source: &str, options: JsValue) -> Result<JsValue, JsValue> {
    Compiler::new().parse_sfc_method(source, options)
}

/// Compile SFC (free function)
#[wasm_bindgen(js_name = "compileSfc")]
pub fn compile_sfc_fn(source: &str, options: JsValue) -> Result<JsValue, JsValue> {
    Compiler::new().compile_sfc(source, options)
}

/// Parse CSS to AST (free function)
#[wasm_bindgen(js_name = "parseCssAst")]
pub fn parse_css_ast_fn(css: &str, options: JsValue) -> Result<JsValue, JsValue> {
    Compiler::new().parse_css_ast_method(css, options)
}

/// Print CSS from AST (free function)
#[wasm_bindgen(js_name = "printCssAst")]
pub fn print_css_ast_fn(ast: JsValue, options: JsValue) -> Result<JsValue, JsValue> {
    Compiler::new().print_css_ast_method(ast, options)
}

/// Compile CSS (free function)
#[wasm_bindgen(js_name = "compileCss")]
pub fn compile_css_fn(css: &str, options: JsValue) -> Result<JsValue, JsValue> {
    Compiler::new().compile_css_method(css, options)
}
