//! Native component setup contracts around genuine shared backend renderers.

use super::{Context, JsxComponent, JsxDiagnostic, ModuleWriter, error, js_string, plans};

impl Context<'_> {
    pub(super) fn native_plan(&self, index: usize) -> Option<&plans::Plan> {
        let &(start, end) = self.spans.get(index)?;
        self.native_plans
            .iter()
            .find(|plan| plan.setup.render_start == start && plan.setup.render_end == end)
    }

    pub(super) fn native_wrapper(
        &self,
        index: usize,
        writer: &mut ModuleWriter<'_>,
    ) -> Result<(), JsxDiagnostic> {
        let component = self
            .components
            .get(index)
            .ok_or_else(|| error(0, 0, "missing native component"))?;
        let plan = self
            .native_plan(index)
            .ok_or_else(|| error(0, 0, "missing native component plan"))?;
        let setup = &plan.setup;
        writer.synthetic("({\n");
        writer.synthetic(if matches!(component, JsxComponent::Ssr(_)) {
            "__ssrInlineRender: true,\n"
        } else {
            "__vapor: true,\n"
        });
        if matches!(component, JsxComponent::Ssr(component) if component.mode == crate::JsxOutputMode::Vapor)
        {
            writer.synthetic("__vapor: true,\n");
        }
        if let Some(name) = component.component_name() {
            writer.synthetic("name: ");
            writer.synthetic(&js_string(name));
            writer.synthetic(",\n");
        }
        if !setup.destructured_props.is_empty() {
            writer.synthetic("props: [");
            for (i, prop) in setup.destructured_props.iter().enumerate() {
                if i > 0 {
                    writer.synthetic(", ");
                }
                writer.synthetic(&js_string(prop));
            }
            writer.synthetic("],\n");
        }
        if setup.is_async {
            writer.synthetic("async ");
        }
        writer.synthetic("setup");
        if setup.type_params_start != setup.type_params_end {
            writer.synthetic("<");
            writer.authored(setup.type_params_start, setup.type_params_end)?;
            writer.synthetic(">");
        }
        writer.synthetic("(");
        writer.authored(setup.params_start, setup.params_end)?;
        writer.synthetic(") {\n");
        self.authored_range(setup.setup_start, setup.setup_end, index, writer)?;
        writer.synthetic("return ");
        self.expression(index, !matches!(component, JsxComponent::Ssr(_)), writer)?;
        writer.synthetic("\n}\n})");
        Ok(())
    }
}
