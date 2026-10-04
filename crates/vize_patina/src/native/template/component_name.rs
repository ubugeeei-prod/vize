//! Genuine registered filename callback over the original native bare root.

use crate::{
    component_name::{EXCEPTION_NAMES, is_kebab_case, is_nuxt_route_file, is_pascal_case},
    rules::vue::ComponentDefinitionNameCasing,
};
use vize_l1::markup::NativeLintComponent;

use super::{NativeTemplateLintContext, NativeTemplateLintRefusal, NativeTemplateRule};

impl NativeTemplateRule for ComponentDefinitionNameCasing {
    fn run_on_template<'a>(
        &self,
        context: &mut NativeTemplateLintContext<'_, 'a>,
        root: &NativeLintComponent<'a>,
    ) -> Result<(), NativeTemplateLintRefusal> {
        if !core::ptr::eq(context.owner(), root) {
            return Err(NativeTemplateLintRefusal::SourceMismatch);
        }
        let filename = context.filename();
        if !filename.ends_with(".vue") || is_nuxt_route_file(filename) {
            return Ok(());
        }
        let stem = filename
            .rsplit('/')
            .next()
            .unwrap_or(filename)
            .rsplit('\\')
            .next()
            .unwrap_or(filename)
            .trim_end_matches(".vue");
        if EXCEPTION_NAMES.contains(&stem)
            || stem.starts_with('[')
            || stem.chars().all(|c| c.is_ascii_lowercase())
        {
            return Ok(());
        }
        if !is_pascal_case(stem) && !is_kebab_case(stem) {
            context.warn_root_with_help(
                "vue/component-definition-name-casing.message",
                &[("name", stem)],
                "vue/component-definition-name-casing.help",
            );
        }
        Ok(())
    }
}
