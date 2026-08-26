use vize_atelier_core::{TransformOptions, parse, transform};

fn transform_errors(source: &str) -> std::vec::Vec<vize_atelier_core::CompilerError> {
    let allocator = vize_s0::Allocator::new();
    let (mut root, errors) = parse(&allocator, source);
    assert!(errors.is_empty(), "Parse errors: {:?}", errors);
    transform(&allocator, &mut root, TransformOptions::default(), None)
}

#[test]
fn conditional_named_slot_does_not_count_as_implicit_default_child() {
    let errors =
        transform_errors(r#"<Comp><template v-if="show" #footer>Footer</template></Comp>"#);
    assert!(errors.is_empty(), "Unexpected errors: {:?}", errors);
}

#[test]
fn looped_named_slot_does_not_count_as_implicit_default_child() {
    let errors = transform_errors(
        r#"<Comp><template v-for="name in names" #[name]>Named</template></Comp>"#,
    );
    assert!(errors.is_empty(), "Unexpected errors: {:?}", errors);
}
