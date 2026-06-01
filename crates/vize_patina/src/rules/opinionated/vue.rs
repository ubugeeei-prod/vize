mod component_name_in_template_casing;
mod html_self_closing;
mod multi_word_component_names;
mod no_boolean_attr_value;
mod no_inline_style;
mod no_preprocessor_lang;
mod no_script_non_standard_lang;
mod no_src_attribute;
mod no_template_lang;
mod no_template_shadow;
mod prefer_props_shorthand;
mod require_component_registration;
mod scoped_event_names;
mod use_unique_element_ids;
mod use_v_on_exact;
mod v_bind_style;
mod warn_custom_block;
mod warn_custom_directive;

use crate::rule::RuleRegistry;

pub use component_name_in_template_casing::ComponentNameInTemplateCasing;
pub use html_self_closing::HtmlSelfClosing;
pub use multi_word_component_names::MultiWordComponentNames;
pub use no_boolean_attr_value::NoBooleanAttrValue;
pub use no_inline_style::NoInlineStyle;
pub use no_preprocessor_lang::NoPreprocessorLang;
pub use no_script_non_standard_lang::NoScriptNonStandardLang;
pub use no_src_attribute::NoSrcAttribute;
pub use no_template_lang::NoTemplateLang;
pub use no_template_shadow::NoTemplateShadow;
pub use prefer_props_shorthand::PreferPropsShorthand;
pub use require_component_registration::RequireComponentRegistration;
pub use scoped_event_names::ScopedEventNames;
pub use use_unique_element_ids::UseUniqueElementIds;
pub use use_v_on_exact::UseVOnExact;
pub use v_bind_style::{VBindStyle, VBindStyleOption};
pub use warn_custom_block::WarnCustomBlock;
pub use warn_custom_directive::WarnCustomDirective;

pub(crate) fn register(registry: &mut RuleRegistry) {
    register_shared(registry);
    registry.register(Box::new(RequireComponentRegistration::default()));
}

pub(crate) fn register_nuxt(registry: &mut RuleRegistry) {
    register_shared(registry);
}

fn register_shared(registry: &mut RuleRegistry) {
    registry.register(Box::new(MultiWordComponentNames::default()));
    registry.register(Box::new(UseVOnExact));

    registry.register(Box::new(NoTemplateShadow));
    registry.register(Box::new(VBindStyle::default()));
    registry.register(Box::new(HtmlSelfClosing));
    registry.register(Box::new(ScopedEventNames));
    registry.register(Box::new(PreferPropsShorthand));

    registry.register(Box::new(UseUniqueElementIds::default()));

    registry.register(Box::new(ComponentNameInTemplateCasing::default()));
    registry.register(Box::new(NoPreprocessorLang));
    registry.register(Box::new(NoScriptNonStandardLang));
    registry.register(Box::new(NoTemplateLang));
    registry.register(Box::new(NoSrcAttribute));
    registry.register(Box::new(NoInlineStyle));
    registry.register(Box::new(WarnCustomBlock));
    registry.register(Box::new(WarnCustomDirective));
    registry.register(Box::new(NoBooleanAttrValue));
}
