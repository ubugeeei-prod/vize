//! "Wrap with `.value`" / "Unwrap `.value`" quick fixes for script refs (#691).
#![expect(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    clippy::disallowed_macros,
    reason = "tower-lsp code actions take std String titles/edits and a std HashMap of changes"
)]

use tower_lsp::lsp_types::{
    CodeAction, CodeActionKind, CodeActionOrCommand, Position, Range, TextEdit, WorkspaceEdit,
};

use super::{CodeActionService, identifier_at_cursor, is_ident_byte, offset_to_line_col};
use crate::ide::IdeContext;
use crate::virtual_code::BlockType;

const VALUE: &str = ".value";

impl CodeActionService {
    /// Offer an "Unwrap `.value`" quick fix when the cursor sits on the
    /// `.value` segment of a reactive ref access in script context. Useful
    /// when the user copies template code (where refs auto-unwrap) into
    /// script and the trailing `.value` is now redundant — or when they
    /// pass `count.value` to a function that expects the wrapper itself.
    pub(super) fn collect_unwrap_value(ctx: &IdeContext) -> Vec<CodeActionOrCommand> {
        if !in_script(ctx) {
            return Vec::new();
        }
        let Some(dot) = value_access_at(&ctx.content, ctx.offset) else {
            return Vec::new();
        };
        let Some(receiver) = ctx
            .content
            .get(..dot)
            .and_then(|prefix| identifier_at_cursor(prefix, prefix.len()))
            .map(|(_, receiver)| receiver)
        else {
            return Vec::new();
        };
        if !is_reactive_ref_in_script(ctx, receiver) {
            return Vec::new();
        }
        let start = position(&ctx.content, dot);
        let end = position(&ctx.content, dot + VALUE.len());
        vec![quick_fix(
            ctx,
            format!("Unwrap `.value` from `{receiver}`"),
            Range::new(start, end),
            String::new(),
            None,
        )]
    }

    /// Offer a "Wrap with `.value`" quick fix when the cursor sits on an
    /// identifier that resolves to a reactive ref in script context. This is
    /// the most common Vue fix-up; complementary actions (unwrap, add to
    /// defineEmits, etc.) follow the same shape and will land in follow-ups.
    pub(super) fn collect_wrap_with_value(ctx: &IdeContext) -> Vec<CodeActionOrCommand> {
        // Only activate on script blocks — refs in template are auto-unwrapped.
        if !in_script(ctx) {
            return Vec::new();
        }
        let Some((range, identifier)) = identifier_at_cursor(&ctx.content, ctx.offset) else {
            return Vec::new();
        };
        if !is_ref_read(&ctx.content, range.clone()) || !is_reactive_ref_in_script(ctx, identifier)
        {
            return Vec::new();
        }
        let end = position(&ctx.content, range.end);
        vec![quick_fix(
            ctx,
            format!("Wrap `{identifier}` with `.value`"),
            Range::new(end, end),
            VALUE.to_string(),
            Some(true),
        )]
    }
}

fn in_script(ctx: &IdeContext) -> bool {
    matches!(
        ctx.block_type,
        Some(BlockType::ScriptSetup | BlockType::Script)
    )
}

/// Start of the `.value` member access whose span (dot included) holds the
/// cursor.
fn value_access_at(content: &str, cursor: usize) -> Option<usize> {
    let cursor = cursor.min(content.len());
    (cursor.saturating_sub(VALUE.len())..=cursor).find(|&dot| {
        content
            .get(dot..)
            .is_some_and(|tail| tail.starts_with(VALUE))
            && !content
                .as_bytes()
                .get(dot + VALUE.len())
                .is_some_and(|byte| is_ident_byte(*byte))
    })
}

/// A read of the binding itself: not its declaration, not another object's
/// member of the same name, and not already followed by `.value`.
fn is_ref_read(content: &str, range: std::ops::Range<usize>) -> bool {
    let before = content.get(..range.start).unwrap_or_default().trim_end();
    let declared = ["const", "let", "var"].iter().any(|keyword| {
        before.strip_suffix(keyword).is_some_and(|prefix| {
            !prefix
                .as_bytes()
                .last()
                .is_some_and(|byte| is_ident_byte(*byte))
        })
    });
    let member = before.ends_with('.') && !before.ends_with("...");
    let unwrapped = content
        .get(range.end..)
        .is_some_and(|tail| tail.trim_start().starts_with(VALUE));
    !declared && !member && !unwrapped
}

fn position(content: &str, offset: usize) -> Position {
    let (line, character) = offset_to_line_col(content, offset);
    Position { line, character }
}

fn quick_fix(
    ctx: &IdeContext,
    title: String,
    range: Range,
    new_text: String,
    is_preferred: Option<bool>,
) -> CodeActionOrCommand {
    let mut changes = std::collections::HashMap::new();
    changes.insert(ctx.uri.clone(), vec![TextEdit { range, new_text }]);
    CodeActionOrCommand::CodeAction(CodeAction {
        title,
        kind: Some(CodeActionKind::QUICKFIX),
        diagnostics: None,
        edit: Some(WorkspaceEdit {
            changes: Some(changes),
            document_changes: None,
            change_annotations: None,
        }),
        command: None,
        is_preferred,
        disabled: None,
        data: None,
    })
}

/// True when `name` resolves to a Vue ref (Ref / ShallowRef / ToRef /
/// ComputedRef) declared anywhere in the SFC's script-setup block. Used to
/// decide whether "Wrap with `.value`" makes sense at the cursor.
fn is_reactive_ref_in_script(ctx: &IdeContext, name: &str) -> bool {
    use vize_croquis::reactivity::ReactiveKind;
    let Some(descriptor) = ctx.descriptor() else {
        return false;
    };
    let Some(ref script_setup) = descriptor.script_setup else {
        return false;
    };
    let mut analyzer = vize_croquis::Drawer::with_options(vize_croquis::DrawerOptions {
        analyze_script: true,
        ..Default::default()
    });
    analyzer.analyze_script_setup(&script_setup.content);
    let croquis = analyzer.finish();
    vize_croquis::facts::reactivity_lookup(&croquis, name).is_some_and(|source| {
        matches!(
            source.kind,
            ReactiveKind::Ref
                | ReactiveKind::ShallowRef
                | ReactiveKind::ToRef
                | ReactiveKind::Computed
        )
    })
}
