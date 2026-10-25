//! Synthetic placeholders must not resolve an authored `undefined` binding.
use super::{TemplateGlobal, VirtualTsOptions, VueVersion, generate_template_context};
use oxc_allocator::Allocator;
use oxc_ast::ast::{Expression, IdentifierReference, VariableDeclarator};
use oxc_ast_visit::{Visit, walk::walk_variable_declarator};
use oxc_parser::Parser;
use oxc_span::SourceType;
use oxc_syntax::operator::UnaryOperator;

#[derive(Default)]
struct ContextAst {
    undefined_references: usize,
    operator_placeholders: usize,
}

impl<'a> Visit<'a> for ContextAst {
    fn visit_identifier_reference(&mut self, reference: &IdentifierReference<'a>) {
        self.undefined_references += usize::from(reference.name == "undefined");
    }

    fn visit_variable_declarator(&mut self, declarator: &VariableDeclarator<'a>) {
        if let Some(initializer) = &declarator.init
            && let Expression::TSAsExpression(_) = initializer
            && let Expression::UnaryExpression(unary) = initializer.get_inner_expression()
            && unary.operator == UnaryOperator::Void
        {
            assert!(
                matches!(&unary.argument, Expression::NumericLiteral(value) if value.value == 0.0)
            );
            self.operator_placeholders += 1;
        }
        walk_variable_declarator(self, declarator);
    }
}

struct ContextCase {
    name: String,
    text: String,
    has_own_slots: bool,
}

fn contexts() -> Vec<ContextCase> {
    let mut contexts = Vec::new();
    for dialect in [VueVersion::V3, VueVersion::V2, VueVersion::V2_7] {
        for strict in [false, true] {
            for rich in [false, true] {
                let options = VirtualTsOptions {
                    strict_instance_globals: strict,
                    template_globals: if rich {
                        vec![TemplateGlobal {
                            name: "$plugin".into(),
                            type_annotation: "string | undefined".into(),
                            default_value: "undefined".into(),
                        }]
                    } else {
                        Vec::new()
                    },
                    css_modules: if rich {
                        vec!["$style".into()]
                    } else {
                        Vec::new()
                    },
                    ..Default::default()
                };
                let text = generate_template_context(
                    &options,
                    dialect,
                    false,
                    rich,
                    (
                        rich.then_some("{ value: string | undefined }"),
                        rich.then_some("{ node: Element | undefined }"),
                        rich.then_some("(void 0) as unknown as (...args: unknown[]) => void"),
                    ),
                );
                if rich {
                    let attrs: Vec<_> = text
                        .lines()
                        .filter_map(|line| line.strip_prefix("    const $attrs = "))
                        .collect();
                    let refs: Vec<_> = text
                        .lines()
                        .filter_map(|line| line.strip_prefix("    const $refs = "))
                        .collect();
                    assert_eq!(
                        attrs,
                        ["(void 0) as unknown as { value: string | undefined };"]
                    );
                    assert_eq!(
                        refs,
                        ["(void 0) as unknown as { node: Element | undefined };"]
                    );
                }
                contexts.push(ContextCase {
                    name: format!("{dialect:?}-strict{strict}-rich{rich}"),
                    text: text.to_string(),
                    has_own_slots: rich,
                });
            }
        }
    }
    contexts
}

#[test]
fn every_context_option_uses_operator_placeholders_without_undefined_lookup() {
    for ContextCase {
        name,
        text: context,
        has_own_slots,
    } in contexts()
    {
        let caller = if has_own_slots {
            "const $slots: Record<string, unknown> = {};\n"
        } else {
            ""
        };
        let source = format!(
            "export {{}};\nfunction template() {{\n{caller}{context}\nconst undefined = 'authored';\n}}\n"
        );
        let allocator = Allocator::default();
        let parsed = Parser::new(&allocator, &source, SourceType::ts()).parse();
        assert!(
            parsed.diagnostics.is_empty(),
            "{name}: {:?}",
            parsed.diagnostics
        );
        let mut ast = ContextAst::default();
        ast.visit_program(&parsed.program);
        assert_eq!(ast.undefined_references, 0, "{name}: {context}");
        assert!(ast.operator_placeholders > 0, "{name}: {context}");
        eprintln!(
            "context={name}, emitted-bytes={}, typed-operator-placeholders={}, undefined-value-references={}",
            context.len(),
            ast.operator_placeholders,
            ast.undefined_references
        );
    }
}

#[test]
fn embedded_model_placeholder_keeps_its_type_assertion_without_a_value_lookup() {
    let helper = crate::virtual_ts::helpers::setup_macros::SETUP_MACRO_HELPERS
        .iter()
        .find(|helper| helper.name == "defineModel")
        .unwrap();
    let source = format!("{}\nconst undefined = 'authored';\n", helper.embedded);
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, &source, SourceType::ts()).parse();
    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    let mut ast = ContextAst::default();
    ast.visit_program(&parsed.program);
    assert_eq!(
        (ast.operator_placeholders, ast.undefined_references),
        (1, 0)
    );
    assert_eq!(
        helper.embedded,
        "  const defineModel = (void 0) as unknown as typeof import('vue') extends { defineModel: infer __Macro } ? __Macro : never;\n"
    );
}

#[test]
#[ignore = "requires both real physical native providers and the published Vue graph"]
fn actual_native_context_options_keep_whole_clean_and_authored_error_packets() {
    use std::{fs, path::PathBuf, process::Command};
    let native = PathBuf::from(
        std::env::var_os("VIZE_TEST_CONTEXT_UNDEFINED_NATIVE")
            .expect("actual native 7.0.2 required"),
    );
    let beta = PathBuf::from(
        std::env::var_os("VIZE_TEST_CONTENT_MAPPER_TSGO").expect("actual native beta required"),
    );
    let vue = PathBuf::from(
        std::env::var_os("VIZE_TEST_CONTENT_MAPPER_VUE").expect("published Vue graph required"),
    );
    let output_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/vize-tests/typed-context-undefined-options-10667");
    fs::create_dir_all(&output_root).unwrap();
    let mut observations = Vec::new();
    for ContextCase {
        name,
        text: context,
        has_own_slots,
    } in contexts()
    {
        for invalid in [false, true] {
            let root = output_root.join(format!("{name}-invalid{invalid}"));
            fs::create_dir_all(root.join("node_modules")).unwrap();
            let link = root.join("node_modules/vue");
            if !link.exists() {
                #[cfg(unix)]
                std::os::unix::fs::symlink(&vue, &link).unwrap();
                #[cfg(windows)]
                std::os::windows::fs::symlink_dir(&vue, &link).unwrap();
            }
            let error = if invalid {
                "const bad: number = undefined;\nvoid bad;\n"
            } else {
                ""
            };
            // `has_own_slots` is a caller-owned binding contract. Supply that
            // original binding instead of asking the context emitter to invent it.
            let caller = if has_own_slots {
                "const $slots: Record<string, unknown> = {};\n"
            } else {
                ""
            };
            let source = format!(
                "export {{}};\nfunction template() {{\n{caller}{context}\nconst undefined = 'authored';\n{error}}}\n"
            );
            fs::write(root.join("Context.ts"), &source).unwrap();
            let expected = if invalid {
                let marker = source.find("const bad: number").unwrap() + "const ".len();
                let before = source.get(..marker).unwrap();
                let line = before.bytes().filter(|b| *b == b'\n').count() + 1;
                let column = before.rsplit('\n').next().unwrap().encode_utf16().count() + 1;
                format!(
                    "Context.ts({line},{column}): error TS2322: Type 'string' is not assignable to type 'number'.\n"
                )
            } else {
                String::new()
            };
            for (host, binary) in [("native-702", &native), ("native-beta", &beta)] {
                let argv = [
                    "--noEmit",
                    "--strict",
                    "--target",
                    "es2022",
                    "--module",
                    "esnext",
                    "--moduleResolution",
                    "bundler",
                    "--skipLibCheck",
                    "--pretty",
                    "false",
                    "--ignoreConfig",
                    "Context.ts",
                ];
                let result = Command::new(binary)
                    .current_dir(&root)
                    .args(argv)
                    .output()
                    .unwrap();
                fs::write(root.join(format!("{host}.stdout.raw")), &result.stdout).unwrap();
                fs::write(root.join(format!("{host}.stderr.raw")), &result.stderr).unwrap();
                fs::write(root.join(format!("{host}.json")), serde_json::to_vec_pretty(&serde_json::json!({"schema":"vize.typed-context-options-host","version":1,"host":host,"argv":argv,"binary":binary,"source":source,"status":result.status.code(),"stdout":result.stdout,"stderr":result.stderr})).unwrap()).unwrap();
                observations.push((
                    format!("{name}-invalid{invalid}/{host}"),
                    result,
                    expected.clone(),
                    if !invalid {
                        0
                    } else if host == "native-702" {
                        1
                    } else {
                        2
                    },
                ));
            }
        }
    }
    assert_eq!(observations.len(), 48);
    for (name, result, expected, status) in observations {
        assert_eq!(result.status.code(), Some(status), "{name}: {result:?}");
        assert!(result.stderr.is_empty(), "{name}: {:?}", result.stderr);
        assert_eq!(
            String::from_utf8(result.stdout).unwrap(),
            expected,
            "{name}: entire stdout"
        );
    }
}

#[test]
fn options_api_setup_return_preserves_its_authored_undefined_shadow() {
    let script =
        "export default { setup() { const undefined = 'authored'; return { undefined }; } };";
    let allocator = vize_carton::Allocator::new();
    let (root, _) = vize_armature::parse(&allocator, "<p>{{ undefined }}</p>");
    let mut analyzer = vize_croquis::Analyzer::with_options(vize_croquis::AnalyzerOptions::full())
        .with_options_api();
    analyzer.analyze_script_plain(script);
    analyzer.analyze_template(&root);
    let summary = analyzer.finish();
    let generated = super::generate_virtual_ts_with_offsets_options_api(
        &summary,
        Some(script),
        Some(&root),
        0,
        0,
        &Default::default(),
    );
    let shadows: Vec<_> = generated
        .code
        .lines()
        .map(str::trim)
        .filter(|line| line.starts_with("var undefined:"))
        .collect();
    assert_eq!(
        shadows,
        ["var undefined: __U<__R_undefined> = (void 0) as any;"]
    );
    let source = shadows.join("\n");
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, &source, SourceType::ts()).parse();
    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    let mut ast = ContextAst::default();
    ast.visit_program(&parsed.program);
    assert_eq!(
        (ast.operator_placeholders, ast.undefined_references),
        (1, 0)
    );
}
