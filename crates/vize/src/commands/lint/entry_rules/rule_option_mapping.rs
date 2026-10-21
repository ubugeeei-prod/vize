use vize_l0::config::{
    CustomEventNameCasing as ConfigEventNameCasing, TemplateComponentNameCasing,
};

pub(super) fn component_casing(
    casing: TemplateComponentNameCasing,
) -> vize_patina::rules::ComponentCasing {
    match casing {
        TemplateComponentNameCasing::PascalCase => vize_patina::rules::ComponentCasing::PascalCase,
        TemplateComponentNameCasing::KebabCase => vize_patina::rules::ComponentCasing::KebabCase,
    }
}

pub(super) fn event_name_casing(
    casing: ConfigEventNameCasing,
) -> vize_patina::rules::script::EventNameCasing {
    match casing {
        ConfigEventNameCasing::CamelCase => vize_patina::rules::script::EventNameCasing::CamelCase,
        ConfigEventNameCasing::KebabCase => vize_patina::rules::script::EventNameCasing::KebabCase,
    }
}

pub(super) fn no_mutating_props_options(
    options: vize_l0::config::NoMutatingPropsOptions,
) -> vize_patina::rules::NoMutatingPropsOptions {
    vize_patina::rules::NoMutatingPropsOptions {
        shallow_only: options.shallow_only,
    }
}

pub(super) fn sfc_element_order_options(
    options: vize_l0::config::SfcElementOrderOptions,
) -> vize_patina::rules::SfcElementOrderOptions {
    vize_patina::rules::SfcElementOrderOptions {
        order: options
            .order
            .into_iter()
            .map(|group| vize_patina::rules::SfcElementOrderGroup::new(group.selectors()))
            .collect(),
    }
}

pub(super) fn html_self_closing_options(
    options: vize_l0::config::HtmlSelfClosingOptions,
) -> vize_patina::rules::HtmlSelfClosingOptions {
    vize_patina::rules::HtmlSelfClosingOptions {
        html: vize_patina::rules::HtmlSelfClosingHtmlOptions {
            void: html_self_closing_style(options.html.void_elements),
            normal: html_self_closing_style(options.html.normal),
            component: html_self_closing_style(options.html.component),
        },
        svg: html_self_closing_style(options.svg),
        math: html_self_closing_style(options.math),
    }
}

pub(super) fn html_quotes(
    options: vize_l0::config::HtmlQuotesConfig,
) -> vize_patina::rules::vue::HtmlQuotes {
    vize_patina::rules::vue::HtmlQuotes {
        style: match options.style() {
            vize_l0::config::HtmlQuoteStyle::Double => {
                vize_patina::rules::vue::HtmlQuotesOption::Double
            }
            vize_l0::config::HtmlQuoteStyle::Single => {
                vize_patina::rules::vue::HtmlQuotesOption::Single
            }
        },
        avoid_escape: options.avoid_escape(),
    }
}

pub(super) fn mustache_interpolation_spacing(
    style: vize_l0::config::MustacheInterpolationSpacingStyle,
) -> vize_patina::rules::vue::MustacheInterpolationSpacingStyle {
    match style {
        vize_l0::config::MustacheInterpolationSpacingStyle::Always => {
            vize_patina::rules::vue::MustacheInterpolationSpacingStyle::Always
        }
        vize_l0::config::MustacheInterpolationSpacingStyle::Never => {
            vize_patina::rules::vue::MustacheInterpolationSpacingStyle::Never
        }
    }
}

fn html_self_closing_style(
    style: vize_l0::config::HtmlSelfClosingStyle,
) -> vize_patina::rules::HtmlSelfClosingStyle {
    match style {
        vize_l0::config::HtmlSelfClosingStyle::Always => {
            vize_patina::rules::HtmlSelfClosingStyle::Always
        }
        vize_l0::config::HtmlSelfClosingStyle::Never => {
            vize_patina::rules::HtmlSelfClosingStyle::Never
        }
        vize_l0::config::HtmlSelfClosingStyle::Any => vize_patina::rules::HtmlSelfClosingStyle::Any,
    }
}

pub(super) fn v_on_event_hyphenation_style(
    style: vize_l0::config::HyphenationStyle,
) -> vize_patina::rules::VOnEventHyphenationStyle {
    match style {
        vize_l0::config::HyphenationStyle::Always => {
            vize_patina::rules::VOnEventHyphenationStyle::Always
        }
        vize_l0::config::HyphenationStyle::Never => {
            vize_patina::rules::VOnEventHyphenationStyle::Never
        }
    }
}

pub(super) fn v_on_style(
    style: vize_l0::config::VOnStyle,
) -> vize_patina::rules::vue::VOnStyleOption {
    match style {
        vize_l0::config::VOnStyle::Shorthand => vize_patina::rules::vue::VOnStyleOption::Shorthand,
        vize_l0::config::VOnStyle::Longform => vize_patina::rules::vue::VOnStyleOption::Longform,
    }
}

pub(super) fn v_slot_style(
    style: vize_l0::config::VSlotStyle,
) -> vize_patina::rules::vue::VSlotStyle {
    use vize_l0::config::{VSlotNamedStyle, VSlotStyle, VSlotUniformStyle};
    use vize_patina::rules::vue::{VSlotStyle as PatinaVSlotStyle, VSlotStyleOption};

    match style {
        VSlotStyle::Uniform(style) => PatinaVSlotStyle::uniform(match style {
            VSlotUniformStyle::Shorthand => VSlotStyleOption::Shorthand,
            VSlotUniformStyle::Longform => VSlotStyleOption::Longform,
        }),
        VSlotStyle::ByPosition(options) => PatinaVSlotStyle {
            at_component: v_slot_style_value(options.at_component),
            default_slot: v_slot_style_value(options.default_slot),
            named: match options.named {
                VSlotNamedStyle::Shorthand => VSlotStyleOption::Shorthand,
                VSlotNamedStyle::Longform => VSlotStyleOption::Longform,
            },
        },
    }
}

fn v_slot_style_value(
    style: vize_l0::config::VSlotStyleValue,
) -> vize_patina::rules::vue::VSlotStyleOption {
    match style {
        vize_l0::config::VSlotStyleValue::Shorthand => {
            vize_patina::rules::vue::VSlotStyleOption::Shorthand
        }
        vize_l0::config::VSlotStyleValue::Longform => {
            vize_patina::rules::vue::VSlotStyleOption::Longform
        }
        vize_l0::config::VSlotStyleValue::VSlot => vize_patina::rules::vue::VSlotStyleOption::VSlot,
    }
}

pub(super) fn v_bind_style(
    style: vize_l0::config::VBindStyle,
) -> vize_patina::rules::vue::VBindStyleOption {
    match style {
        vize_l0::config::VBindStyle::Shorthand => {
            vize_patina::rules::vue::VBindStyleOption::Shorthand
        }
        vize_l0::config::VBindStyle::Longform => {
            vize_patina::rules::vue::VBindStyleOption::Longform
        }
    }
}

pub(super) fn attribute_hyphenation_style(
    style: vize_l0::config::HyphenationStyle,
) -> vize_patina::rules::HyphenationStyle {
    match style {
        vize_l0::config::HyphenationStyle::Always => vize_patina::rules::HyphenationStyle::Always,
        vize_l0::config::HyphenationStyle::Never => vize_patina::rules::HyphenationStyle::Never,
    }
}
