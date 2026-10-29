//! Shared context for compiling an SFC template through each script shape.

use vize_l1_to_l2::lower::pug::PugSourceMap;

use crate::compile_template::TemplateBlockCompileContext;
use crate::types::BindingMetadata;

pub(super) struct SfcTemplateContext<'a> {
    pub(super) scope_id: &'a str,
    pub(super) has_scoped: bool,
    pub(super) slotted: bool,
    pub(super) is_ts: bool,
    pub(super) component_name: &'a str,
    pub(super) experimental_self_component: bool,
    pub(super) authored_source: &'a str,
    pub(super) pug_source_map: Option<&'a PugSourceMap>,
}

impl<'a> SfcTemplateContext<'a> {
    pub(super) fn block<'b>(
        &'b self,
        inline: bool,
        bindings: Option<&'b BindingMetadata>,
        croquis: Option<vize_croquis::croquis::Croquis>,
    ) -> TemplateBlockCompileContext<'b>
    where
        'a: 'b,
    {
        TemplateBlockCompileContext {
            scope_id: self.scope_id,
            apply_scope_id: self.has_scoped,
            has_scoped: self.has_scoped,
            slotted: self.slotted,
            is_ts: self.is_ts,
            inline,
            component_name: Some(self.component_name),
            experimental_self_component: self.experimental_self_component,
            bindings,
            croquis,
            collect_map: false,
            authored_source: Some(self.authored_source),
            pug_source_map: self.pug_source_map,
        }
    }
}
