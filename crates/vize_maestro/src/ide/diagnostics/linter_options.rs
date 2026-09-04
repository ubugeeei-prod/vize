pub(super) fn apply_rule_options(
    mut linter: vize_patina::Linter,
    options: &vize_s0::config::ConfigLintRuleOptions,
) -> vize_patina::Linter {
    if let Some(casing) = options.component_name_in_template_casing() {
        linter = linter.with_component_name_in_template_casing(component_casing(casing));
    }
    if let Some(casing) = options.custom_event_name_casing() {
        linter = linter.with_custom_event_name_casing(event_name_casing(casing));
    }
    linter
}

fn component_casing(
    casing: vize_s0::config::TemplateComponentNameCasing,
) -> vize_patina::rules::ComponentCasing {
    match casing {
        vize_s0::config::TemplateComponentNameCasing::PascalCase => {
            vize_patina::rules::ComponentCasing::PascalCase
        }
        vize_s0::config::TemplateComponentNameCasing::KebabCase => {
            vize_patina::rules::ComponentCasing::KebabCase
        }
    }
}

fn event_name_casing(
    casing: vize_s0::config::CustomEventNameCasing,
) -> vize_patina::rules::script::EventNameCasing {
    match casing {
        vize_s0::config::CustomEventNameCasing::CamelCase => {
            vize_patina::rules::script::EventNameCasing::CamelCase
        }
        vize_s0::config::CustomEventNameCasing::KebabCase => {
            vize_patina::rules::script::EventNameCasing::KebabCase
        }
    }
}
