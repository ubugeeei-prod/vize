//! Complete attribute entity accumulation at the existing callback boundary.

use vize_l1::markup::entity::DecodedEntity;

use super::Parser;

impl<'a> Parser<'a> {
    /// Complete decoded values arrive only at an actual attribute entity.
    /// Named expansions already borrow valid UTF-8 from the entity table;
    /// append those bytes once instead of decoding and encoding each scalar.
    #[cold]
    pub(in crate::parser) fn on_attrib_entity_value_impl(
        &mut self,
        value: DecodedEntity,
        start: usize,
        end: usize,
    ) {
        match value {
            DecodedEntity::Named("") => {}
            DecodedEntity::Named(content) => {
                self.flush_pending_text();
                self.accumulate_attr_or_dir_value(content, start, end);
            }
            DecodedEntity::Numeric(ch) => {
                self.flush_pending_text();
                self.on_attrib_entity_impl(ch, start, end);
            }
        }
    }
}
