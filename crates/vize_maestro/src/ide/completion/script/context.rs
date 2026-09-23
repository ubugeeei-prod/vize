//! Cursor- and SFC-level context helpers for script completion: extracting the
//! script block's content/offset, classifying scope kinds, and locating the
//! member-access receiver under the cursor.
#![expect(
    clippy::disallowed_types,
    reason = "script context text is fed to APIs that take std `String`"
)]

use vize_croquis::ScopeKind;

use crate::ide::IdeContext;

pub(super) fn script_content_for_context(ctx: &IdeContext<'_>, is_setup: bool) -> Option<String> {
    script_content_and_offset_for_context(ctx, is_setup).map(|(content, _)| content)
}

/// Returns the script (or script setup) content along with the byte offset of
/// the block's content within the full SFC. The offset lets callers translate
/// SFC-absolute cursor positions into script-local positions, which is the
/// coordinate system used by Croquis scope spans.
pub(super) fn script_content_and_offset_for_context(
    ctx: &IdeContext<'_>,
    is_setup: bool,
) -> Option<(String, usize)> {
    let descriptor = ctx.descriptor()?;
    if is_setup {
        descriptor
            .script_setup
            .as_ref()
            .map(|script| (String::from(script.content.as_ref()), script.loc.start))
    } else {
        descriptor
            .script
            .as_ref()
            .map(|script| (String::from(script.content.as_ref()), script.loc.start))
    }
}

/// True for scope kinds that only become visible from inside the script setup
/// body (closures, blocks, v-for, etc.). Module-level and global scopes are
/// excluded so we don't re-add Vue Composition API names that
/// `composition_api_completions` already covers.
pub(super) fn is_nested_user_scope(kind: ScopeKind) -> bool {
    matches!(
        kind,
        ScopeKind::Closure
            | ScopeKind::Block
            | ScopeKind::Function
            | ScopeKind::Callback
            | ScopeKind::EventHandler
            | ScopeKind::VFor
            | ScopeKind::VSlot
            | ScopeKind::ClientOnly
            | ScopeKind::Universal
    )
}

pub(super) fn scope_kind_short_label(kind: ScopeKind) -> &'static str {
    match kind {
        ScopeKind::Closure => "closure",
        ScopeKind::Block => "block",
        ScopeKind::Function => "function",
        ScopeKind::Callback => "callback",
        ScopeKind::EventHandler => "event handler",
        ScopeKind::VFor => "v-for",
        ScopeKind::VSlot => "v-slot",
        ScopeKind::ClientOnly => "lifecycle hook",
        ScopeKind::Universal => "setup body",
        _ => "local",
    }
}

/// True when the receiver looks like an identifier or member chain rather
/// than a numeric literal. `1.` and `42.` are decimal-literal contexts even
/// though `CursorContext` exposes them as `MemberAccess { receiver: "1" }`.
pub(super) fn receiver_is_member_chain(receiver: &str) -> bool {
    receiver
        .bytes()
        .any(|b| !b.is_ascii_digit() && b != b']' && b != b'.')
}

pub(super) fn member_access_receiver(content: &str, offset: usize) -> Option<&str> {
    let before = content.get(..offset.min(content.len()))?.trim_end();
    let head = before.strip_suffix('.')?;
    let receiver_start = head
        .bytes()
        .rposition(|byte| !is_receiver_byte(byte))
        .map_or(0, |index| index + 1);
    let receiver = head.get(receiver_start..)?;
    (!receiver.is_empty()).then_some(receiver)
}

// Mirrors `CursorContext::is_receiver_byte`. Including `.` and `]` keeps
// chained receivers like `count.value` and indexed receivers like `arr[0]`
// intact so the two detectors agree (cf. issue #751).
#[inline]
fn is_receiver_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'$' | b'.' | b']')
}
