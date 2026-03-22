//! Vue-specific lint rules.
//!
//! These rules are compatible with eslint-plugin-vue's essential and
//! strongly-recommended rule sets.
//!
//! ## Rule Categories
//!
//! - **Essential**: Prevent errors (default severity: Error)
//! - **Strongly Recommended**: Improve readability (default severity: Warning)
//! - **Recommended**: Ensure consistency (default severity: Warning)

// Essential rules
mod attribute_hyphenation;
mod attribute_order;
mod no_child_content;
mod no_dupe_v_else_if;
mod no_duplicate_attributes;
mod no_reserved_component_names;
mod no_template_key;
mod no_textarea_mustache;
mod no_unused_vars;
mod no_use_v_if_with_v_for;
mod no_useless_template_attributes;
mod no_v_text_v_html_on_component;
mod prop_name_casing;
mod require_component_is;
mod require_scoped_style;
mod require_v_for_key;
mod valid_attribute_name;
mod valid_v_bind;
mod valid_v_else;
mod valid_v_for;
mod valid_v_if;
mod valid_v_memo;
mod valid_v_model;
mod valid_v_on;
mod valid_v_show;
mod valid_v_slot;

// Strongly recommended rules
mod component_definition_name_casing;
mod html_quotes;
mod mustache_interpolation_spacing;
mod no_lone_template;
mod no_multi_spaces;
mod sfc_element_order;
mod v_on_style;
mod v_slot_style;
// Most implementations live under rules::opinionated::vue and are re-exported here.

// Security rules
mod no_unsafe_url;
mod no_v_html;

// Semantic analysis rules (require Croquis)
mod no_mutating_props;
mod no_undefined_refs;
mod no_unused_components;
mod no_unused_properties;

// Accessibility rules
mod a11y_img_alt;
mod permitted_contents;
// `use_unique_element_ids` implementation lives under rules::opinionated::vue.

// Style rules
mod single_style_block;
// Opinionated style rules live under rules::opinionated::vue.

// Warning rules
// Opinionated warning rules live under rules::opinionated::vue.

// Essential rules exports
pub use crate::rules::opinionated::vue::MultiWordComponentNames;
pub use crate::rules::opinionated::vue::UseVOnExact;
pub use no_child_content::NoChildContent;
pub use no_dupe_v_else_if::NoDupeVElseIf;
pub use no_duplicate_attributes::NoDuplicateAttributes;
pub use no_reserved_component_names::NoReservedComponentNames;
pub use no_template_key::NoTemplateKey;
pub use no_textarea_mustache::NoTextareaMustache;
pub use no_unused_vars::NoUnusedVars;
pub use no_use_v_if_with_v_for::NoUseVIfWithVFor;
pub use no_useless_template_attributes::NoUselessTemplateAttributes;
pub use no_v_text_v_html_on_component::NoVTextVHtmlOnComponent;
pub use require_component_is::RequireComponentIs;
pub use require_v_for_key::RequireVForKey;
pub use valid_attribute_name::ValidAttributeName;
pub use valid_v_bind::ValidVBind;
pub use valid_v_else::ValidVElse;
pub use valid_v_for::ValidVFor;
pub use valid_v_if::ValidVIf;
pub use valid_v_memo::ValidVMemo;
pub use valid_v_model::ValidVModel;
pub use valid_v_on::ValidVOn;
pub use valid_v_show::ValidVShow;
pub use valid_v_slot::ValidVSlot;

// Strongly recommended rules exports
pub use crate::rules::opinionated::vue::HtmlSelfClosing;
pub use crate::rules::opinionated::vue::NoTemplateShadow;
pub use crate::rules::opinionated::vue::{VBindStyle, VBindStyleOption};
pub use attribute_hyphenation::AttributeHyphenation;
pub use component_definition_name_casing::ComponentDefinitionNameCasing;
pub use html_quotes::{HtmlQuotes, HtmlQuotesOption};
pub use mustache_interpolation_spacing::MustacheInterpolationSpacing;
pub use no_multi_spaces::NoMultiSpaces;
pub use prop_name_casing::PropNameCasing;
pub use v_on_style::{VOnStyle, VOnStyleOption};
pub use v_slot_style::VSlotStyle;

// Recommended rules exports
pub use crate::rules::opinionated::vue::ComponentNameInTemplateCasing;
pub use crate::rules::opinionated::vue::NoInlineStyle;
pub use crate::rules::opinionated::vue::PreferPropsShorthand;
pub use crate::rules::opinionated::vue::RequireComponentRegistration;
pub use crate::rules::opinionated::vue::ScopedEventNames;
pub use attribute_order::AttributeOrder;
pub use no_lone_template::NoLoneTemplate;
pub use sfc_element_order::SfcElementOrder;

// Security rules exports
pub use no_unsafe_url::NoUnsafeUrl;
pub use no_v_html::NoVHtml;

// Semantic analysis rules exports
pub use no_mutating_props::NoMutatingProps;
pub use no_undefined_refs::NoUndefinedRefs;
pub use no_unused_components::NoUnusedComponents;
pub use no_unused_properties::NoUnusedProperties;

// Accessibility rules exports
pub use crate::rules::opinionated::vue::UseUniqueElementIds;
pub use a11y_img_alt::A11yImgAlt;
pub use permitted_contents::PermittedContents;

// HTML conformance rules exports
pub use crate::rules::opinionated::vue::NoBooleanAttrValue;

// Style rules exports
pub use crate::rules::opinionated::vue::NoPreprocessorLang;
pub use crate::rules::opinionated::vue::NoScriptNonStandardLang;
pub use crate::rules::opinionated::vue::NoSrcAttribute;
pub use crate::rules::opinionated::vue::NoTemplateLang;
pub use require_scoped_style::RequireScopedStyle;
pub use single_style_block::SingleStyleBlock;

// Warning rules exports
pub use crate::rules::opinionated::vue::WarnCustomBlock;
pub use crate::rules::opinionated::vue::WarnCustomDirective;
