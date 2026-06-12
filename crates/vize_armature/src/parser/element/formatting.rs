//! Adoption-agency recovery for misnested formatting end tags.

use vize_carton::Vec;
use vize_relief::*;

use super::super::{Parser, ParserStackEntry, StackInsertion};

impl<'a> Parser<'a> {
    pub(super) fn handle_formatting_end_tag(
        &mut self,
        tag: &str,
        start: usize,
        end: usize,
    ) -> bool {
        if !Self::is_formatting_tag(tag) {
            return false;
        }
        if self
            .stack
            .last()
            .is_some_and(|entry| entry.element.tag.eq_ignore_ascii_case(tag))
        {
            return false;
        }
        let Some(index) = self.find_open_element_index(tag) else {
            return false;
        };
        if index + 1 == self.stack.len() {
            return false;
        }

        let loc = self.create_loc(start.saturating_sub(2), end + 1);
        self.report_tree_construction_recovery(
            &loc,
            "Misnested formatting end tag was repaired using the adoption agency recovery path.",
        );

        let mut reopened = Vec::new_in(self.allocator);
        while self.stack.len() > index + 1 {
            if let Some(entry) = self.pop_stack_entry() {
                if Self::is_formatting_tag(entry.element.tag.as_str()) {
                    reopened.push(Self::formatting_shell(
                        self.allocator,
                        &entry.element,
                        self.in_pre,
                        self.in_v_pre,
                    ));
                }
                let mut parent = self
                    .pop_stack_entry()
                    .expect("formatting match is still open");
                self.push_entry_as_child(&mut parent.element.children, entry);
                self.push_stack_entry(parent);
            }
        }

        let match_index = self.stack.len() - 1;
        self.close_stack_element_at(match_index, false);

        for mut entry in reopened.into_iter().rev() {
            entry.in_pre = self.in_pre;
            entry.in_v_pre = self.in_v_pre;
            self.push_stack_entry(entry);
        }

        true
    }

    fn formatting_shell(
        allocator: &'a vize_carton::Bump,
        element: &ElementNode<'a>,
        in_pre: bool,
        in_v_pre: bool,
    ) -> ParserStackEntry<'a> {
        let mut reopened = ElementNode::new(allocator, element.tag.clone(), element.loc.clone());
        reopened.ns = element.ns;
        reopened.tag_type = element.tag_type;
        ParserStackEntry {
            element: reopened,
            in_pre,
            in_v_pre,
            insertion: StackInsertion::Normal,
            implicit: true,
            fostered_before: Vec::new_in(allocator),
        }
    }

    fn is_formatting_tag(tag: &str) -> bool {
        Self::tag_in(
            tag,
            &[
                "a", "b", "big", "code", "em", "font", "i", "nobr", "s", "small", "strike",
                "strong", "tt", "u",
            ],
        )
    }
}
