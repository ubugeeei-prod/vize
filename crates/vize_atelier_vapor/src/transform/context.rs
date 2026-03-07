//! Transform context for tracking state during AST-to-IR transformation.

use vize_carton::{Bump, FxHashMap, FxHashSet, String, Vec};

/// Transform context
pub(crate) struct TransformContext<'a> {
    pub(crate) allocator: &'a Bump,
    temp_id: usize,
    pub(crate) templates: Vec<'a, String>,
    pub(crate) element_template_map: FxHashMap<usize, usize>,
    pub(crate) standalone_text_elements: FxHashSet<usize>,
}

impl<'a> TransformContext<'a> {
    pub(crate) fn new(allocator: &'a Bump) -> Self {
        Self {
            allocator,
            temp_id: 0,
            templates: Vec::new_in(allocator),
            element_template_map: FxHashMap::default(),
            standalone_text_elements: FxHashSet::default(),
        }
    }

    pub(crate) fn next_id(&mut self) -> usize {
        let id = self.temp_id;
        self.temp_id += 1;
        id
    }

    pub(crate) fn add_template(&mut self, element_id: usize, template: String) -> usize {
        let template_index = self.templates.len();
        self.templates.push(template);
        self.element_template_map.insert(element_id, template_index);
        template_index
    }
}
