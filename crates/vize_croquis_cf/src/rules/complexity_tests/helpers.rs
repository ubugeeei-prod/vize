use crate::graph::ModuleNode;
use vize_carton::{CompactString, smallvec};
use vize_croquis::VForScopeData;

pub(super) fn v_for_data(value_alias: &str, source: &str) -> VForScopeData {
    VForScopeData {
        value_alias: CompactString::new(value_alias),
        value_bindings: smallvec![CompactString::new(value_alias)],
        key_alias: None,
        index_alias: None,
        source: CompactString::new(source),
        key_expression: None,
    }
}

pub(super) fn component_node(id: crate::FileId, path: &str, name: &str) -> ModuleNode {
    let mut node = ModuleNode::new(id, path);
    node.component_name = Some(CompactString::new(name));
    node
}
