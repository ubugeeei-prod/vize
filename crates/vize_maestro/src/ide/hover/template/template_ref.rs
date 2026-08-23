use tower_lsp::lsp_types::Hover;

use super::HoverBuilder;
use crate::ide::IdeContext;

pub(crate) fn hover_static_template_ref(ctx: &IdeContext<'_>) -> Option<Hover> {
    let target = crate::ide::template_ref::target_at_offset(ctx)?;
    let mut hover = HoverBuilder::new()
        .title(&target.ref_name)
        .meta("Vue template ref")
        .description("Static template ref resolved through `useTemplateRef()`.")
        .bullets(
            "Editor behavior",
            &[
                "Hover and go-to-definition jump to the authored `useTemplateRef()` binding.",
                "When the type backend is available, hover upgrades to the binding's exact TypeScript type.",
            ],
        )
        .docs(
            "Vue Template Refs",
            "https://vuejs.org/guide/essentials/template-refs.html",
        )
        .build();
    hover.range = Some(target.value_range(&ctx.content));
    Some(hover)
}
