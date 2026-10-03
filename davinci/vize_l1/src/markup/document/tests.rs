use super::{
    DocumentLexicalRefusal, DocumentTokenKind as Kind, DocumentTreePolicy, NativeDocument,
};
use crate::markup::{LexErrorCode, ProfileKind, parse_component};
use alloc::vec::Vec;
use vize_l0::{Allocator, SourceRoot, Span};

fn root(source: &str) -> SourceRoot<'_> {
    SourceRoot::new(source).expect("small original source")
}

#[test]
fn declarations_are_original_document_events_without_component_reprofiling() {
    let source = "é<!DOCTYPE html><DIV>{{ count }}</DIV>";
    let allocator = Allocator::default();
    let document = NativeDocument::lex_in(&allocator, root(source));
    assert!(document.errors().is_empty());
    assert!(core::ptr::eq(document.source().as_ptr(), source.as_ptr()));
    let declaration = document
        .tokens()
        .find(|token| matches!(token.kind(), Kind::Declaration { .. }))
        .expect("declaration");
    assert_eq!(declaration.kind(), Kind::Declaration { terminated: true });
    assert_eq!(declaration.span(), Span::new(2, 17));
    assert_eq!(declaration.source(), Some("<!DOCTYPE html>"));
    assert!(core::ptr::eq(declaration.owner(), &document));
    let completion = document
        .normal_completion()
        .expect("normal lexical completion");
    assert_eq!(completion.profile(), ProfileKind::Document);
    assert!(core::ptr::eq(completion.owner(), &document));
    let component = parse_component(&allocator, source).expect("component source");
    assert!(
        component
            .errors
            .iter()
            .any(|error| error.code == vize_l0::ErrorCode::IncorrectlyOpenedComment)
    );
}

#[test]
fn declaration_start_survives_bogus_comment_and_cdata_prefixes() {
    let allocator = Allocator::default();
    for source in ["before<!-bogus>after", "before<![oops]>after"] {
        let document = NativeDocument::lex_in(&allocator, root(source));
        let declarations: Vec<_> = document
            .tokens()
            .filter(|token| matches!(token.kind(), Kind::Declaration { .. }))
            .collect();
        assert_eq!(declarations.len(), 1, "{source}");
        assert_eq!(
            declarations[0].kind(),
            Kind::Declaration { terminated: true }
        );
        assert_eq!(declarations[0].span().start, 6);
        assert!(
            declarations[0]
                .source()
                .expect("authored declaration")
                .starts_with("<!")
        );
        assert_eq!(
            document
                .tokens()
                .filter(|token| token.kind() == Kind::Text)
                .map(|token| token.source().expect("original text"))
                .collect::<Vec<_>>(),
            ["before", "after"]
        );
    }
}

#[test]
fn unterminated_declarations_retain_whole_original_frame_and_refuse() {
    let allocator = Allocator::default();
    for source in ["<!", "<!DOCTYPE html", "<!-", "<![CDA"] {
        let document = NativeDocument::lex_in(&allocator, root(source));
        let declaration = document
            .tokens()
            .find(|token| matches!(token.kind(), Kind::Declaration { .. }))
            .expect("unfinished declaration");
        assert_eq!(declaration.kind(), Kind::Declaration { terminated: false });
        assert_eq!(declaration.span(), Span::new(0, source.len() as u32));
        assert_eq!(declaration.source(), Some(source));
        assert_eq!(
            document
                .normal_completion()
                .expect_err("no normal completion"),
            DocumentLexicalRefusal::UnterminatedDeclaration
        );
    }
}

#[test]
fn complete_comments_and_cdata_never_become_declarations() {
    let allocator = Allocator::default();
    let document = NativeDocument::lex_in(&allocator, root("<!--ok--><![CDATA[value]]>"));
    assert!(
        !document
            .tokens()
            .any(|token| matches!(token.kind(), Kind::Declaration { .. }))
    );
    assert_eq!(
        document
            .tokens()
            .map(|token| (token.kind(), token.source()))
            .collect::<Vec<_>>(),
        [(Kind::Comment, Some("ok")), (Kind::Cdata, Some("value"))]
    );
    assert!(document.normal_completion().is_ok());
}

#[test]
fn entities_keep_one_authored_event_and_original_bytes() {
    let source = "<div title='&NotEqualTilde;'>&NotEqualTilde; {{ π }}</div>";
    let allocator = Allocator::default();
    let document = NativeDocument::lex_in(&allocator, root(source));
    let entities: Vec<_> = document
        .tokens()
        .filter(|token| matches!(token.kind(), Kind::TextEntity | Kind::AttributeEntity))
        .collect();
    assert_eq!(entities.len(), 2);
    for token in entities {
        assert_eq!(token.source(), Some("&NotEqualTilde;"));
        assert!(core::ptr::eq(
            token.source().expect("entity").as_ptr(),
            source.as_ptr().wrapping_add(token.span().start as usize)
        ));
    }
    assert_eq!(
        document
            .tokens()
            .find(|token| token.kind() == Kind::Interpolation)
            .expect("interpolation")
            .source(),
        Some(" π ")
    );
}

#[test]
fn malformed_utf8_recovery_keeps_diagnostics_and_checks_point_boundaries() {
    let source = "<x 界";
    let allocator = Allocator::default();
    let document = NativeDocument::lex_in(&allocator, root(source));
    assert_eq!(document.errors().len(), 1);
    assert_eq!(document.errors()[0].code, LexErrorCode::EofInTag);
    assert_eq!(document.errors()[0].offset, source.len() as u32);
    let end = document
        .tokens()
        .find(|token| token.kind() == Kind::OpenTagEnd)
        .expect("original recovered callback");
    assert_eq!(
        end.span(),
        Span::new(source.len() as u32 - 1, source.len() as u32 - 1)
    );
    assert_eq!(end.source(), None);
    assert_eq!(
        document.normal_completion().expect_err("recovered input"),
        DocumentLexicalRefusal::RecoveredSyntax
    );
    assert!(core::ptr::eq(document.source().as_ptr(), source.as_ptr()));
}

#[test]
fn lexical_completion_never_certifies_browser_tree_policies() {
    let allocator = Allocator::default();
    for source in [
        "<MyComp></mycomp>",
        "<div/><span></span>",
        "<p><div></div>",
        "<table>text<tr><td>x</td></tr></table>",
    ] {
        let document = NativeDocument::lex_in(&allocator, root(source));
        assert!(document.normal_completion().is_ok(), "{source}");
        assert_eq!(
            document.unfinished_tree_policies(),
            &[
                DocumentTreePolicy::FoldNameCase,
                DocumentTreePolicy::HtmlSelfClosing,
                DocumentTreePolicy::ImpliedEndTags,
                DocumentTreePolicy::TableContentModel
            ]
        );
    }
    let document = NativeDocument::lex_in(&allocator, root("<MyComp></mycomp>"));
    assert_eq!(
        document
            .tokens()
            .filter(|token| matches!(token.kind(), Kind::OpenTagName | Kind::CloseTagName))
            .map(|token| token.source().expect("original name"))
            .collect::<Vec<_>>(),
        ["MyComp", "mycomp"]
    );
}

#[test]
fn pinned_petite_vue_document_keeps_tags_interpolations_and_raw_script_style() {
    let source = include_str!("../../../tests/fixtures/document/petite-vue-svg.html");
    assert_eq!(source.len(), 2495);
    let allocator = Allocator::default();
    let document = NativeDocument::lex_in(&allocator, root(source));
    assert!(document.normal_completion().is_ok());
    let names: Vec<_> = document
        .tokens()
        .filter(|token| token.kind() == Kind::OpenTagName)
        .map(|token| token.source().expect("authored name"))
        .collect();
    assert_eq!(
        names,
        [
            "script", "div", "svg", "g", "polygon", "circle", "text", "div", "label", "input",
            "span", "button", "form", "input", "button", "pre", "style"
        ]
    );
    assert_eq!(
        document
            .tokens()
            .filter(|token| token.kind() == Kind::Interpolation)
            .map(|token| token.source().expect("authored expression"))
            .collect::<Vec<_>>(),
        [" label ", "stat.label", "stat.value", " stats "]
    );
    let text: Vec<_> = document
        .tokens()
        .filter(|token| token.kind() == Kind::Text)
        .filter_map(|token| token.source())
        .collect();
    assert!(
        text.iter()
            .any(|text| text.contains("if (ty < 0) debugger") && text.contains("`${x},${y}`"))
    );
    assert!(
        text.iter()
            .any(|text| text.contains("fill: #42b983") && text.contains("left: 300px"))
    );
    assert!(
        document
            .tokens()
            .any(|token| token.kind() == Kind::DirectiveName && token.source() == Some("v-scope"))
    );
    assert_eq!(document.source().as_bytes(), source.as_bytes());
    assert_eq!(document.unfinished_tree_policies().len(), 4);
}

#[test]
fn moved_owner_reborrows_original_token_and_completion_without_relexing() {
    let source = "<!DOCTYPE html><div>{{ n }}</div>";
    let allocator = Allocator::default();
    let original = NativeDocument::lex_in(&allocator, root(source));
    let first_count = original.tokens().len();
    let moved = original;
    for _ in 0..2 {
        assert_eq!(moved.tokens().len(), first_count);
        assert!(core::ptr::eq(
            moved
                .normal_completion()
                .expect("same completed run")
                .owner(),
            &moved
        ));
        assert!(
            moved
                .tokens()
                .all(|token| core::ptr::eq(token.owner(), &moved))
        );
    }
}

#[test]
fn silent_eof_states_cannot_mint_normal_lexical_completion() {
    let allocator = Allocator::default();
    for source in ["{{", "<div/", "{", "abc&", "<x a='&"] {
        let document = NativeDocument::lex_in(&allocator, root(source));
        assert!(
            document.errors().is_empty(),
            "silent inherited EOF: {source}"
        );
        assert_eq!(
            document
                .normal_completion()
                .expect_err("pending actual lexer state"),
            DocumentLexicalRefusal::PendingSyntax,
            "unfinished original source: {source}",
        );
        assert_eq!(document.source(), source);
    }
}

#[test]
fn swallowed_declaration_terminators_cannot_mint_normal_completion() {
    let allocator = Allocator::default();
    for source in ["<!>", "<!->", "<!>tail<div>", "<!->tail<div>"] {
        let document = NativeDocument::lex_in(&allocator, root(source));
        assert_eq!(
            document
                .normal_completion()
                .expect_err("original recovered boundary"),
            DocumentLexicalRefusal::RecoveredDeclaration,
            "inherited declaration recovery: {source}",
        );
        let tokens: Vec<_> = document.tokens().collect();
        assert_eq!(tokens.len(), 1);
        assert!(matches!(tokens[0].kind(), Kind::DeclarationRecovery { .. }));
        assert_eq!(tokens[0].source(), Some(source));
        assert_eq!(document.source(), source);
    }
}

#[test]
fn later_normal_declaration_cannot_erase_an_original_recovery() {
    let allocator = Allocator::default();
    let source = "<!>tail<div><!DOCTYPE html>";
    let document = NativeDocument::lex_in(&allocator, root(source));
    assert_eq!(
        document
            .tokens()
            .map(|token| (token.kind(), token.source()))
            .collect::<Vec<_>>(),
        [
            (
                Kind::DeclarationRecovery { terminated: true },
                Some("<!>tail<div>")
            ),
            (
                Kind::Declaration { terminated: true },
                Some("<!DOCTYPE html>")
            ),
        ]
    );
    assert_eq!(
        document
            .normal_completion()
            .expect_err("sticky original recovery"),
        DocumentLexicalRefusal::RecoveredDeclaration
    );
}
