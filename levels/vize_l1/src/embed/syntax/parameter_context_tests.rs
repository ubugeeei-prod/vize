use super::{Embed, EmbedHole, EmbedSource, Grammar, Lang, Shape, parse_once};
use vize_l0::{Allocator, Span};

fn embed(text: &str, shape: Shape, lang: Lang) -> Embed<'_> {
    Embed {
        grammar: Grammar { shape, lang },
        source: EmbedSource::authored(text, Span::new(0, text.len() as u32)).unwrap(),
    }
}

#[test]
fn parameter_await_is_a_precise_context_hole_with_full_owned_observations() {
    let allocator = Allocator::default();
    for lang in [Lang::Js, Lang::Ts] {
        for text in ["x=await y", "x=/*keep*/await y", "{[await y]:x}"] {
            let syntax = parse_once(&allocator, embed(text, Shape::SlotParams, lang));
            assert_eq!(
                syntax.hole(),
                Some(EmbedHole::InvalidParameterContext),
                "{text}"
            );
            assert!(syntax.slot_params().is_none());
            assert_eq!(syntax.source().text(), text);
            // OXC produced no diagnostic. Keep the original empty owner rather
            // than fabricating one or advertising a recovered parameter root.
            assert_eq!(syntax.diagnostics().count(), 0);
            let retained = syntax.into_slot_params().unwrap();
            assert_eq!(retained.hole(), Some(EmbedHole::InvalidParameterContext));
            assert!(retained.parameters().is_none() && retained.rest().is_none());
            if text.contains("keep") {
                let comment = retained.comments().next().unwrap();
                assert_eq!(comment.text().unwrap(), "/*keep*/");
                assert_eq!(comment.decoded_span(), Ok(Span::new(2, 10)));
                assert_eq!(comment.authored_span(), Ok(Span::new(2, 10)));
            }
        }
        for text in ["await", "x=\\u0061wait y", "x=new.target"] {
            let syntax = parse_once(&allocator, embed(text, Shape::SlotParams, lang));
            assert_eq!(syntax.hole(), Some(EmbedHole::Syntax), "{text}");
            let count = syntax.diagnostics().count();
            assert!(count > 0);
            let message = syntax.diagnostics().next().unwrap().message().as_ptr();
            let retained = syntax.into_slot_params().unwrap();
            assert_eq!(retained.diagnostics().count(), count);
            assert_eq!(
                retained.diagnostics().next().unwrap().message().as_ptr(),
                message
            );
            for diagnostic in retained.diagnostics() {
                for label in diagnostic.labels() {
                    let span = label.authored_span().unwrap();
                    assert!(
                        retained.source().span().start <= span.start
                            && span.end <= retained.source().span().end
                    );
                }
            }
        }
    }
}

#[test]
fn parameter_flags_restore_across_nested_bodies_and_later_defaults() {
    let allocator = Allocator::default();
    for lang in [Lang::Js, Lang::Ts] {
        for text in [
            "x=async()=>await y",
            "x={await:1}",
            "{await:x}",
            "x=function(){return new.target}",
        ] {
            let syntax = parse_once(&allocator, embed(text, Shape::SlotParams, lang));
            assert_eq!(syntax.hole(), None, "{text}");
            assert_eq!(syntax.slot_params().unwrap().parameters().len(), 1);
        }
        for text in ["x=async(y=await z)=>y", "x=async()=>await y,z=await y"] {
            let syntax = parse_once(&allocator, embed(text, Shape::SlotParams, lang));
            assert_eq!(
                syntax.hole(),
                Some(EmbedHole::InvalidParameterContext),
                "{text}"
            );
            assert!(syntax.slot_params().is_none());
        }
        let syntax = parse_once(
            &allocator,
            embed("return async(x=await y)=>x", Shape::HandlerBody, lang),
        );
        assert_eq!(syntax.hole(), Some(EmbedHole::InvalidParameterContext));
        assert!(syntax.handler_body().is_none());
        let syntax = parse_once(
            &allocator,
            embed("return async()=>await y", Shape::HandlerBody, lang),
        );
        assert_eq!(syntax.hole(), None);
        assert!(syntax.handler_body().is_some());
    }
}

#[test]
fn parameter_context_preserves_authored_offsets_and_existing_module_refusal() {
    let allocator = Allocator::default();
    let authored = "zzx=/*keep*/await yzz";
    let source = EmbedSource::authored(authored, Span::new(2, authored.len() as u32 - 2)).unwrap();
    let syntax = parse_once(
        &allocator,
        Embed {
            grammar: Grammar {
                shape: Shape::SlotParams,
                lang: Lang::Ts,
            },
            source,
        },
    );
    let comment = syntax.comments().next().unwrap().text().unwrap().as_ptr();
    let retained = syntax.into_slot_params().unwrap();
    assert_eq!(retained.hole(), Some(EmbedHole::InvalidParameterContext));
    assert_eq!(retained.source().span(), source.span());
    assert_eq!(
        retained.comments().next().unwrap().authored_span(),
        Ok(Span::new(4, 12))
    );
    assert_eq!(
        retained.comments().next().unwrap().text().unwrap().as_ptr(),
        comment
    );
    let syntax = parse_once(
        &allocator,
        embed("x=()=>{import 'm'}", Shape::SlotParams, Lang::Js),
    );
    assert_eq!(syntax.hole(), Some(EmbedHole::InvalidModuleContext));
    assert!(syntax.slot_params().is_none());
}
