use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_atelier_core::{
    CodegenMode, CodegenOptions, TransformOptions, generate, parse, transform,
};
use vize_s0::String;

fn compile_module(source: &str) -> String {
    let allocator = vize_s0::Allocator::new();
    let (mut root, errors) = parse(&allocator, source);
    assert!(errors.is_empty(), "template parse errors: {errors:?}");
    transform(&allocator, &mut root, TransformOptions::default(), None);

    let result = generate(
        &root,
        CodegenOptions {
            mode: CodegenMode::Module,
            ..Default::default()
        },
    );
    let mut output = String::with_capacity(result.preamble.len() + result.code.len() + 1);
    output.push_str(&result.preamble);
    output.push('\n');
    output.push_str(&result.code);
    output
}

#[test]
fn v_once_quotes_non_identifier_prop_keys_and_emits_parseable_javascript() {
    let output = compile_module(r#"<div v-once data-testid="once" :aria-label="label"></div>"#);

    insta::assert_snapshot!(output.as_str());

    let allocator = Allocator::default();
    let parsed = Parser::new(
        &allocator,
        output.as_str(),
        SourceType::default().with_module(true),
    )
    .parse();
    assert!(
        parsed.diagnostics.is_empty(),
        "v-once output must parse as JavaScript: {:?}\n{output}",
        parsed.diagnostics
    );
}
