//! `<script setup>` declaration facts hover rewrites, memoized per open version.
//!
//! Keyword alignment and import checks walk the same script. The walk is pure
//! in the script text and its `lang`, so one document version reuses it.

use std::sync::{Arc, OnceLock};

use oxc_allocator::Allocator;
use oxc_ast::ast::{
    BindingPattern, Declaration, ImportDeclarationSpecifier, Statement, VariableDeclarationKind,
};
use oxc_parser::Parser;
use parking_lot::Mutex;
use vize_l0::{CompactString, FxHashMap, FxHashSet};

use super::script_source_type;
use crate::ide::IdeContext;

#[derive(Clone, Copy, PartialEq, Eq)]
enum DeclKind {
    Const,
    Let,
    Ambiguous,
}

impl DeclKind {
    fn as_keyword(self) -> Option<&'static str> {
        match self {
            Self::Const => Some("const"),
            Self::Let => Some("let"),
            Self::Ambiguous => None,
        }
    }
}

#[derive(Default)]
pub(super) struct Facts {
    keywords: FxHashMap<CompactString, DeclKind>,
    imports: FxHashSet<CompactString>,
}

impl Facts {
    pub(super) fn keyword(&self, word: &str) -> Option<&'static str> {
        self.keywords.get(word).and_then(|kind| kind.as_keyword())
    }

    pub(super) fn imports_word(&self, word: &str) -> bool {
        !word.is_empty() && self.imports.contains(word)
    }
}

struct Slot {
    version: i32,
    lang: u8,
    script: Box<str>,
    facts: Arc<Facts>,
}

#[derive(Default)]
struct Cache {
    slots: FxHashMap<CompactString, Slot>,
    #[cfg(test)]
    parses: FxHashMap<CompactString, u64>,
}

fn cache() -> &'static Mutex<Cache> {
    static CACHE: OnceLock<Mutex<Cache>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(Cache::default()))
}

#[cfg(test)]
/// Parses counted for `uri`. A repeat hover at the same version does not move it.
pub(super) fn script_fact_parses(uri: &tower_lsp::lsp_types::Url) -> u64 {
    cache()
        .lock()
        .parses
        .get(uri.as_str())
        .copied()
        .unwrap_or(0)
}

/// Authored `const`/`let` for `word`, reusing this document version's walk.
pub(super) fn keyword(ctx: &IdeContext<'_>, word: &str) -> Option<&'static str> {
    if word.is_empty() {
        return None;
    }
    facts(ctx)?.keyword(word)
}

/// Whether a value import binds `word`, reusing this document version's walk.
pub(super) fn imports_value(ctx: &IdeContext<'_>, word: &str) -> bool {
    if word.is_empty() {
        return false;
    }
    facts(ctx).is_some_and(|facts| facts.imports_word(word))
}

fn facts(ctx: &IdeContext<'_>) -> Option<Arc<Facts>> {
    let block = ctx.descriptor()?.script_setup.as_ref()?;
    let script: &str = &block.content;
    let lang = block.lang.as_deref();
    let lang_id = super::script_lang_id(lang);
    let uri_key = CompactString::new(ctx.uri.as_str());
    let version = ctx.state.documents.version(ctx.uri);

    if let Some(version) = version {
        let guard = cache().lock();
        if let Some(slot) = guard.slots.get(&uri_key)
            && slot.version == version
            && slot.lang == lang_id
            && &*slot.script == script
        {
            return Some(Arc::clone(&slot.facts));
        }
    }

    let built = Arc::new(parse_facts(script, lang));
    if let Some(version) = version {
        let mut guard = cache().lock();
        if ctx.state.documents.version(ctx.uri) == Some(version) {
            if guard.slots.len() >= 64 && !guard.slots.contains_key(&uri_key) {
                guard.slots.clear();
            }
            guard.slots.insert(
                uri_key.clone(),
                Slot {
                    version,
                    lang: lang_id,
                    script: script.into(),
                    facts: Arc::clone(&built),
                },
            );
        }
        #[cfg(test)]
        *guard.parses.entry(uri_key).or_default() += 1;
    }
    Some(built)
}

pub(super) fn parse_facts(script: &str, lang: Option<&str>) -> Facts {
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, script, script_source_type(lang)).parse();
    let mut facts = Facts::default();
    for statement in &parsed.program.body {
        match statement {
            Statement::ImportDeclaration(import) => {
                if import.import_kind.is_type() {
                    continue;
                }
                let Some(specifiers) = &import.specifiers else {
                    continue;
                };
                for specifier in specifiers {
                    if let ImportDeclarationSpecifier::ImportSpecifier(named) = specifier
                        && named.import_kind.is_type()
                    {
                        continue;
                    }
                    facts
                        .imports
                        .insert(CompactString::new(specifier.local().name.as_str()));
                }
            }
            Statement::VariableDeclaration(declaration) => {
                collect_declaration(declaration, &mut facts.keywords);
            }
            Statement::ExportNamedDeclaration(export) => {
                if let Some(Declaration::VariableDeclaration(declaration)) = &export.declaration {
                    collect_declaration(declaration, &mut facts.keywords);
                }
            }
            _ => {}
        }
    }
    facts
}

fn collect_declaration(
    declaration: &oxc_ast::ast::VariableDeclaration<'_>,
    keywords: &mut FxHashMap<CompactString, DeclKind>,
) {
    let kind = match declaration.kind {
        VariableDeclarationKind::Const => DeclKind::Const,
        VariableDeclarationKind::Let => DeclKind::Let,
        _ => return,
    };
    for declarator in &declaration.declarations {
        collect_pattern(&declarator.id, kind, keywords);
    }
}

fn collect_pattern(
    pattern: &BindingPattern<'_>,
    kind: DeclKind,
    keywords: &mut FxHashMap<CompactString, DeclKind>,
) {
    match pattern {
        BindingPattern::BindingIdentifier(identifier) => {
            observe(keywords, identifier.name.as_str(), kind);
        }
        BindingPattern::ObjectPattern(object) => {
            for property in &object.properties {
                collect_pattern(&property.value, kind, keywords);
            }
            if let Some(rest) = &object.rest {
                collect_pattern(&rest.argument, kind, keywords);
            }
        }
        BindingPattern::ArrayPattern(array) => {
            for element in array.elements.iter().flatten() {
                collect_pattern(element, kind, keywords);
            }
            if let Some(rest) = &array.rest {
                collect_pattern(&rest.argument, kind, keywords);
            }
        }
        BindingPattern::AssignmentPattern(assignment) => {
            collect_pattern(&assignment.left, kind, keywords);
        }
    }
}

fn observe(keywords: &mut FxHashMap<CompactString, DeclKind>, name: &str, kind: DeclKind) {
    match keywords.get(name) {
        Some(existing) if *existing == kind => {}
        Some(_) => {
            keywords.insert(CompactString::new(name), DeclKind::Ambiguous);
        }
        None => {
            keywords.insert(CompactString::new(name), kind);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::script_fact_parses;
    use crate::{ide::IdeContext, server::ServerState};
    use tower_lsp::lsp_types::{Hover, HoverContents, MarkupContent, MarkupKind, Url};

    const SOURCE: &str = "<script setup lang=\"ts\">\nconst message = 'hello'\nlet count = 1\n</script>\n<template>{{ message }} {{ count }}</template>\n";

    fn hover(value: &str) -> Hover {
        Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: value.to_owned(),
            }),
            range: None,
        }
    }

    fn markdown(hover: &Hover) -> &str {
        match &hover.contents {
            HoverContents::Markup(content) => content.value.as_str(),
            _ => panic!("expected markdown hover"),
        }
    }

    #[test]
    fn same_version_does_not_reparse_script_facts() {
        let uri = Url::parse("file:///workspace/hover-script-facts-6872.vue").unwrap();
        let state = ServerState::new();
        state
            .documents
            .open(uri.clone(), SOURCE.to_owned(), 3, "vue".to_owned());
        let ctx = IdeContext::new(&state, &uri, 0).unwrap();
        let raw = "```typescript\nvar message: string\n```";
        let direct = super::super::align_leading_var(
            raw,
            "const message = 'hello'\nlet count = 1\n",
            Some("ts"),
            "message",
        )
        .unwrap();
        let mut first = hover(raw);
        super::super::align_hover(&ctx, "message", &mut first);
        let text = markdown(&first).to_owned();
        assert_eq!(text, direct);
        assert_eq!(script_fact_parses(&uri), 1);

        let mut again = hover(raw);
        super::super::align_hover(&ctx, "message", &mut again);
        assert_eq!(markdown(&again), text);
        let raw_count = "```typescript\nvar count: number\n```";
        let mut count = hover(raw_count);
        super::super::align_hover(&ctx, "count", &mut count);
        assert_eq!(markdown(&count), "```typescript\nlet count: number\n```");
        assert_eq!(script_fact_parses(&uri), 1);

        state
            .documents
            .open(uri.clone(), SOURCE.to_owned(), 4, "vue".to_owned());
        let ctx = IdeContext::new(&state, &uri, 0).unwrap();
        let mut revised = hover(raw);
        super::super::align_hover(&ctx, "message", &mut revised);
        assert_eq!(markdown(&revised), text);
        assert_eq!(script_fact_parses(&uri), 2);
        super::super::align_hover(&ctx, "message", &mut revised);
        assert_eq!(script_fact_parses(&uri), 2);
    }

    #[test]
    fn reopened_document_with_reused_version_reparses_changed_script() {
        let uri = Url::parse("file:///workspace/hover-script-facts-reopen-6872.vue").unwrap();
        let state = ServerState::new();
        state
            .documents
            .open(uri.clone(), SOURCE.to_owned(), 3, "vue".to_owned());
        let raw = "```typescript\nvar message: string\n```";
        let mut first = hover(raw);
        let ctx = IdeContext::new(&state, &uri, 0).unwrap();
        super::super::align_hover(&ctx, "message", &mut first);
        assert_eq!(
            markdown(&first),
            "```typescript\nconst message: string\n```"
        );
        assert_eq!(script_fact_parses(&uri), 1);

        state.documents.close(&uri);
        let changed = SOURCE.replace("const message", "let message");
        state
            .documents
            .open(uri.clone(), changed, 3, "vue".to_owned());
        let mut reopened = hover(raw);
        let ctx = IdeContext::new(&state, &uri, 0).unwrap();
        super::super::align_hover(&ctx, "message", &mut reopened);
        assert_eq!(
            markdown(&reopened),
            "```typescript\nlet message: string\n```"
        );
        assert_eq!(script_fact_parses(&uri), 2);
    }
}
