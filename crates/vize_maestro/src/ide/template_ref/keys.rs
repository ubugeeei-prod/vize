//! Template-ref keys as one renamable name.
//!
//! `useTemplateRef('box')`, `$refs.box` and `ref="box"` are linked only by
//! the string `box`, so no TypeScript symbol connects them. This module
//! treats the key itself as the symbol: references list every place it is
//! spelled, a rename rewrites all of them together, and go to definition
//! from a read of the key reaches the elements that register it. The binding
//! a call is assigned to is an ordinary script symbol and keeps its name.
//!
//! A key is spelled by `useTemplateRef('box')`, `$refs.box`, `$refs['box']`
//! (`this.$refs` in a script), `ref="box"` and `:ref="'box'"`. It is renamed
//! only when every spelling is known: a component that names refs some other
//! way (a computed `useTemplateRef` argument or `$refs` index, a dynamic
//! `:ref`, a string with escapes) gets no rename or references here, so a
//! rename never leaves one spelling behind.

use tower_lsp::lsp_types::{GotoDefinitionResponse, Location, Range, TextEdit, WorkspaceEdit};

use super::range_from_offsets;
use crate::ide::IdeContext;

mod script;
mod template;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Site {
    /// `useTemplateRef('box')`.
    Call,
    /// `$refs.box`: only an identifier can be spelled here.
    Member,
    /// `$refs['box']`.
    Index,
    /// `ref="box"` or `:ref="'box'"`.
    Attribute,
}

impl Site {
    fn reads(self) -> bool {
        self != Self::Attribute
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Spelling {
    key: String,
    start: usize,
    end: usize,
    site: Site,
}

/// Every key spelling found in a component.
#[derive(Debug)]
struct Spellings {
    list: Vec<Spelling>,
    /// Whether no ref is named in a way this module cannot follow.
    complete: bool,
}

impl Spellings {
    fn push(&mut self, key: &str, start: usize, end: usize, site: Site) {
        self.list.push(Spelling {
            key: key.to_string(),
            start,
            end,
            site,
        });
    }

    fn at(&self, offset: usize) -> Option<&Spelling> {
        self.list
            .iter()
            .find(|spelling| spelling.start <= offset && offset <= spelling.end)
    }

    fn of<'s>(&'s self, key: &'s str) -> impl Iterator<Item = &'s Spelling> {
        self.list.iter().filter(move |spelling| spelling.key == key)
    }
}

/// The key under the cursor and every place the component spells it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TemplateRefKey {
    pub(crate) name: String,
    selected: (usize, usize),
    /// Source offsets of the key text, without quotes, in source order.
    spellings: Vec<(usize, usize)>,
    /// Whether some spelling is a `$refs.key` member name.
    member: bool,
}

impl TemplateRefKey {
    pub(crate) fn selected_range(&self, content: &str) -> Range {
        range_from_offsets(content, self.selected.0, self.selected.1)
    }

    pub(crate) fn locations(&self, ctx: &IdeContext<'_>) -> Vec<Location> {
        self.spellings
            .iter()
            .map(|&(start, end)| location(ctx, start, end))
            .collect()
    }

    pub(crate) fn rename(&self, ctx: &IdeContext<'_>, new_name: &str) -> Option<WorkspaceEdit> {
        if !is_editable_key(new_name) || (self.member && !is_identifier_name(new_name)) {
            return None;
        }
        let edits = self
            .spellings
            .iter()
            .map(|&(start, end)| TextEdit {
                range: range_from_offsets(&ctx.content, start, end),
                new_text: new_name.to_string(),
            })
            .collect();
        Some(WorkspaceEdit {
            changes: Some([(ctx.uri.clone(), edits)].into_iter().collect()),
            document_changes: None,
            change_annotations: None,
        })
    }
}

/// The template-ref key at the cursor, when the cursor is on one of its
/// spellings, the key is read at least once, and every spelling in the
/// component is known.
pub(crate) fn key_at_offset(ctx: &IdeContext<'_>) -> Option<TemplateRefKey> {
    let spellings = spellings(ctx).filter(|spellings| spellings.complete)?;
    let selected = spellings.at(ctx.offset)?;
    if !spellings
        .of(&selected.key)
        .any(|spelling| spelling.site.reads())
    {
        return None;
    }
    Some(TemplateRefKey {
        name: selected.key.clone(),
        selected: (selected.start, selected.end),
        spellings: spellings
            .of(&selected.key)
            .map(|spelling| (spelling.start, spelling.end))
            .collect(),
        member: spellings
            .of(&selected.key)
            .any(|spelling| spelling.site == Site::Member),
    })
}

/// The elements that register the key read at the cursor, for a cursor on a
/// `useTemplateRef` argument or a `$refs` key. Other spellings the module
/// cannot follow do not hide the ones it can.
pub(crate) fn definition(ctx: &IdeContext<'_>) -> Option<GotoDefinitionResponse> {
    // A template reads a key only through `$refs`.
    if ctx.is_in_template() && !ctx.content.contains("$refs") {
        return None;
    }
    let spellings = spellings(ctx)?;
    let selected = spellings.at(ctx.offset).filter(|read| read.site.reads())?;
    let mut targets = spellings
        .of(&selected.key)
        .filter(|spelling| spelling.site == Site::Attribute)
        .map(|spelling| location(ctx, spelling.start, spelling.end))
        .collect::<Vec<_>>();
    match targets.len() {
        0 => None,
        1 => targets.pop().map(GotoDefinitionResponse::Scalar),
        _ => Some(GotoDefinitionResponse::Array(targets)),
    }
}

/// Every key spelling in the component. `None` when the component cannot
/// name a read key at all, or its template cannot be read.
fn spellings(ctx: &IdeContext<'_>) -> Option<Spellings> {
    // Every read spells `useTemplateRef` (an alias is imported under that
    // name) or `$refs`.
    if !ctx.content.contains("useTemplateRef") && !ctx.content.contains("$refs") {
        return None;
    }
    let descriptor = ctx.descriptor()?;
    let mut spellings = Spellings {
        list: Vec::new(),
        complete: true,
    };
    for script in [descriptor.script.as_ref(), descriptor.script_setup.as_ref()]
        .into_iter()
        .flatten()
    {
        script::collect(
            script.content.as_ref(),
            script.lang.as_deref(),
            script.loc.start,
            &mut spellings,
        );
    }
    if let Some(block) = descriptor.template.as_ref() {
        if !vize_l1::container::lang::is_html_template(block.lang.as_deref()) {
            return None;
        }
        template::collect(block.content.as_ref(), block.loc.start, &mut spellings);
    }
    spellings.list.sort_by_key(|spelling| spelling.start);
    Some(spellings)
}

fn location(ctx: &IdeContext<'_>, start: usize, end: usize) -> Location {
    Location {
        uri: ctx.uri.clone(),
        range: range_from_offsets(&ctx.content, start, end),
    }
}

/// `$refs` as a whole word at `start` of `content`.
fn is_refs_word(content: &str, start: usize) -> bool {
    let bytes = content.as_bytes();
    let is_name_byte = |byte: &u8| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'$');
    content.get(start..start + "$refs".len()) == Some("$refs")
        && !start
            .checked_sub(1)
            .and_then(|index| bytes.get(index))
            .is_some_and(is_name_byte)
        && !bytes.get(start + "$refs".len()).is_some_and(is_name_byte)
}

/// Whether every whole-word `$refs` in `content` starts at one of `known`
/// (offsets into `content`), so no other use of `$refs` names a key.
fn every_refs_is_known(content: &str, known: &[usize]) -> bool {
    content
        .match_indices("$refs")
        .all(|(start, _)| !is_refs_word(content, start) || known.contains(&start))
}

/// A string literal's key: the text between its quotes, or `None` when the
/// text is not the key verbatim (escapes) and so has no single span.
fn verbatim_key(content: &str, start: usize, end: usize, value: &str) -> Option<(usize, usize)> {
    let (start, end) = (start + 1, end.checked_sub(1)?);
    (content.get(start..end)? == value).then_some((start, end))
}

/// A key that can be written between either quote kind without escaping.
fn is_editable_key(key: &str) -> bool {
    !key.is_empty()
        && !key
            .chars()
            .any(|ch| ch.is_whitespace() || matches!(ch, '"' | '\'' | '`' | '\\' | '<' | '>' | '&'))
}

/// A key that can follow `$refs.`.
fn is_identifier_name(key: &str) -> bool {
    let mut chars = key.chars();
    chars
        .next()
        .is_some_and(|first| first.is_alphabetic() || matches!(first, '_' | '$'))
        && chars.all(|ch| ch.is_alphanumeric() || matches!(ch, '_' | '$'))
}

#[cfg(test)]
#[expect(clippy::string_slice, reason = "tests assert by panicking")]
mod tests;

#[cfg(test)]
#[cfg(feature = "native")]
mod wire_tests;
