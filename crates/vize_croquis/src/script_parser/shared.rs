//! One parse per text and dialect while one SFC is projected.
//!
//! Projecting an SFC reads its script through many independent analyses, and
//! most of them parse the same text with the same [`SourceType`]. Template
//! guards and handlers are likewise parsed again by each reader. Inside a
//! [`SharedParseScope`], [`with_shared_parse`] parses each distinct
//! `(text, source type)` program once and [`with_shared_expression`] each
//! distinct expression once, and every later reader borrows that parse.
//! Without a scope each read parses privately, as before.
//!
//! Readers only borrow the parse, so they cannot change the tree. The one
//! state a reader can write is the ids an `oxc_semantic` build stores in AST
//! cells. Every reader that reads ids builds semantic first, and a build
//! assigns the same ids to the same tree, so a shared tree answers each reader
//! exactly as a private one would.
//!
//! A shared parse owns its text and arena. The parse is stored with its
//! lifetime erased and only lent for a borrow of the value that owns both.

#![expect(
    clippy::disallowed_types,
    reason = "a lent parse outlives the scope's borrow of its list, so a nested read can add one"
)]

use std::cell::RefCell;
use std::hash::BuildHasher;
use std::mem::ManuallyDrop;
use std::ptr::NonNull;
use std::rc::Rc;

use oxc_allocator::Allocator;
use oxc_ast::ast::Expression;
use oxc_parser::{Parser, ParserReturn};
use oxc_span::SourceType;
use rustc_hash::FxBuildHasher;
use vize_carton::FxHashMap;

/// The text a shared parse borrows, with the arena its tree lives in.
struct Source {
    allocator: Allocator,
    text: Box<str>,
    source_type: SourceType,
}

impl Source {
    fn new(text: &str, source_type: SourceType) -> Self {
        Self {
            allocator: Allocator::default(),
            text: Box::from(text),
            source_type,
        }
    }

    fn is(&self, text: &str, source_type: SourceType) -> bool {
        self.source_type == source_type && *self.text == *text
    }
}

trait Shared {
    fn source(&self) -> &Source;
}

/// Declares a parse stored together with the text and arena it borrows.
macro_rules! owned_parse {
    ($(#[$doc:meta])* $name:ident, $dependent:ident) => {
        $(#[$doc])*
        struct $name {
            /// Borrows only `*source`; dropped before it (see `Drop`).
            parsed: ManuallyDrop<$dependent<'static>>,
            /// Owned by this value: allocated in `new`, freed in `Drop`.
            source: NonNull<Source>,
        }

        impl $name {
            fn new(
                source: Source,
                parse: impl for<'a> FnOnce(&'a Source) -> $dependent<'a>,
            ) -> Self {
                let source = NonNull::from(Box::leak(Box::new(source)));
                // SAFETY: `source` is a live heap value that this struct owns
                // and frees only in `Drop`, after `parsed`; nothing mutates it.
                let parsed = parse(unsafe { source.as_ref() });
                // SAFETY: only the lifetime changes. `parsed` borrows
                // `*source`, which outlives it, and it is only lent as
                // `&'s T<'s>` for a borrow `'s` of `self` (see `parsed`).
                let parsed = unsafe {
                    std::mem::transmute::<$dependent<'_>, $dependent<'static>>(parsed)
                };
                Self {
                    parsed: ManuallyDrop::new(parsed),
                    source,
                }
            }

            fn parsed(&self) -> &$dependent<'_> {
                &self.parsed
            }
        }

        impl Shared for $name {
            fn source(&self) -> &Source {
                // SAFETY: `source` stays allocated while `self` lives.
                unsafe { self.source.as_ref() }
            }
        }

        impl Drop for $name {
            fn drop(&mut self) {
                // SAFETY: the parse is dropped first and never used again,
                // then the source it borrowed is freed exactly once, as
                // allocated in `new`.
                unsafe {
                    ManuallyDrop::drop(&mut self.parsed);
                    drop(Box::from_raw(self.source.as_ptr()));
                }
            }
        }
    };
}

owned_parse!(
    /// A program parse together with the text and arena it borrows.
    SharedParse,
    ParserReturn
);

/// An expression parse; `None` when the text is not one expression.
type ParsedExpression<'a> = Option<Expression<'a>>;

owned_parse!(
    /// An expression parse together with the text and arena it borrows.
    SharedExpression,
    ParsedExpression
);

/// Parses by the hash of their text and source type.
struct Shelf<T>(FxHashMap<u64, Vec<Rc<T>>>);

impl<T> Default for Shelf<T> {
    fn default() -> Self {
        Self(FxHashMap::default())
    }
}

#[derive(Default)]
struct Parses {
    programs: Shelf<SharedParse>,
    expressions: Shelf<SharedExpression>,
}

thread_local! {
    /// The current projection's parses. Projections are synchronous on their
    /// thread, like the bindings table in Canon's script facts.
    static PARSES: RefCell<Option<Parses>> = const { RefCell::new(None) };
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
            *parses = Some(Parses::default());
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

/// The scope's parse of `text` as `source_type`, made by `parse` on first
/// read, or `None` outside a scope.
fn share<T: Shared>(
    text: &str,
    source_type: SourceType,
    shelf: fn(&mut Parses) -> &mut Shelf<T>,
    parse: impl FnOnce(Source) -> T,
) -> Option<Rc<T>> {
    let key = FxBuildHasher.hash_one((text, source_type));
    let found = PARSES.with_borrow_mut(|parses| {
        let parses = parses.as_mut()?;
        Some(shelf(parses).0.get(&key).and_then(|entries| {
            entries
                .iter()
                .find(|entry| entry.source().is(text, source_type))
                .cloned()
        }))
    })?;
    if found.is_some() {
        return found;
    }
    // No borrow is held while parsing: a reader of another shared parse may
    // be running further up this thread's stack.
    let entry = Rc::new(parse(Source::new(text, source_type)));
    PARSES.with_borrow_mut(|parses| {
        if let Some(parses) = parses {
            shelf(parses)
                .0
                .entry(key)
                .or_default()
                .push(Rc::clone(&entry));
        }
    });
    Some(entry)
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
    let shared = share(
        text,
        source_type,
        |parses| &mut parses.programs,
        |source| {
            SharedParse::new(source, |source| {
                Parser::new(&source.allocator, &source.text, source.source_type).parse()
            })
        },
    );
    match shared {
        Some(parse) => read(parse.parsed()),
        None => {
            let allocator = Allocator::default();
            let parsed = Parser::new(&allocator, text, source_type).parse();
            read(&parsed)
        }
    }
}

/// Read [`Parser::parse_expression`] of `text` as `source_type` with default
/// parse options: the expression, or `None` when it fails.
///
/// Sharing follows [`with_shared_parse`]; an expression and a program of the
/// same text are separate parses.
pub fn with_shared_expression<R>(
    text: &str,
    source_type: SourceType,
    read: impl for<'a> FnOnce(Option<&Expression<'a>>) -> R,
) -> R {
    let shared = share(
        text,
        source_type,
        |parses| &mut parses.expressions,
        |source| {
            SharedExpression::new(source, |source| {
                Parser::new(&source.allocator, &source.text, source.source_type)
                    .parse_expression()
                    .ok()
            })
        },
    );
    match shared {
        Some(expression) => read(expression.parsed().as_ref()),
        None => {
            let allocator = Allocator::default();
            let parsed = Parser::new(&allocator, text, source_type)
                .parse_expression()
                .ok();
            read(parsed.as_ref())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use oxc_span::GetSpan;

    fn tree_address(text: &str, source_type: SourceType) -> usize {
        with_shared_parse(text, source_type, |parsed| {
            std::ptr::from_ref(&parsed.program) as usize
        })
    }

    fn expression_address(text: &str) -> Option<usize> {
        with_shared_expression(text, SourceType::ts(), |expression| {
            expression.map(|expression| std::ptr::from_ref(expression) as usize)
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
    fn expressions_share_apart_from_programs_and_keep_failures() {
        let private = with_shared_expression("(a) && b.c", SourceType::ts(), |expression| {
            expression.map(GetSpan::span)
        });
        let _scope = SharedParseScope::enter();
        let first = expression_address("(a) && b.c");
        assert!(first.is_some());
        assert_eq!(expression_address("(a) && b.c"), first);
        assert_ne!(expression_address("(a) || b.c"), first);
        assert_eq!(
            with_shared_expression("(a) && b.c", SourceType::ts(), |expression| {
                expression.map(GetSpan::span)
            }),
            private
        );
        assert_eq!(expression_address("a +"), None);
        assert_eq!(expression_address("a +"), None);
        tree_address("(a) && b.c", SourceType::ts());
        PARSES.with_borrow(|parses| {
            let parses = parses.as_ref().map(|parses| {
                (
                    parses.programs.0.values().map(Vec::len).sum::<usize>(),
                    parses.expressions.0.values().map(Vec::len).sum::<usize>(),
                )
            });
            assert_eq!(parses, Some((1, 3)));
        });
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
