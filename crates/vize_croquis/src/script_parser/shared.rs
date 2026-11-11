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
//! A `.ts`/`.tsx` file extension on a TypeScript source type is not part of
//! the key: the parser reads it only to tell `.mts`/`.cts` apart.
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
use oxc_ast::ast::{Expression, Statement};
use oxc_parser::{Parser, ParserReturn};
use oxc_span::{FileExtension, GetSpan, ModuleKind, SourceType};
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

/// The source type a shared program is parsed and found under.
///
/// A `.ts` or `.tsx` file extension (`SourceType::from_path("script.ts")`)
/// changes nothing the parser does: it reads the extension only to tell
/// `.mts`/`.cts` apart. Such a source type shares the parse of the same
/// dialect without an extension, whose `program.source_type` carries no
/// extension either. Every other source type, JavaScript included, stays
/// its own parse.
fn program_source_type(source_type: SourceType) -> SourceType {
    if !matches!(
        source_type.extension(),
        Some(FileExtension::Ts | FileExtension::Tsx)
    ) || !source_type.is_typescript()
        || source_type.is_typescript_definition()
    {
        return source_type;
    }
    let dialect = SourceType::ts().with_jsx(source_type.is_jsx());
    let dialect = match source_type.module_kind() {
        ModuleKind::Unambiguous => dialect,
        ModuleKind::Module => dialect.with_module(true),
        ModuleKind::Script => dialect.with_script(true),
        ModuleKind::CommonJS => return source_type,
    };
    if dialect.is_typescript() && dialect.is_jsx() == source_type.is_jsx() {
        dialect
    } else {
        source_type
    }
}

/// The scope's existing parse of `text` as `source_type`: `None` outside a
/// scope, `Some(None)` when the scope has not parsed it.
fn find<T: Shared>(
    text: &str,
    source_type: SourceType,
    shelf: fn(&mut Parses) -> &mut Shelf<T>,
) -> Option<Option<Rc<T>>> {
    let key = FxBuildHasher.hash_one((text, source_type));
    PARSES.with_borrow_mut(|parses| {
        let parses = parses.as_mut()?;
        Some(shelf(parses).0.get(&key).and_then(|entries| {
            entries
                .iter()
                .find(|entry| entry.source().is(text, source_type))
                .cloned()
        }))
    })
}

/// The scope's parse of `text` as `source_type`, made by `parse` on first
/// read, or `None` outside a scope.
fn share<T: Shared>(
    text: &str,
    source_type: SourceType,
    shelf: fn(&mut Parses) -> &mut Shelf<T>,
    parse: impl FnOnce(Source) -> T,
) -> Option<Rc<T>> {
    let found = find(text, source_type, shelf)?;
    if found.is_some() {
        return found;
    }
    let key = FxBuildHasher.hash_one((text, source_type));
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
    match shared_program(text, source_type) {
        Some(parse) => read(parse.get()),
        None => {
            let allocator = Allocator::default();
            let parsed = Parser::new(&allocator, text, source_type).parse();
            read(&parsed)
        }
    }
}

/// A program parse a [`SharedParseScope`] lends, for readers that keep it
/// beside other state, as `SfcScriptParses` does.
#[derive(Clone)]
pub struct SharedProgram(Rc<SharedParse>);

impl SharedProgram {
    /// The parse.
    pub fn get(&self) -> &ParserReturn<'_> {
        self.0.parsed()
    }
}

/// The scope's parse of `text` as `source_type`, or `None` outside a scope.
/// Sharing follows [`with_shared_parse`].
pub fn shared_program(text: &str, source_type: SourceType) -> Option<SharedProgram> {
    let source_type = program_source_type(source_type);
    if let Some(parse) = module_goal_from_unambiguous(text, source_type) {
        return Some(SharedProgram(parse));
    }
    share(
        text,
        source_type,
        |parses| &mut parses.programs,
        |source| {
            SharedParse::new(source, |source| {
                Parser::new(&source.allocator, &source.text, source.source_type).parse()
            })
        },
    )
    .map(SharedProgram)
}

/// The scope's unambiguous parse of `text`, when it is exactly the parse of
/// the module goal `source_type` asks for.
///
/// An unambiguous TypeScript parse that finds an `import` or `export`
/// resolves to the module goal, and from the statement after the first such
/// one on it parses exactly as the module goal does. The goals differ only in
/// three places: whether a top-level `await` is a keyword before that point,
/// HTML-like comments (`<!--`, `-->`), and the order of the errors those
/// produce. So the unambiguous parse is reused only when it resolved to a
/// module through an `import`/`export` statement, no `await` appears in the
/// text up to the end of the first one, and the text has no `<!--` or `-->`.
/// Any other script keeps its own module-goal parse.
fn module_goal_from_unambiguous(text: &str, source_type: SourceType) -> Option<Rc<SharedParse>> {
    if source_type.module_kind() != ModuleKind::Module
        || !source_type.is_typescript()
        || source_type.is_typescript_definition()
        || source_type.extension().is_some()
    {
        return None;
    }
    let parse = find(text, source_type.with_unambiguous(true), |parses| {
        &mut parses.programs
    })??;
    parses_as_module_goal(parse.parsed(), text).then_some(parse)
}

/// Whether an unambiguous parse of `text` is exactly its module-goal parse.
fn parses_as_module_goal(parsed: &ParserReturn<'_>, text: &str) -> bool {
    if parsed.panicked
        || !parsed.program.source_type.is_module()
        || text.contains("<!--")
        || text.contains("-->")
    {
        return false;
    }
    let Some(first) = parsed.program.body.iter().find(|statement| {
        matches!(
            statement,
            Statement::ImportDeclaration(_)
                | Statement::ExportAllDeclaration(_)
                | Statement::ExportDefaultDeclaration(_)
                | Statement::ExportNamedDeclaration(_)
        )
    }) else {
        return false;
    };
    text.get(..first.span().end as usize)
        .is_some_and(|prefix| !prefix.contains("await"))
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
mod tests;
