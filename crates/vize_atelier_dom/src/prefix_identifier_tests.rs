//! Prefix identifier coverage for dynamic DOM keys.

use super::{DomCompilerOptions, compile_template_with_options};
use vize_s0::Allocator;

/// SFC / module-mode compile sets `DomCompilerOptions.prefix_identifiers`, but
/// codegen used to keep the default `false`. Compound dynamic keys then
/// emitted bare identifiers and crashed as `ReferenceError` at render time.
#[test]
fn prefixed_compound_dynamic_bind_and_on_keys_walk_identifiers() {
    let allocator = Allocator::new();
    let options = DomCompilerOptions {
        prefix_identifiers: true,
        ..Default::default()
    };

    let (_, errors, bind) = compile_template_with_options(
        &allocator,
        r#"<div :[prefix+suffix]="value"></div>"#,
        options.clone(),
    );
    assert!(errors.is_empty(), "Errors: {:?}", errors);
    assert!(
        bind.code.contains("[_ctx.prefix+_ctx.suffix || \"\"]"),
        "bind key was not prefixed:\n{}",
        bind.code
    );
    assert!(
        bind.code.contains("_ctx.value"),
        "bind value was not prefixed:\n{}",
        bind.code
    );

    let (_, errors, on) = compile_template_with_options(
        &allocator,
        r#"<button @[prefix+suffix]="handler"></button>"#,
        options,
    );
    assert!(errors.is_empty(), "Errors: {:?}", errors);
    assert!(
        on.code.contains("_toHandlerKey(_ctx.prefix+_ctx.suffix)"),
        "on key was not prefixed:\n{}",
        on.code
    );
}
