use super::{Allocator, check, complete, owner};
use vize_l2::file::{DeclarationKind, InitializerKind};

#[test]
fn original_decoded_primitive_strings_keep_selected_setup_and_exact_value_class()
-> Result<(), &'static str> {
    for (literal, expected) in [
        (r#"'a\0b'"#, InitializerKind::PrimitiveStringWithNulOrCr),
        (r#"'a\x00b'"#, InitializerKind::PrimitiveStringWithNulOrCr),
        (r#"'a\u0000b'"#, InitializerKind::PrimitiveStringWithNulOrCr),
        (r#"'a\rb'"#, InitializerKind::PrimitiveStringWithNulOrCr),
        (r#"'a\x0db'"#, InitializerKind::PrimitiveStringWithNulOrCr),
        (r#"'a\u000db'"#, InitializerKind::PrimitiveStringWithNulOrCr),
        (r#"'a\r\nb'"#, InitializerKind::PrimitiveStringWithNulOrCr),
        (
            r#"'a\ud800b'"#,
            InitializerKind::PrimitiveStringWithLoneSurrogates,
        ),
        (
            r#"'a\udc00b'"#,
            InitializerKind::PrimitiveStringWithLoneSurrogates,
        ),
        (
            r#"'a\ud800\0b'"#,
            InitializerKind::PrimitiveStringWithLoneSurrogates,
        ),
        (r#"'a\nb'"#, InitializerKind::PrimitiveLiteral),
        (r#"'a\\0b'"#, InitializerKind::PrimitiveLiteral),
        (r#"'a\\rb'"#, InitializerKind::PrimitiveLiteral),
        ("'雪🌸'", InitializerKind::PrimitiveLiteral),
        (r#"'a\ud83c\udf38b'"#, InitializerKind::PrimitiveLiteral),
        (r#"'a\ufffdb'"#, InitializerKind::PrimitiveLiteral),
    ] {
        for (kind, declaration) in [
            (DeclarationKind::Const, "const"),
            (DeclarationKind::Let, "let"),
            (DeclarationKind::Var, "var"),
        ] {
            for (header, annotation) in [("", ""), (" lang=ts", ":string")] {
                let arena = Allocator::default();
                let source = format!(
                    "<script setup{header}>/* same original */ {declaration} value{annotation}={literal};</script><template><div/></template>"
                );
                let mut original = owner(&arena, &source)?;
                let unit = original.parse_setup_program().map_err(|_| "actual setup")?;
                let original = complete(original)?;
                let setup = original.setup().map_err(|_| "selected primitive setup")?;
                check(core::ptr::eq(setup.file(), original.file().ok_or("File")?))?;
                let mut bindings = setup.bindings();
                let binding = bindings.next().ok_or("actual declaration")?;
                check(bindings.next().is_none())?;
                let row = binding.declaration().ok_or("retained declaration")?;
                check(row.initializer == expected && row.initializer.is_primitive())?;
                check(row.kind == kind && row.unit == unit && row.scope == setup.scope())?;
                check(row.is_direct_program() && row.name == "value")?;
                check(setup.binding(binding).is_ok())?;
                check(setup.type_annotations().count() == usize::from(!annotation.is_empty()))?;
            }
        }
    }
    Ok(())
}

#[test]
fn original_comments_and_escape_spellings_do_not_replace_decoded_value_evidence()
-> Result<(), &'static str> {
    let arena = Allocator::default();
    let source =
        r#"<script setup>/* '\0' '\r' */ const value=1;</script><template><div/></template>"#;
    let mut original = owner(&arena, source)?;
    original.parse_setup_program().map_err(|_| "actual setup")?;
    let original = complete(original)?;
    let setup = original.setup().map_err(|_| "selected primitive setup")?;
    let binding = setup.bindings().next().ok_or("actual declaration")?;
    check(binding.declaration().ok_or("row")?.initializer == InitializerKind::PrimitiveLiteral)?;
    check(!InitializerKind::Unknown.is_primitive())?;
    check(!InitializerKind::Function.is_primitive())?;
    Ok(())
}
