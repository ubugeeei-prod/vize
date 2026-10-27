//! pug's `include`, as `pug-load@3.0.0` and `pug-linker@4.0.0` apply it.
//!
//! Every include is loaded before the template runs, wherever it stands
//! (a branch pug skips, a mixin nothing calls): a path is resolved against
//! the file that names it, a `.pug` / `.jade` file is parsed and spliced
//! in as nodes — with the directive's own block at its `yield`, or at the
//! end of its last nested block — and any other file is inserted as raw
//! text. The host reads files ([`PugIncludes`]); a surface without one
//! keeps refusing includes. Spans inside an included file land on the
//! directive in the template that included it.

use alloc::vec::Vec as StdVec;

use vize_l0::{Allocator, Span, String, cstr};
use vize_l1::pug::{
    PugBlock, PugHtmlItem, PugInline, PugNode, PugRefusal, PugRefused, PugTag, PugTextPiece,
    PugTree, parse_pug,
};

use super::emit::Emitter;

mod head;
mod limits;
use head::head;
pub use limits::MAX_INCLUDE_BYTES;

/// Reads the files a pug template includes.
pub trait PugIncludes {
    /// The file `path` names, as written in the file `from`: its resolved
    /// name and content, or why it cannot be read.
    fn read(&mut self, from: &str, path: &str) -> Result<(String, String), String>;
}

/// Keep an independent depth backstop for non-cyclic include chains.
const MAX_DEPTH: usize = 32;

enum Loaded<'t> {
    Pug(&'t PugTree<'t>),
    Raw(&'t str),
}

/// An included source: its bytes, where it lands in the template, and its
/// name for its own relative includes.
struct File {
    start: usize,
    end: usize,
    origin: Span,
    name: String,
}

/// The includes a template loads.
pub(super) struct Includes<'t> {
    /// Whether the host reads files; without one includes are refused.
    reads: bool,
    pub(super) budget: limits::Budget,
    /// The template's own file name.
    filename: Option<String>,
    arena: &'t Allocator,
    files: StdVec<File>,
    loaded: StdVec<(&'t PugRefused<'t>, Loaded<'t>)>,
    /// `yield` keywords, or default node lists, and the block they emit.
    yields: StdVec<(usize, &'t [PugNode<'t>])>,
    /// Resolved files on this recursive chain; siblings may include a file again.
    active: StdVec<String>,
    pub(super) dependencies: StdVec<String>,
}

impl<'t> Includes<'t> {
    pub(super) fn new(arena: &'t Allocator, reads: bool, filename: Option<&str>) -> Self {
        Self {
            reads,
            budget: limits::Budget::default(),
            filename: filename.map(String::from),
            active: filename.into_iter().map(String::from).collect(),
            arena,
            files: StdVec::new(),
            loaded: StdVec::new(),
            yields: StdVec::new(),
            dependencies: StdVec::new(),
        }
    }

    /// The included file holding the bytes at `at`, if any.
    fn file(&self, at: usize) -> Option<&File> {
        self.files
            .iter()
            .find(|file| (file.start..=file.end).contains(&at))
    }
}

/// `pug-linker`'s `defaultYieldLocation`: the block of the last node with
/// one (text-only tags skipped), followed down. Keys are a tag (its block
/// ends after its same-line content and nested nodes) or a node list.
fn yield_location(nodes: &[PugNode<'_>], owner: usize) -> usize {
    let mut location = owner;
    for node in nodes {
        match node {
            PugNode::Tag(tag) => location = tag_location(tag).unwrap_or(location),
            PugNode::Call(call) => location = tag_location(&call.tag).unwrap_or(location),
            PugNode::Refused(refused)
                if refused.refusal != PugRefusal::Conditional && !refused.block.is_empty() =>
            {
                location = yield_location(&refused.block, refused.block.as_ptr() as usize);
            }
            PugNode::Code(code) if !code.block.is_empty() => {
                location = yield_location(&code.block, code.block.as_ptr() as usize);
            }
            _ => {}
        }
    }
    location
}

fn tag_location(tag: &PugTag<'_>) -> Option<usize> {
    let nested = match &tag.block {
        PugBlock::Nodes(nodes) => nodes.as_slice(),
        PugBlock::Text(_) => &[],
    };
    let inline = !matches!(tag.inline, PugInline::None | PugInline::SelfClosing(_));
    if tag.dot.is_some() || !(inline || !nested.is_empty()) {
        return None;
    }
    let mut location = core::ptr::from_ref(tag) as usize;
    if let PugInline::Expansion { node, .. } = &tag.inline {
        location = yield_location(core::slice::from_ref(&**node), location);
    }
    Some(yield_location(nested, location))
}

/// Every node under `nodes`, in order.
fn walk<'t>(nodes: &'t [PugNode<'t>], visit: &mut dyn FnMut(&'t PugNode<'t>)) {
    for node in nodes {
        visit(node);
        vize_l0::ensure_sufficient_stack(|| walk_children(node, visit));
    }
}

fn walk_children<'t>(node: &'t PugNode<'t>, visit: &mut dyn FnMut(&'t PugNode<'t>)) {
    let text = |pieces: &'t [PugTextPiece<'t>], visit: &mut dyn FnMut(&'t PugNode<'t>)| {
        for piece in pieces {
            if let PugTextPiece::Interpolation(interpolation) = piece {
                walk(&interpolation.nodes, visit);
            }
        }
    };
    let tag = |tag: &'t PugTag<'t>, visit: &mut dyn FnMut(&'t PugNode<'t>)| {
        match &tag.inline {
            PugInline::Text(inline) => text(&inline.pieces, visit),
            PugInline::Expansion { node, .. } => walk(core::slice::from_ref(&**node), visit),
            _ => {}
        }
        match &tag.block {
            PugBlock::Nodes(nodes) => walk(nodes, visit),
            PugBlock::Text(block) => text(&block.pieces, visit),
        }
    };
    match node {
        PugNode::Tag(node) => tag(node, visit),
        PugNode::Call(call) => tag(&call.tag, visit),
        PugNode::Text(node) => text(&node.pieces, visit),
        PugNode::TextBlock(block) => text(&block.text.pieces, visit),
        PugNode::Html(html) => {
            for item in &html.items {
                match item {
                    PugHtmlItem::Line(line) => text(&line.pieces, visit),
                    PugHtmlItem::Block(nodes) => walk(nodes, visit),
                    PugHtmlItem::Code(code) => walk(&code.block, visit),
                }
            }
        }
        PugNode::Code(code) => walk(&code.block, visit),
        PugNode::Refused(refused) => walk(&refused.block, visit),
        PugNode::Unexpected(unexpected) => walk(&unexpected.block, visit),
        PugNode::Comment(_) => {}
    }
}

impl<'t> Emitter<'t> {
    /// Load every include under `nodes` (pug links them all, run or not).
    pub(super) fn load_includes(
        &mut self,
        nodes: &'t [PugNode<'t>],
        depth: usize,
        reader: &mut dyn PugIncludes,
    ) {
        if self.includes.budget.halted {
            return;
        }
        let mut directives = StdVec::new();
        walk(nodes, &mut |node| {
            if let PugNode::Refused(refused) = node
                && refused.refusal == PugRefusal::Include
            {
                directives.push(&**refused);
            }
        });
        for directive in directives {
            if self.includes.budget.halted {
                break;
            }
            self.load(directive, depth, reader);
        }
    }

    fn load(&mut self, directive: &'t PugRefused<'t>, depth: usize, reader: &mut dyn PugIncludes) {
        let span = self.span(&directive.head);
        let at = directive.head.text.as_ptr() as usize;
        let origin = self.includes.file(at).map(|file| file.origin);
        let from = match self.includes.file(at) {
            Some(file) => Some(file.name.clone()),
            None => self.includes.filename.clone(),
        };
        let (path, filtered) = match head(directive.head.text) {
            Ok(head) => head,
            Err(message) => return self.error(span, String::from(message)),
        };
        if filtered {
            return self.error(span, super::refusal::message(PugRefusal::Filter));
        }
        let Some(from) = from.filter(|_| !limits::absolute(path)) else {
            let option = if limits::absolute(path) {
                "\"basedir\" option is required to use includes and extends with \"absolute\""
            } else {
                "\"filename\" option is required to use includes and extends with \"relative\""
            };
            return self.error(span, cstr!("the {option} paths"));
        };
        if depth >= MAX_DEPTH {
            return self.error(span, String::from(super::refusal::INCLUDE_DEPTH));
        }
        if !self.includes.budget.load() {
            return self.include_budget_error(span);
        }
        let (name, content) = match reader.read(&from, path) {
            Ok(read) => read,
            Err(reason) => return self.error(span, super::refusal::include(path, &reason)),
        };
        if !self.includes.budget.source(content.len()) {
            return self.include_budget_error(span);
        }
        if self.includes.active.contains(&name) {
            return self.error(span, String::from(super::refusal::INCLUDE_DEPTH));
        }
        if !self.includes.dependencies.contains(&name) {
            self.includes.dependencies.push(name.clone());
        }
        let arena = self.includes.arena;
        let pug = path.ends_with(".pug") || path.ends_with(".jade");
        if !pug {
            if !directive.block.is_empty() {
                let message = String::from("Raw inclusion cannot contain a block");
                return self.error(span, message);
            }
            let text = arena.alloc_str(&content.replace('\r', ""));
            self.includes.loaded.push((directive, Loaded::Raw(text)));
            return;
        }
        let source = arena.alloc_str(&content);
        let (tree, errors) = parse_pug(arena, source);
        let tree: &'t PugTree<'t> = arena.as_oxc().alloc(tree);
        self.includes.active.push(name.clone());
        self.includes.files.push(File {
            start: source.as_ptr() as usize,
            end: source.as_ptr() as usize + source.len(),
            origin: origin.unwrap_or(span),
            name,
        });
        for error in errors.iter() {
            let origin = origin.unwrap_or(span);
            self.diagnostics.push(crate::exemptions::surface_syntax(
                origin,
                error.code.message(),
            ));
        }
        if !directive.block.is_empty() {
            let mut yields = StdVec::new();
            walk(&tree.nodes, &mut |node| {
                if let PugNode::Refused(refused) = node
                    && refused.refusal == PugRefusal::Yield
                {
                    yields.push(core::ptr::from_ref(&**refused) as usize);
                }
            });
            if yields.is_empty() {
                yields.push(yield_location(&tree.nodes, tree.nodes.as_ptr() as usize));
            }
            for key in yields {
                self.includes.yields.push((key, &directive.block));
            }
        }
        self.includes.loaded.push((directive, Loaded::Pug(tree)));
        self.load_includes(&tree.nodes, depth + 1, reader);
        self.includes.active.pop();
    }

    /// Emit an include where it stands.
    pub(super) fn include(&mut self, directive: &'t PugRefused<'t>) {
        if !self.includes.reads {
            let span = self.span(&directive.head);
            return self.error(span, super::refusal::message(PugRefusal::Include));
        }
        if self.includes.budget.halted {
            return;
        }
        if !self.includes.budget.expand() {
            let span = self.span(&directive.head);
            return self.include_budget_error(span);
        }
        let loaded = self
            .includes
            .loaded
            .iter()
            .find(|(loaded, _)| core::ptr::eq(*loaded, directive))
            .map(|(_, loaded)| match loaded {
                Loaded::Pug(tree) => Loaded::Pug(tree),
                Loaded::Raw(text) => Loaded::Raw(text),
            });
        match loaded {
            Some(Loaded::Pug(tree)) => {
                self.nodes(&tree.nodes);
                // An empty file's yield location is its (empty) root.
                if tree.nodes.is_empty() {
                    self.nodes(&directive.block);
                }
            }
            Some(Loaded::Raw(text)) => {
                let span = self.span(&directive.head);
                self.synth(text, span);
            }
            // Loading reported why.
            None => {}
        }
    }

    /// The block an include put at `key` (a `yield`, or the node list
    /// it is appended to).
    pub(super) fn yielded(&self, key: usize) -> Option<&'t [PugNode<'t>]> {
        (self.includes.yields.iter())
            .find(|(at, _)| *at == key)
            .map(|(_, block)| *block)
    }

    /// Where bytes at `at` land in the template: `None` for the template's
    /// own bytes, else the directive that included them.
    pub(super) fn origin(&self, at: usize) -> Option<Span> {
        if self.includes.files.is_empty() {
            return None;
        }
        self.includes.file(at).map(|file| file.origin)
    }
}
