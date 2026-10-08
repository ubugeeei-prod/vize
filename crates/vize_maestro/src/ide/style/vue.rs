//! The four existing Vue CSS entries retain their labels, order and snippets.

use super::super::markup::Markdown;

pub(super) struct VueFeature {
    pub name: &'static str,
    pub signature: &'static str,
    pub snippet: &'static str,
    pub description: &'static str,
    pub example: &'static str,
    pub anchor: &'static str,
}

pub(super) const FEATURES: [VueFeature; 4] = [
    VueFeature {
        name: "v-bind",
        signature: "v-bind()",
        snippet: "v-bind($1)",
        description: "Bind a CSS value to component state. Vue compiles the expression to a hashed CSS custom property, updates it reactively on the component root, and supports both plain and scoped style blocks. Quote complex JavaScript expressions.",
        example: ".text { color: v-bind('theme.color'); }",
        anchor: "v-bind-in-css",
    },
    VueFeature {
        name: ":deep",
        signature: ":deep()",
        snippet: ":deep($1)",
        description: "Reach a child component's descendants from scoped CSS. Vue scopes the selector before :deep(); the selector inside :deep() stays unscoped.",
        example: ".parent :deep(.child) { color: red; }",
        anchor: "deep-selectors",
    },
    VueFeature {
        name: ":slotted",
        signature: ":slotted()",
        snippet: ":slotted($1)",
        description: "Style content passed into a slot in scoped CSS. Ordinary scoped selectors do not affect slot content because that content belongs to the component that provides it.",
        example: ":slotted(div) { color: red; }",
        anchor: "slotted-selectors",
    },
    VueFeature {
        name: ":global",
        signature: ":global()",
        snippet: ":global($1)",
        description: "Make one selector global inside a scoped style block. Vue leaves the selector inside :global() unscoped, so the rule can match outside the component.",
        example: ":global(.red) { color: red; }",
        anchor: "global-selectors",
    },
];

pub(super) fn feature(name: &str) -> Option<&'static VueFeature> {
    FEATURES.iter().find(|feature| feature.name == name)
}

pub(super) fn markdown(feature: &VueFeature) -> String {
    Markdown::new()
        .title(feature.signature)
        .meta("Vue SFC CSS feature")
        .paragraph(feature.description)
        .example("css", feature.example)
        .docs(
            "Vue SFC CSS features",
            &[
                "https://vuejs.org/api/sfc-css-features.html#",
                feature.anchor,
            ]
            .concat(),
        )
        .build()
}
