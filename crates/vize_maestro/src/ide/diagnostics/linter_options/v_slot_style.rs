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
            at_component: value(options.at_component),
            default_slot: value(options.default_slot),
            named: match options.named {
                VSlotNamedStyle::Shorthand => VSlotStyleOption::Shorthand,
                VSlotNamedStyle::Longform => VSlotStyleOption::Longform,
            },
        },
    }
}

fn value(style: vize_l0::config::VSlotStyleValue) -> vize_patina::rules::vue::VSlotStyleOption {
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
