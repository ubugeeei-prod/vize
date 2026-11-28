//! One parse per script text and dialect while one SFC is projected.
//!
//! Projecting an SFC reads its script through many independent analyses, and
//! most of them parse the same text with the same [`SourceType`]. Through a
//! [`ScriptParses`] handle, [`with_shared_parse`] parses each distinct
//! `(text, source type)` once and lends every later reader that same
//! [`ParserReturn`]. Without a handle it parses privately.
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
//! The handle borrows an arena its caller owns. Each shared parse copies its
//! text into that arena and builds its tree there, so the borrow checker
//! proves that every lent parse lives inside the arena's borrow.

#![expect(
    clippy::disallowed_types,
    reason = "a lent parse outlives the handle's borrow of its list, so a nested read can add one"
)]

use std::cell::RefCell;
use std::rc::Rc;

use oxc_allocator::Allocator;
use oxc_ast::ast::Statement;
use oxc_parser::{Parser, ParserReturn};
use oxc_span::{FileExtension, GetSpan, ModuleKind, SourceType};

/// A parse together with the text it was parsed from.
struct SharedParse<'p> {
    text: &'p str,
    source_type: SourceType,
    parsed: ParserReturn<'p>,
}

/// The parses one projection shares, in an arena its caller owns.
pub struct ScriptParses<'p> {
    allocator: &'p Allocator,
    parses: RefCell<Vec<Rc<SharedParse<'p>>>>,
}

impl std::fmt::Debug for ScriptParses<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ScriptParses")
            .field("parses", &self.parses.borrow().len())
            .finish_non_exhaustive()
    }
}

impl<'p> ScriptParses<'p> {
    /// Share parses whose text and trees live in `allocator`.
    pub fn new(allocator: &'p Allocator) -> Self {
        Self {
            allocator,
            parses: RefCell::new(Vec::new()),
        }
    }

    /// The shared parse of `text` as `source_type`. Sharing follows
    /// [`with_shared_parse`].
    pub fn program(&self, text: &str, source_type: SourceType) -> SharedProgram<'p> {
        let source_type = program_source_type(source_type);
        if let Some(parse) = self.module_goal_from_unambiguous(text, source_type) {
            return SharedProgram(parse);
        }
        if let Some(parse) = self.find_program(text, source_type) {
            return SharedProgram(parse);
        }

        // No borrow is held while parsing: a reader of another shared parse may
        // be running further up this thread's stack.
        let text = &*self.allocator.alloc_str(text);
        let parse = Rc::new(SharedParse {
            text,
            source_type,
            parsed: Parser::new(self.allocator, text, source_type).parse(),
        });
        self.parses.borrow_mut().push(Rc::clone(&parse));
        SharedProgram(parse)
    }

    /// The existing parse of `text` as `source_type`, if any.
    fn find_program(&self, text: &str, source_type: SourceType) -> Option<Rc<SharedParse<'p>>> {
        self.parses
            .borrow()
            .iter()
            .find(|parse| parse.source_type == source_type && parse.text == text)
            .cloned()
    }

    /// The unambiguous parse of `text`, when it is exactly the parse of the
    /// module goal `source_type` asks for.
    ///
    /// An unambiguous TypeScript parse that finds an `import` or `export`
    /// resolves to the module goal, and from the statement after the first
    /// such one on it parses exactly as the module goal does. The goals differ
    /// only in three places: whether a top-level `await` is a keyword before
    /// that point, HTML-like comments (`<!--`, `-->`), and the order of the
    /// errors those produce. So the unambiguous parse is reused only when it
    /// resolved to a module through an `import`/`export` statement, no
    /// `await` appears in the text up to the end of the first one, and the
    /// text has no `<!--` or `-->`. Any other script keeps its own module-goal
    /// parse.
    fn module_goal_from_unambiguous(
        &self,
        text: &str,
        source_type: SourceType,
    ) -> Option<Rc<SharedParse<'p>>> {
        if source_type.module_kind() != ModuleKind::Module
            || !source_type.is_typescript()
            || source_type.is_typescript_definition()
            || source_type.extension().is_some()
        {
            return None;
        }
        let parse = self.find_program(text, source_type.with_unambiguous(true))?;
        parses_as_module_goal(&parse.parsed, text).then_some(parse)
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

/// Read the parse of `text` as `source_type` with default parse options.
///
/// Through `parses`, equal text read as the same source type is parsed once;
/// `read` receives that parse. Without a handle the parse is private to this
/// call.
pub fn with_shared_parse<R>(
    parses: Option<&ScriptParses<'_>>,
    text: &str,
    source_type: SourceType,
    read: impl for<'a> FnOnce(&ParserReturn<'a>) -> R,
) -> R {
    match parses {
        Some(parses) => read(parses.program(text, source_type).get()),
        None => {
            let allocator = Allocator::default();
            let parsed = Parser::new(&allocator, text, source_type).parse();
            read(&parsed)
        }
    }
}

/// A program parse a [`ScriptParses`] lends, for readers that keep it beside
/// other state, as `SfcScriptParses` does.
#[derive(Clone)]
pub struct SharedProgram<'p>(Rc<SharedParse<'p>>);

impl<'p> SharedProgram<'p> {
    /// The parse.
    pub fn get(&self) -> &ParserReturn<'p> {
        &self.0.parsed
    }
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

#[cfg(test)]
mod tests;
