//! Bounded select stack rules in the HTML "in body" insertion mode.
//! No separate select mode or synthesized element is introduced.

use super::{Builder, Refusal, Tag};
use crate::markup::Profile;

impl<P: Profile> Builder<'_, '_, P> {
    pub(super) fn select_start(&mut self, tag: &Tag) -> Result<(), Refusal> {
        if !self.in_scope(&["select"], &[]) {
            return Ok(());
        }
        match tag.name {
            // The browser ignores the nested select token and closes the
            // original select. An input also closes it with a parse error.
            "select" => Err(Refusal::IgnoredTag(tag.span)),
            "input" => Err(Refusal::ImpliedEnd(tag.span)),
            "option" => {
                self.implied_ends(Some("optgroup"), tag.event)?;
                if self.in_scope(&["option"], &[]) {
                    return Err(Refusal::ImpliedEnd(tag.span));
                }
                Ok(())
            }
            "optgroup" | "hr" => {
                self.implied_ends(None, tag.event)?;
                if self.in_scope(&["option", "optgroup"], &[]) {
                    return Err(Refusal::ImpliedEnd(tag.span));
                }
                Ok(())
            }
            // Modern HTML can retain other elements in select. They need
            // their own native contracts, not the old ignore-token rule.
            _ => Err(Refusal::UnsupportedSelectContent(tag.span)),
        }
    }
}
