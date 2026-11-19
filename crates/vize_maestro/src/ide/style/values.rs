//! Small standards-defined values supplement property-owned catalog values.

pub(super) struct StandardValue {
    pub name: &'static str,
    pub description: &'static str,
    pub reference: &'static str,
}

pub(super) const WIDE: [StandardValue; 5] = [
    StandardValue {
        name: "initial",
        description: "Use this property's initial value, as defined by its specification.",
        reference: "https://www.w3.org/TR/css-cascade-5/#initial",
    },
    StandardValue {
        name: "inherit",
        description: "Use the computed value of this property on the parent element.",
        reference: "https://www.w3.org/TR/css-cascade-5/#inherit",
    },
    StandardValue {
        name: "unset",
        description: "Inherit for an inherited property; otherwise use the property's initial value.",
        reference: "https://www.w3.org/TR/css-cascade-5/#unset",
    },
    StandardValue {
        name: "revert",
        description: "Roll back the declaration to the value established by an earlier cascade origin.",
        reference: "https://www.w3.org/TR/css-cascade-5/#revert",
    },
    StandardValue {
        name: "revert-layer",
        description: "Roll back the declaration to the value established in an earlier cascade layer.",
        reference: "https://www.w3.org/TR/css-cascade-5/#revert-layer",
    },
];

pub(super) const FUNCTIONS: [StandardValue; 2] = [
    StandardValue {
        name: "var",
        description: "Substitute the value of a CSS custom property. The optional second argument is a fallback for a missing or invalid custom property.",
        reference: "https://www.w3.org/TR/css-variables-1/#using-variables",
    },
    StandardValue {
        name: "calc",
        description: "Compute a CSS numeric value with arithmetic. The resulting value must be valid for the property; surround + and - with whitespace.",
        reference: "https://www.w3.org/TR/css-values-4/#calc-func",
    },
];

pub(super) fn lookup(
    values: &'static [StandardValue],
    name: &str,
) -> Option<&'static StandardValue> {
    values
        .iter()
        .find(|value| value.name.eq_ignore_ascii_case(name))
}

pub(super) fn accepts_colors(property: &super::data::CssEntry) -> bool {
    property.restrictions.contains(&"color")
}

pub(super) fn accepts_calc(property: &super::data::CssEntry) -> bool {
    property.restrictions.iter().any(|restriction| {
        matches!(
            *restriction,
            "length" | "percentage" | "number" | "integer" | "angle" | "time" | "frequency"
        )
    })
}
