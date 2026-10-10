use super::super::{
    LexOptions,
    tests::{TokenEvent, lex_with},
};
use super::scan_argument;
use crate::markup::token::LexErrorCode;
use vize_l0::{SmallVec, cstr};

#[test]
fn emits_one_complete_argument_and_preserves_modifiers() {
    for argument in [
        "keys[index]",
        "keys[indices[index]]",
        "keys[']']",
        "keys['[']",
        r#"keys["\"]"]"#,
        "keys['a.b']",
        "`prefix${keys['[']}`",
        "`prefix${`nested${keys[']']}`}`",
        "`escaped\\`][${keys[index]}`",
        "(slot=>slot)(outside)",
        "(other=>slot)(outside)",
    ] {
        let source = cstr!("<Child v-model:[{argument}].trim=\"value\"/>");
        let tokenizer = lex_with(&source, LexOptions::default());
        assert!(tokenizer.errors.is_empty(), "{source}");
        let args: SmallVec<[&str; 2]> = tokenizer
            .events
            .iter()
            .filter_map(|event| match event {
                TokenEvent::DirArg(start, end) => Some(&source[*start..*end]),
                _ => None,
            })
            .collect();
        assert_eq!(args.as_slice(), [argument], "{source}");
        assert!(tokenizer.events.iter().any(|event| matches!(
            event, TokenEvent::DirModifier(start, end) if &source[*start..*end] == "trim"
        )));
    }
}

#[test]
fn boundaries_and_unterminated_literals_recover_without_swallowing_markup() {
    for suffix in ["=\"value\"", " other", "/>", ">", "\nother", ""] {
        for argument in [
            "keys[index]",
            "keys['broken",
            "keys[\"broken",
            "`broken",
            "keys['escaped\\",
            "(slot=>slot)(outside)",
        ] {
            let source = cstr!("<Child :[{argument}{suffix}");
            let tokenizer = lex_with(&source, LexOptions::default());
            let boundary = "<Child :[".len() + argument.len();
            assert!(
                tokenizer
                    .errors
                    .contains(&(LexErrorCode::MissingDynamicDirectiveArgumentEnd, boundary)),
                "{source}: {:?}",
                tokenizer.errors
            );
            let args: SmallVec<[&str; 2]> = tokenizer
                .events
                .iter()
                .filter_map(|event| match event {
                    TokenEvent::DirArg(start, end) => Some(&source[*start..*end]),
                    _ => None,
                })
                .collect();
            assert_eq!(args.as_slice(), [argument], "{source}");
        }
    }
}

#[test]
fn scanning_is_iterative_and_returns_the_outer_boundary() {
    let argument = cstr!("{}key{}", "[".repeat(20_000), "]".repeat(20_000));
    let source = cstr!("{argument}].trim=\"value\"");
    assert_eq!(scan_argument(source.as_bytes(), 0), (argument.len(), true));
    let source = cstr!("{argument}=\"value\"");
    assert_eq!(scan_argument(source.as_bytes(), 0), (argument.len(), false));
}
