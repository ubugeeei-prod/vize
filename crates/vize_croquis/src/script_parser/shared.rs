//! One parse per script text and dialect while one SFC is projected.
//!
//! Projecting an SFC reads its script through many independent analyses, and
//! most of them parse the same text with the same [`SourceType`]. Inside a
//! [`SharedParseScope`], [`with_shared_parse`] parses each distinct
//! `(text, source type)` once and lends every later reader that same
//! [`ParserReturn`]. Without a scope it parses privately, as before.
//!
//! Readers only borrow the parse, so they cannot change the tree. The one
//! state a reader can write is the ids an `oxc_semantic` build stores in AST
//! cells. Every reader that reads ids builds semantic first, and a build
//! assigns the same ids to the same tree, so a shared tree answers each reader
//! exactly as a private one would.

#![expect(
    clippy::disallowed_types,
    reason = "a lent parse outlives the scope's borrow of its list, so a nested read can add one"
)]

use std::cell::RefCell;
use std::rc::Rc;

use oxc_allocator::Allocator;
use oxc_parser::{Parser, ParserReturn};
use oxc_span::SourceType;

/// The text a shared parse borrows, with the arena its tree lives in.
struct Source {
    allocator: Allocator,
    text: Box<str>,
    source_type: SourceType,
}

self_cell::self_cell!(
    struct SharedParse {
        owner: Source,

        #[not_covariant]
        dependent: ParserReturn,
    }
);

thread_local! {
    /// The current projection's parses. Projections are synchronous on their
    /// thread, like the bindings table in Canon's script facts.
    static PARSES: RefCell<Option<Vec<Rc<SharedParse>>>> = const { RefCell::new(None) };
}

/// Shares parses until dropped. A scope opened inside another one joins it.
#[must_use = "parses are shared only while the scope is alive"]
pub struct SharedParseScope {
    owns: bool,
}

impl SharedParseScope {
    /// Share parses on this thread until the returned scope is dropped.
    pub fn enter() -> Self {
        let owns = PARSES.with_borrow_mut(|parses| {
            if parses.is_some() {
                return false;
            }
            *parses = Some(Vec::new());
            true
        });
        Self { owns }
    }
}

impl Drop for SharedParseScope {
    fn drop(&mut self) {
        if self.owns {
            let parses = PARSES.with_borrow_mut(Option::take);
            drop(parses);
        }
    }
}

/// Read the parse of `text` as `source_type` with default parse options.
///
/// Inside a [`SharedParseScope`], equal text read as the same source type is
/// parsed once; `read` receives that parse. Outside a scope the parse is
/// private to this call.
pub fn with_shared_parse<R>(
    text: &str,
    source_type: SourceType,
    read: impl for<'a> FnOnce(&ParserReturn<'a>) -> R,
) -> R {
    let shared = PARSES.with_borrow(|parses| {
        parses.as_ref().map(|parses| {
            parses
                .iter()
                .find(|parse| {
                    let source = parse.borrow_owner();
                    source.source_type == source_type && *source.text == *text
                })
                .cloned()
        })
    });
    let parse = match shared {
        None => {
            let allocator = Allocator::default();
            let parsed = Parser::new(&allocator, text, source_type).parse();
            return read(&parsed);
        }
        Some(Some(parse)) => parse,
        Some(None) => {
            let source = Source {
                allocator: Allocator::default(),
                text: Box::from(text),
                source_type,
            };
            let parse = Rc::new(SharedParse::new(source, |source| {
                Parser::new(&source.allocator, &source.text, source.source_type).parse()
            }));
            PARSES.with_borrow_mut(|parses| {
                if let Some(parses) = parses {
                    parses.push(Rc::clone(&parse));
                }
            });
            parse
        }
    };
    parse.with_dependent(|_, parsed| read(parsed))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tree_address(text: &str, source_type: SourceType) -> usize {
        with_shared_parse(text, source_type, |parsed| {
            std::ptr::from_ref(&parsed.program) as usize
        })
    }

    #[test]
    fn a_scope_parses_equal_text_and_dialect_once() {
        let _scope = SharedParseScope::enter();
        let text: Box<str> = Box::from("const value = 1;\n");
        let copy: Box<str> = Box::from(&*text);
        assert_eq!(
            tree_address(&text, SourceType::ts()),
            tree_address(&copy, SourceType::ts())
        );
        assert_ne!(
            tree_address(&text, SourceType::ts()),
            tree_address(&text, SourceType::ts().with_module(true))
        );
        assert_ne!(
            tree_address(&text, SourceType::ts()),
            tree_address("const value = 2;\n", SourceType::ts())
        );
    }

    #[test]
    fn a_shared_parse_matches_a_private_one() {
        let text = "import { ref } from 'vue'\nconst count = ref(\n";
        let private = with_shared_parse(text, SourceType::ts(), |parsed| {
            (
                parsed.panicked,
                parsed.diagnostics.len(),
                parsed.program.body.len(),
            )
        });
        let _scope = SharedParseScope::enter();
        for _ in 0..2 {
            let shared = with_shared_parse(text, SourceType::ts(), |parsed| {
                (
                    parsed.panicked,
                    parsed.diagnostics.len(),
                    parsed.program.body.len(),
                )
            });
            assert_eq!(shared, private);
        }
    }

    #[test]
    fn nested_reads_share_and_a_nested_scope_joins_the_outer_one() {
        let _scope = SharedParseScope::enter();
        let outer = tree_address("let a = 1;", SourceType::ts());
        let nested = with_shared_parse("let b = 2;", SourceType::ts(), |_| {
            let _inner = SharedParseScope::enter();
            (
                tree_address("let a = 1;", SourceType::ts()),
                tree_address("let b = 2;", SourceType::ts()),
            )
        });
        assert_eq!(nested.0, outer);
        assert_eq!(nested.1, tree_address("let b = 2;", SourceType::ts()));
        assert_eq!(tree_address("let a = 1;", SourceType::ts()), outer);
    }

    #[test]
    fn dropping_the_scope_stops_sharing() {
        {
            let _scope = SharedParseScope::enter();
            tree_address("let a = 1;", SourceType::ts());
        }
        PARSES.with_borrow(|parses| assert!(parses.is_none()));
    }
}
