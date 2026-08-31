use vize_atelier_dom::{Allocator, compile_template};

#[test]
fn standard_compile_emits_nested_anchor_and_button_recoveries() {
    let allocator = Allocator::new();
    for source in [
        r#"<a href="/"><div><a href="/foo">inner</a></div></a>"#,
        "<button><div><button>bbb</button></div></button>",
    ] {
        let (_, errors, result) = compile_template(&allocator, source);
        assert!(
            errors.iter().all(|error| error.is_compatibility_notice()),
            "{source}: nested interactive-content recovery must stay non-fatal without downgrading unrelated errors: {errors:?}"
        );
        assert!(
            !result.code.is_empty(),
            "{source}: compatibility notices must still emit code"
        );
    }
}

#[test]
fn standard_compile_keeps_extra_invalid_anchor_end_tag_fatal() {
    let allocator = Allocator::new();
    let (_, errors, result) = compile_template(
        &allocator,
        r#"<a href="/">outer<a href="/foo">inner</a></a></a>"#,
    );

    assert!(
        errors.iter().any(|error| !error.is_recoverable()),
        "the extra stray </a> must remain a hard parse error: {errors:?}"
    );
    assert!(result.code.is_empty());
}
