use super::{
    Allocator, NoLinks, Recorded, build_native_dom_file_decisions, completed, emit_template, on,
};
use vize_l3::decision::dom::DomUnsupported;
use vize_l4::targets::dom::DomErrorKind;

#[test]
fn syntax_access_name_and_header_order_refusals_keep_exact_original_windows() {
    for (attribute, reason, window) in [
        (
            "@click='return $event;'",
            DomUnsupported::HandlerSyntax,
            "return $event;",
        ),
        (
            "@click='var unused=0; if($event){return 1;}'",
            DomUnsupported::HandlerSyntax,
            "return 1;",
        ),
        (
            "@click='var x=$event; x.count++;'",
            DomUnsupported::HandlerAccess,
            "x",
        ),
        (
            "@click='var unused=0; //kept'",
            DomUnsupported::HandlerSyntax,
            "var unused=0; //kept",
        ),
        (
            "@click='var unused=0; $eve&#110;t.count++;'",
            DomUnsupported::HandlerAccess,
            "$eve&#110;t",
        ),
        (
            "@click='var unused=0;'",
            DomUnsupported::HandlerAccess,
            "var unused=0;",
        ),
        (
            "@click='$event.count++;'",
            DomUnsupported::HandlerSyntax,
            "$event.count++;",
        ),
        (
            "@keyup='var unused=0; $event.count++;'",
            DomUnsupported::BindingName,
            "@keyup='var unused=0; $event.count++;'",
        ),
        (
            "@click='var unused=0; $event.count++;' id='late'",
            DomUnsupported::HandlerSyntax,
            "id='late'",
        ),
        (
            "onClick='earlier' @click='var unused=0; $event.count++;'",
            DomUnsupported::DuplicateProperty,
            "@click='var unused=0; $event.count++;'",
        ),
    ] {
        let arena = Allocator::default();
        let source = format!("<!--雪🌸--><template><button {attribute}/></template>");
        let owner = completed(&arena, &source);
        let analysis = build_native_dom_file_decisions(owner.view().unwrap()).unwrap();
        let file = analysis.file();
        let original_on = on(file.artifact().root());
        let lower = file.handler_for(original_on).unwrap();
        let resolution = lower.resolution().unwrap();
        let body = resolution.input().body();
        let row = analysis.handler(lower.id().node()).unwrap();
        assert!(core::ptr::eq(row.resolution().input().body(), body));
        assert!(row.accepts_on(original_on));
        assert!(core::ptr::eq(
            resolution
                .input()
                .operand()
                .syntax()
                .source()
                .authored_root(),
            source.as_str()
        ));
        let recorded = emit_template::<Recorded>(&analysis).unwrap_err();
        let plain = emit_template::<NoLinks>(&analysis).unwrap_err();
        assert_eq!(recorded, plain);
        assert_eq!(recorded.kind, DomErrorKind::Unsupported(reason));
        assert_eq!(recorded.node, Some(lower.id().node()));
        assert_eq!(recorded.span.slice(&source), window);
        assert!(core::ptr::eq(row.resolution(), resolution));
    }
}
