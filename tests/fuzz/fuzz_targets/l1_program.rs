#![no_main]

// Native whole-Program admission: no expression scaffold, PURE skip or catch.
// One selector byte chooses an explicit source profile before one real parse.
use libfuzzer_sys::fuzz_target;
use vize_l0::{Allocator, Span};
use vize_l1::embed::syntax::{ProgramGoal, ProgramOptions, parse_program_once};
use vize_l1::embed::{EmbedSource, Lang};

fuzz_target!(|data: &[u8]| {
    let Some((&selector, bytes)) = data.split_first() else {
        return;
    };
    let Ok(input) = std::str::from_utf8(bytes) else {
        return;
    };
    let Ok(end) = u32::try_from(input.len()) else {
        return;
    };
    let options = ProgramOptions {
        lang: if selector & 1 == 0 {
            Lang::Js
        } else {
            Lang::Ts
        },
        jsx: selector & 2 != 0,
        goal: if selector & 4 == 0 {
            ProgramGoal::Module
        } else {
            ProgramGoal::Script
        },
    };
    let allocator = Allocator::default();
    let source = EmbedSource::authored(input, Span::new(0, end)).unwrap();
    let syntax = parse_program_once(&allocator, source, options);
    assert_eq!(syntax.source().text(), input);
    assert_eq!(syntax.source_type(), options.source_type());
    for comment in syntax.comments() {
        assert!(comment.text().is_ok());
        assert!(comment.authored_span().is_ok());
    }
    for diagnostic in syntax.diagnostics() {
        assert!(!diagnostic.message().is_empty());
        for label in diagnostic.labels() {
            let span = label.authored_span().unwrap();
            assert!(span.start <= span.end && span.end <= end);
            assert!(input.is_char_boundary(span.start as usize));
            assert!(input.is_char_boundary(span.end as usize));
        }
    }
    if let Some(program) = syntax.program() {
        assert!(syntax.hole().is_none());
        assert_eq!(program.source_text, input);
        assert_eq!(program.source_type, options.source_type());
    }
    // Full owned diagnostic storage drops before the shared syntax arena.
    drop(syntax);
});
