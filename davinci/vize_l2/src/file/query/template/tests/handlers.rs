use super::{
    Allocator, PositionQueryError, TemplateQueryError, TemplateSiteScope, TemplateSymbolRef, at,
    file, native, site, uses,
};

#[test]
fn original_handler_locals_keep_shadowing_and_actual_declaration_site_scopes() {
    let arena = Allocator::default();
    let source = "<script setup>const value=2;</script><template><button @click='let value=$event; {let value=3; value;} value;' @blur='value'/></template>";
    let output = native(&arena, source);
    let file = file(&output);
    let declaration = site(file, source, "value=$event");
    let nested = site(file, source, "value=3");
    let outer_use = site(file, source, "value;' @blur");
    let inner_use = site(file, source, "value;} value");
    assert!(declaration.usage().is_none() && nested.usage().is_none());
    assert!(declaration.symbol().same_symbol(outer_use.symbol()));
    assert!(nested.symbol().same_symbol(inner_use.symbol()));
    assert!(!declaration.symbol().same_symbol(nested.symbol()));
    assert_eq!(uses(file, declaration.symbol()), [outer_use.span()]);
    assert_eq!(uses(file, nested.symbol()), [inner_use.span()]);
    let TemplateSymbolRef::HandlerLocal(local) = declaration.symbol() else {
        panic!("local")
    };
    assert_eq!(local.binding().name, "value");
    assert_eq!(local.declarations().count(), 1);
    let TemplateSiteScope::Handler(handler, scope) = declaration.scope() else {
        panic!("handler scope")
    };
    assert!(handler.same_owner(local.handler()));
    assert_eq!(scope, local.declarations().next().unwrap().scope);
    let blur = site(file, source, "value'/></template>");
    let TemplateSymbolRef::File(binding) = blur.symbol() else {
        panic!("outer File binding")
    };
    assert_eq!(binding.declaration().unwrap().span.slice(source), "value");
    assert!(!blur.symbol().same_symbol(declaration.symbol()));
    let event = site(file, source, "$event;");
    let TemplateSymbolRef::HandlerLocal(event) = event.symbol() else {
        panic!("event local")
    };
    assert!(event.declarations().next().is_none());
    assert_eq!(event.binding().name, "$event");
    assert!(core::ptr::eq(declaration.file(), file));
}

#[test]
fn original_handler_entity_spelling_half_open_utf8_and_utf16_are_preserved() {
    let arena = Allocator::default();
    let source =
        "<template><button @click='/*kept\r\n😀*/ let café=$event; return caf&#233;'/></template>";
    let output = native(&arena, source);
    let file = file(&output);
    let declaration = site(file, source, "café=");
    let reference = site(file, source, "caf&#233;");
    assert!(declaration.symbol().same_symbol(reference.symbol()));
    assert_eq!(reference.span().slice(source), "caf&#233;");
    for byte in reference.span().start..reference.span().end {
        assert!(
            file.template_symbol_at_offset(byte)
                .unwrap()
                .unwrap()
                .symbol()
                .same_symbol(reference.symbol())
        );
    }
    assert!(
        file.template_symbol_at_offset(reference.span().end)
            .unwrap()
            .is_none()
    );
    assert!(matches!(
        file.template_symbol_at_offset(declaration.span().start + 4),
        Err(TemplateQueryError::Position(
            PositionQueryError::NotCharBoundary
        ))
    ));
    assert!(
        file.template_symbol_at_offset(source.len() as u32)
            .unwrap()
            .is_none()
    );
    assert!(
        file.template_symbol_at_offset(at(source, "button"))
            .unwrap()
            .is_none()
    );
    let (line, column) =
        vize_l0::line_index::LineIndex::new(source).line_col(reference.span().start as usize);
    assert_eq!(line, 1);
    let line_start = source.find('\n').unwrap() + 1;
    assert_eq!(
        column as usize,
        source[line_start..reference.span().start as usize]
            .encode_utf16()
            .count()
    );
    assert!(
        source[line_start..reference.span().start as usize]
            .chars()
            .count()
            < column as usize
    );
}

#[test]
fn original_var_site_scope_remains_distinct_from_its_hoisted_binding() {
    let arena = Allocator::default();
    let source = "<template><button @click='{var value=$event;} return value'/></template>";
    let output = native(&arena, source);
    let file = file(&output);
    let declaration = site(file, source, "value=$event");
    let reference = site(file, source, "value'/>");
    let TemplateSymbolRef::HandlerLocal(local) = declaration.symbol() else {
        panic!("var")
    };
    let TemplateSiteScope::Handler(_, site_scope) = declaration.scope() else {
        panic!("scope")
    };
    assert_ne!(site_scope, local.binding().scope);
    assert_eq!(site_scope, local.declarations().next().unwrap().scope);
    assert!(declaration.symbol().same_symbol(reference.symbol()));
}
