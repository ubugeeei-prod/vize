//! Streaming input preprocessing, separated from retained reference expansions.

use crate::markup::document::Event;
use crate::markup::entity::DecodedEntity;

pub(super) fn text<'a>(event: &Event, source: &'a str) -> Scalars<'a> {
    match event.decoded_entity() {
        Some(DecodedEntity::Named(value)) => Scalars::Named(value.chars()),
        Some(DecodedEntity::Numeric(value)) => Scalars::Numeric(Some(value)),
        None => Scalars::Literal {
            value: event.span.slice(source).chars().peekable(),
            comment: false,
        },
    }
}

pub(super) fn comment(raw: &str) -> Scalars<'_> {
    Scalars::Literal {
        value: raw.chars().peekable(),
        comment: true,
    }
}

#[derive(Debug)]
pub(super) enum Scalars<'a> {
    Named(core::str::Chars<'static>),
    Numeric(Option<char>),
    Literal {
        value: core::iter::Peekable<core::str::Chars<'a>>,
        comment: bool,
    },
}

impl Iterator for Scalars<'_> {
    type Item = char;

    fn next(&mut self) -> Option<char> {
        match self {
            Self::Named(value) => value.next(),
            Self::Numeric(value) => value.take(),
            Self::Literal { value, comment } => loop {
                match value.next()? {
                    '\r' => {
                        if value.peek() == Some(&'\n') {
                            value.next();
                        }
                        return Some('\n');
                    }
                    '\0' if *comment => return Some('\u{fffd}'),
                    '\0' => {}
                    ch => return Some(ch),
                }
            },
        }
    }
}
