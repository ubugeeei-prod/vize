use super::super::{
    Tokenizer,
    tests::{TestCallbacks, TokenEvent},
};
use super::scan_argument;
use vize_relief::ErrorCode;
use vize_s0::cstr;

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
    ] {
        let source = cstr!("<Child v-model:[{argument}].trim=\"value\"/>");
        let mut tokenizer = Tokenizer::new(&source, TestCallbacks::default());
        tokenizer.tokenize();
        assert!(tokenizer.callbacks.errors.is_empty(), "{source}");
        let args: Vec<_> = tokenizer
            .callbacks
            .events
            .iter()
            .filter_map(|event| match event {
                TokenEvent::DirArg(start, end) => Some(&source[*start..*end]),
                _ => None,
            })
            .collect();
        assert_eq!(args, [argument], "{source}");
        assert!(tokenizer.callbacks.events.iter().any(|event| matches!(
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
        ] {
            let source = cstr!("<Child :[{argument}{suffix}");
            let mut tokenizer = Tokenizer::new(&source, TestCallbacks::default());
            tokenizer.tokenize();
            let boundary = "<Child :[".len() + argument.len();
            assert!(
                tokenizer
                    .callbacks
                    .errors
                    .contains(&(ErrorCode::MissingDynamicDirectiveArgumentEnd, boundary)),
                "{source}: {:?}",
                tokenizer.callbacks.errors
            );
            let args: Vec<_> = tokenizer
                .callbacks
                .events
                .iter()
                .filter_map(|event| match event {
                    TokenEvent::DirArg(start, end) => Some(&source[*start..*end]),
                    _ => None,
                })
                .collect();
            assert_eq!(args, [argument], "{source}");
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
