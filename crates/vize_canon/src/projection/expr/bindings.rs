//! Value-bearing bindings projected through the same S4 emission document.
//!
//! Declaration patterns (`v-slot` params and `v-for` aliases) belong to scope
//! emission, not expression rows. A model's read and write currently share
//! one authored expression; emit that expression once, not a synthetic setter.

use super::{EmitDocument, ExprDialect, project_expr, project_name};
use vize_s2::op::BindingOp;

pub(super) fn project_bindings<D: ExprDialect>(
    document: &mut EmitDocument,
    dialect: &D,
    bindings: &[BindingOp<'_>],
) {
    for binding in bindings {
        let value = match binding {
            BindingOp::Bind(bind) => {
                project_name(document, dialect, bind.name);
                bind.value
            }
            BindingOp::On(on) => {
                project_name(document, dialect, on.name);
                on.handler
            }
            BindingOp::Model(model) => {
                project_name(document, dialect, model.argument);
                Some(model.contract.read)
            }
            BindingOp::SlotContent(slot) => {
                project_name(document, dialect, slot.name);
                None
            }
            BindingOp::VueDirective(directive) => {
                project_name(document, dialect, directive.argument);
                directive.value
            }
            BindingOp::VueCssBind(bind) => Some(bind.value),
            BindingOp::VueSync(sync) => Some(sync.value),
            BindingOp::VueMemo(memo) => Some(memo.value),
            BindingOp::VueShow(show) => Some(show.value),
            BindingOp::VueHtml(html) => html.value,
            BindingOp::VueText(text) => text.value,
            BindingOp::VueSlotScope(_) | BindingOp::VueOnce(_) | BindingOp::VueCloak(_) => None,
        };
        if let Some(value) = value {
            project_expr(document, dialect, value);
        }
    }
}
