use oxc_span::SourceType;

use super::ImportRewriter;

#[test]
fn rewrites_ts_import_type_specifiers() {
    let rewriter = ImportRewriter::new();
    let source = r#"type AppModule = typeof import('./App.vue');
type AppProps = import("./App.vue").PublicProps;"#;
    let result = rewriter.rewrite(source, SourceType::ts());

    assert_eq!(
        result.code,
        r#"type AppModule = typeof import('./App.vue.ts');
type AppProps = import("./App.vue.ts").PublicProps;"#
    );
}

#[test]
fn rewrites_declaration_ts_import_type_specifiers() {
    let rewriter = ImportRewriter::new();
    let source = r#"export type AppModule = typeof import('./App.vue.ts');
export type AppProps = import("./App.vue.ts").PublicProps;"#;
    let result = rewriter.rewrite_declaration_specifiers(source, SourceType::ts());

    assert_eq!(
        result.code,
        r#"export type AppModule = typeof import('./App.vue');
export type AppProps = import("./App.vue").PublicProps;"#
    );
}

#[test]
fn rewrites_ts_import_equals_external_module_references() {
    let rewriter = ImportRewriter::new();
    let source = r#"import App = require("./App.vue");
export type AppModule = typeof App;"#;
    let result = rewriter.rewrite(source, SourceType::ts());

    assert_eq!(
        result.code,
        r#"import App = require("./App.vue.ts");
export type AppModule = typeof App;"#
    );
}

#[test]
fn rewrites_common_js_require_specifiers() {
    let rewriter = ImportRewriter::new();
    let source = r#"const App = require("./App.vue");
const util = require("./util");"#;
    let result = rewriter.rewrite(source, SourceType::ts());

    assert_eq!(
        result.code,
        r#"const App = require("./App.vue.ts");
const util = require("./util");"#
    );
}

#[test]
fn rewrites_declaration_require_specifiers() {
    let rewriter = ImportRewriter::new();
    let source = r#"import App = require("./App.vue.ts");
export { App };"#;
    let result = rewriter.rewrite_declaration_specifiers(source, SourceType::ts());

    assert_eq!(
        result.code,
        r#"import App = require("./App.vue");
export { App };"#
    );
}

#[test]
fn collects_relative_vue_specifiers_from_require_forms() {
    let rewriter = ImportRewriter::new();
    let source = r#"import App = require("./App.vue");
const Other = require("../Other.vue");"#;

    assert_eq!(
        rewriter.collect_relative_vue_specifiers(source, SourceType::ts()),
        vec!["./App.vue", "../Other.vue"]
    );
}

#[test]
fn rewrites_module_declaration_specifiers() {
    let rewriter = ImportRewriter::new();
    let source = r#"declare module "./App.vue" {
  export const marker: true;
}
declare module "*.vue" {
  const component: unknown;
  export default component;
}"#;
    let result = rewriter.rewrite(source, SourceType::ts());

    assert_eq!(
        result.code,
        r#"declare module "./App.vue.ts" {
  export const marker: true;
}
declare module "*.vue" {
  const component: unknown;
  export default component;
}"#
    );
}

#[test]
fn rewrites_declaration_module_declaration_specifiers() {
    let rewriter = ImportRewriter::new();
    let source = r#"declare module "./App.vue.ts" {
  export const marker: true;
}"#;
    let result = rewriter.rewrite_declaration_specifiers(source, SourceType::ts());

    assert_eq!(
        result.code,
        r#"declare module "./App.vue" {
  export const marker: true;
}"#
    );
}

#[test]
fn collects_relative_vue_specifiers_from_module_declarations() {
    let rewriter = ImportRewriter::new();
    let source = r#"declare module "./App.vue" {}
declare module "*.vue" {}"#;

    assert_eq!(
        rewriter.collect_relative_vue_specifiers(source, SourceType::ts()),
        vec!["./App.vue"]
    );
}
