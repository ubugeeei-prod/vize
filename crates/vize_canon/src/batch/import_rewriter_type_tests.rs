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
