//! Inlay-hint projection from cached descriptor and Croquis products.

use tower_lsp::lsp_types::{InlayHint, Range, Url};

use super::{InlayHintService, ecosystem};

impl InlayHintService {
    pub(crate) fn get_hints_from_products(
        content: &str,
        uri: &Url,
        range: Range,
        ecosystem_enabled: bool,
        descriptor: &vize_atelier_sfc::SfcDescriptor<'_>,
        croquis: &vize_croquis::Croquis,
    ) -> Vec<InlayHint> {
        let mut hints = Vec::new();
        if ecosystem_enabled {
            ecosystem::i18n::collect_inlay_hints(content, descriptor, Some(uri), range, &mut hints);
        }
        let Some(script_setup) = descriptor.script_setup.as_ref() else {
            return hints;
        };
        let prop_names = croquis
            .macros
            .props()
            .iter()
            .map(|prop| prop.name.to_string())
            .collect::<Vec<_>>();
        let destructured: Vec<&str> = croquis
            .macros
            .props_destructure()
            .map(|props| {
                props
                    .bindings
                    .values()
                    .map(|binding| binding.local.as_str())
                    .collect()
            })
            .unwrap_or_default();
        let define_props_end = croquis
            .macros
            .define_props()
            .map(|call| call.end as usize)
            .unwrap_or(0);
        if !destructured.is_empty() {
            Self::collect_script_props_hints(
                &script_setup.content,
                script_setup.loc.start,
                content,
                &destructured,
                define_props_end,
                range,
                &mut hints,
            );
        }
        if let Some(template) = descriptor.template.as_ref()
            && !prop_names.is_empty()
        {
            let props = prop_names.iter().map(String::as_str).collect::<Vec<_>>();
            Self::collect_template_props_hints(
                &template.content,
                template.loc.start,
                content,
                &props,
                range,
                &mut hints,
            );
        }
        Self::collect_reactive_binding_hints(
            &script_setup.content,
            script_setup.loc.start,
            content,
            croquis,
            range,
            &mut hints,
        );
        hints
    }
}
