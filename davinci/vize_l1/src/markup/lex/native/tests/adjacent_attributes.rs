use super::{LexErrorCode, QuoteType, TokenEvent, tokenize};

#[test]
fn quoted_values_end_before_an_adjacent_attribute_in_every_quote_pair() {
    for (source, first, second) in [
        (
            "<div a=\"x\"b='y'></div>",
            QuoteType::Double,
            QuoteType::Single,
        ),
        (
            "<div a='x'b=\"y\"></div>",
            QuoteType::Single,
            QuoteType::Double,
        ),
        (
            "<div a=\"x\"b=\"y\"></div>",
            QuoteType::Double,
            QuoteType::Double,
        ),
        (
            "<div a='x'b='y'></div>",
            QuoteType::Single,
            QuoteType::Single,
        ),
    ] {
        let callbacks = tokenize(source);
        assert!(
            callbacks.errors.is_empty(),
            "{source}: {:?}",
            callbacks.errors
        );
        assert_eq!(
            callbacks.events.as_slice(),
            &[
                TokenEvent::OpenTagName(1, 4),
                TokenEvent::AttribName(5, 6),
                TokenEvent::AttribData(8, 9),
                TokenEvent::AttribEnd(first, 9),
                TokenEvent::AttribName(10, 11),
                TokenEvent::AttribData(13, 14),
                TokenEvent::AttribEnd(second, 14),
                TokenEvent::OpenTagEnd(15),
                TokenEvent::CloseTag(18, 21),
                TokenEvent::End,
            ],
            "{source}"
        );
    }
}

#[test]
fn adjacency_does_not_change_unquoted_or_unfinished_value_errors() {
    let unquoted = tokenize("<div a=x=y></div>");
    assert_eq!(
        unquoted.errors.as_slice(),
        &[(LexErrorCode::UnexpectedCharacterInUnquotedAttributeValue, 8)]
    );
    assert_eq!(
        unquoted.events.as_slice(),
        &[
            TokenEvent::OpenTagName(1, 4),
            TokenEvent::AttribName(5, 6),
            TokenEvent::AttribData(7, 10),
            TokenEvent::AttribEnd(QuoteType::Unquoted, 10),
            TokenEvent::OpenTagEnd(10),
            TokenEvent::CloseTag(13, 16),
            TokenEvent::End,
        ]
    );
    let unfinished = tokenize("<div a=\"x");
    assert_eq!(unfinished.errors.as_slice(), &[(LexErrorCode::EofInTag, 9)]);
}

#[test]
fn html_documents_keep_their_whitespace_error_and_the_same_attribute_tokens() {
    use super::{LexOptions, Lexer, TestCallbacks};
    use crate::markup::profile::Document;

    let source = "<div a=\"x\"b='y'></div>";
    let component = tokenize(source);
    let mut lexer =
        Lexer::<Document, _>::new(source, TestCallbacks::default(), LexOptions::default());
    lexer.run();
    let document = lexer.into_sink();
    assert!(component.errors.is_empty());
    assert_eq!(document.events, component.events);
    assert_eq!(
        document.errors.as_slice(),
        &[(LexErrorCode::MissingWhitespaceBetweenAttributes, 10)]
    );
}
