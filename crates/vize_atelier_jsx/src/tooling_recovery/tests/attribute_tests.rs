use crate::{JsxLang, lower_source, lower_source_for_typecheck};
use vize_l0::Allocator;

#[test]
fn authored_attributes_and_spreads_retain_complete_native_root_spans() {
    let source = "// 日本語 😀\r\nconst view = <Host separator={choice || <span>{value}</span>} customSlots={{ checkable: () => <i>{count}</i> }} {...{ extra: <b>{value}</b> }} icon=<u>{count}</u> />;";
    let allocator = Allocator::new();
    let ordinary = lower_source(&allocator, allocator.as_oxc(), source, JsxLang::Tsx);
    let native = lower_source_for_typecheck(&allocator, allocator.as_oxc(), source, JsxLang::Tsx);
    assert_eq!(ordinary.diagnostics, native.diagnostics);
    assert!(native.diagnostics.is_empty());
    let spans = |output: &crate::LowerOutput<'_>| {
        output
            .roots
            .iter()
            .map(|root| {
                let span = root.root.loc.span;
                source
                    .get(span.start as usize..span.end as usize)
                    .unwrap()
                    .to_owned()
            })
            .collect::<Vec<_>>()
    };
    let outer = "<Host separator={choice || <span>{value}</span>} customSlots={{ checkable: () => <i>{count}</i> }} {...{ extra: <b>{value}</b> }} icon=<u>{count}</u> />";
    assert_eq!(spans(&ordinary), [outer]);
    assert_eq!(
        spans(&native),
        [
            outer,
            "<span>{value}</span>",
            "<i>{count}</i>",
            "<b>{value}</b>",
            "<u>{count}</u>"
        ]
    );
}

#[test]
fn direct_attribute_fragment_uses_the_same_native_retainer() {
    let source = "const view = <Host icon=<><u>{value}</u></> />;";
    let allocator = Allocator::new();
    let ordinary = lower_source(&allocator, allocator.as_oxc(), source, JsxLang::Tsx);
    let native = lower_source_for_typecheck(&allocator, allocator.as_oxc(), source, JsxLang::Tsx);
    assert_eq!(ordinary.diagnostics, native.diagnostics);
    assert!(native.diagnostics.is_empty());
    let spans: Vec<_> = native
        .roots
        .iter()
        .map(|root| {
            let span = root.root.loc.span;
            source
                .get(span.start as usize..span.end as usize)
                .expect("authored root")
                .to_owned()
        })
        .collect();
    assert_eq!(
        spans,
        ["<Host icon=<><u>{value}</u></> />", "<><u>{value}</u></>"]
    );
}
