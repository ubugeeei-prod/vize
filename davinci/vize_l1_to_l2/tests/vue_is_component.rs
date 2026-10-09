//! Original third-party #8328 and neighboring component-casting semantics.
mod support;
use support::with_lowered;
use vize_l0::{Allocator, Span};
use vize_l1::embed::Lang;
use vize_l1_to_l2::native::lower_component_native;
use vize_l2::op::Op;

const ORIGINAL: &str = include_str!(
    "../../../tests/_fixtures/differential/compiler/vue-is-component-8328/original.template.txt"
);
const PROPS: &str = include_str!(
    "../../../tests/_fixtures/differential/compiler/vue-is-component-8328/props-slot.template.txt"
);
const ENCODED: &str = include_str!(
    "../../../tests/_fixtures/differential/compiler/vue-is-component-8328/encoded.template.txt"
);
const PLAIN: &str = include_str!(
    "../../../tests/_fixtures/differential/compiler/vue-is-component-8328/plain.template.txt"
);

#[test]
fn original_native_element_is_a_component_in_public_semantic_ir() {
    with_lowered(ORIGINAL, |lowered, _| {
        assert!(lowered.diagnostics.is_empty());
        let [Op::Element(outer)] = lowered.root.ops.as_slice() else {
            panic!("native outer div")
        };
        let [Op::Component(component)] = outer.children.ops.as_slice() else {
            panic!("reported inner div must be a ComponentOp")
        };
        assert_eq!(component.name, "my-thing");
        assert!(component.attributes.is_empty());
        let [Op::Text(text)] = component.children.ops.as_slice() else {
            panic!("default-slot text")
        };
        assert_eq!(text.content, "x");
    });
}

#[test]
fn props_bindings_slots_and_source_spans_remain_on_the_component() {
    with_lowered(PROPS, |lowered, _| {
        assert!(lowered.diagnostics.is_empty());
        let [Op::Element(outer)] = lowered.root.ops.as_slice() else {
            panic!("outer div")
        };
        let [Op::Component(component)] = outer.children.ops.as_slice() else {
            panic!("cast button")
        };
        assert_eq!(component.name, "my-thing");
        assert_eq!(component.attributes.len(), 1);
        assert_eq!(component.attributes[0].name, "title");
        assert_eq!(component.attributes[0].value, Some("before"));
        assert_eq!(component.bindings.len(), 1);
        assert_eq!(component.children.ops.len(), 2);
        assert_eq!(component.span, Span::new(5, (PROPS.len() - 6) as u32));
        assert!(
            lowered
                .provenance
                .iter()
                .any(|record| record.rule.as_str() == "drop.vue-is"
                    && record.before.as_str() == "is=\"vue:my-thing\"")
        );
    });
}

#[test]
fn encoded_static_component_names_are_decoded_once() {
    with_lowered(ENCODED, |lowered, _| {
        let [Op::Component(component)] = lowered.root.ops.as_slice() else {
            panic!("decoded vue prefix")
        };
        assert_eq!(component.name, "my-thing");
        assert_eq!(component.attributes.len(), 1);
        assert_eq!(component.attributes[0].name, "title");
        assert_eq!(component.attributes[0].value, Some("a&amp;b"));
    });
    with_lowered("<div is=\"vue:my&amp;amp;thing\"></div>", |lowered, _| {
        let [Op::Component(component)] = lowered.root.ops.as_slice() else {
            panic!("encoded component name")
        };
        assert_eq!(component.name, "my&amp;thing");
    });
}

#[test]
fn plain_bound_and_verbatim_is_keep_the_native_element() {
    for source in [
        PLAIN,
        "<div :is=\"component\">x</div>",
        "<div is=\"Vue:my-thing\">x</div>",
        "<div v-pre is=\"vue:my-thing\">x</div>",
        "<div v-pre><span is=\"vue:my-thing\">x</span></div>",
    ] {
        with_lowered(source, |lowered, _| {
            let [Op::Element(element)] = lowered.root.ops.as_slice() else {
                panic!("native control {source}")
            };
            assert_eq!(element.tag, "div");
            if let [Op::Element(child)] = element.children.ops.as_slice() {
                assert_eq!(child.tag, "span");
            }
        });
    }
}

#[test]
fn native_construction_casts_from_once_prepared_attributes() {
    for source in [ORIGINAL, ENCODED] {
        let allocator = Allocator::default();
        let lowered = lower_component_native(&allocator, source, Lang::Js).unwrap();
        assert!(lowered.is_supported(), "{:?}", lowered.holes);
        assert_eq!(vize_l1::check_fidelity(&lowered.component.tree), Ok(()));
        let root = lowered.artifact.root();
        let component = if source == ORIGINAL {
            let [Op::Element(outer), Op::Text(tail)] = root.ops.as_slice() else {
                panic!("complete original native root")
            };
            assert_eq!(tail.content, "\n");
            let [Op::Text(before), Op::Component(component), Op::Text(after)] =
                outer.children.ops.as_slice()
            else {
                panic!("complete original native children")
            };
            assert_eq!(before.content, "\n  ");
            assert_eq!(after.content, "\n");
            component
        } else {
            let [Op::Component(component)] = root.ops.as_slice() else {
                panic!("encoded native cast")
            };
            component
        };
        assert_eq!(component.name, "my-thing");
        assert!(
            component
                .attributes
                .iter()
                .all(|attribute| attribute.name != "is")
        );
        if source == ENCODED {
            assert_eq!(component.attributes[0].value, Some("a&b"));
        }
    }
}

#[test]
fn empty_rest_is_a_component_name_and_plain_native_attributes_are_retained() {
    with_lowered("<div is=\"vue:\"></div>", |lowered, _| {
        let [Op::Component(component)] = lowered.root.ops.as_slice() else {
            panic!("prefix alone still selects a component")
        };
        assert_eq!(component.name, "");
        assert!(component.attributes.is_empty());
    });
    let allocator = Allocator::default();
    let lowered = lower_component_native(&allocator, PLAIN, Lang::Js).unwrap();
    assert!(lowered.is_supported());
    let root = lowered.artifact.root();
    let [Op::Element(element)] = root.ops.as_slice() else {
        panic!("plain native is")
    };
    assert_eq!(element.tag, "div");
    assert_eq!(element.attributes.len(), 2);
    assert_eq!(
        [
            (element.attributes[0].name, element.attributes[0].value),
            (element.attributes[1].name, element.attributes[1].value)
        ],
        [("is", Some("my-thing")), ("title", Some("native"))]
    );
}

#[test]
fn reserved_static_casts_keep_authored_identity_in_both_semantic_paths() {
    for (source, name) in [
        (
            "<div is=\"vue:component\" title=\"cast\">x</div>",
            "component",
        ),
        (
            "<div is=\"vue:Component\" title=\"cast\">x</div>",
            "Component",
        ),
        ("<div is=\"vue:slot\" title=\"cast\">x</div>", "slot"),
        (
            "<div is=\"vue:template\" title=\"cast\">x</div>",
            "template",
        ),
    ] {
        with_lowered(source, |lowered, _| {
            let [Op::Component(component)] = lowered.root.ops.as_slice() else {
                panic!("static component {name}")
            };
            assert_eq!(component.name, name);
            assert_eq!(component.span.slice(source), source);
            assert!(!vize_l0::general::is_authored_tag(
                source,
                component.span,
                name
            ));
            assert_eq!(component.attributes.len(), 1);
            assert_eq!(component.attributes[0].name, "title");
            let [Op::Text(text)] = component.children.ops.as_slice() else {
                panic!("default slot")
            };
            assert_eq!(text.content, "x");
        });
        let allocator = Allocator::default();
        let lowered = lower_component_native(&allocator, source, Lang::Js).unwrap();
        assert!(lowered.is_supported());
        let [Op::Component(component)] = lowered.artifact.root().ops.as_slice() else {
            panic!("native static component {name}")
        };
        assert_eq!(component.name, name);
        assert_eq!(component.span.slice(source), source);
        assert_eq!(component.attributes.len(), 1);
        assert_eq!(component.attributes[0].name, "title");
    }
}

#[test]
fn verbatim_table_rows_keep_the_native_implicit_wrapper() {
    for source in [
        "<table><tr v-pre is=\"vue:my-row\"><td>x</td></tr></table>",
        "<table><tr is=\"vue:my-row\" v-pre><td>x</td></tr></table>",
    ] {
        with_lowered(source, |lowered, _| {
            let [Op::Element(table)] = lowered.root.ops.as_slice() else {
                panic!("table")
            };
            assert_eq!(table.tag, "table");
            let [Op::Element(tbody)] = table.children.ops.as_slice() else {
                panic!("implicit tbody")
            };
            assert_eq!(tbody.tag, "tbody");
            let [Op::Element(row)] = tbody.children.ops.as_slice() else {
                panic!("native row")
            };
            assert_eq!(row.tag, "tr");
            assert_eq!(row.attributes.len(), 1);
            assert_eq!(
                (row.attributes[0].name, row.attributes[0].value),
                ("is", Some("vue:my-row"))
            );
            let [Op::Element(cell)] = row.children.ops.as_slice() else {
                panic!("native cell")
            };
            assert_eq!(cell.tag, "td");
            let [Op::Text(text)] = cell.children.ops.as_slice() else {
                panic!("text")
            };
            assert_eq!(text.content, "x");
        });
    }
}

#[test]
fn cast_text_policy_belongs_to_the_authored_tag() {
    for (source, name, content) in [
        ("<pre is=\"vue:my-thing\">a  b</pre>", "my-thing", "a  b"),
        ("<div is=\"vue:pre\">a  b</div>", "pre", "a b"),
        ("<div is=\"vue:textarea\">a  b</div>", "textarea", "a b"),
        (
            "<textarea is=\"vue:my-thing\">a  b</textarea>",
            "my-thing",
            "a  b",
        ),
    ] {
        with_lowered(source, |lowered, _| {
            let [Op::Component(component)] = lowered.root.ops.as_slice() else {
                panic!("static cast")
            };
            assert_eq!(component.name, name);
            let [Op::Text(text)] = component.children.ops.as_slice() else {
                panic!("text")
            };
            assert_eq!(text.content, content);
        });
    }
}

#[test]
fn ordinary_template_casts_and_special_authored_tags_keep_distinct_kinds() {
    let source = include_str!(
        "../../../tests/_fixtures/differential/compiler/vue-is-component-8328/ordinary-template.template.txt"
    );
    with_lowered(source, |lowered, _| {
        let [Op::Component(component)] = lowered.root.ops.as_slice() else {
            panic!("ordinary template cast")
        };
        assert_eq!(component.name, "my-thing");
        assert!(component.attributes.is_empty());
        let [Op::Text(text)] = component.children.ops.as_slice() else {
            panic!("slot text")
        };
        assert_eq!(text.content, "x");
    });
    let allocator = Allocator::default();
    let lowered = lower_component_native(&allocator, source, Lang::Js).unwrap();
    assert!(lowered.is_supported());
    let [Op::Component(component)] = lowered.artifact.root().ops.as_slice() else {
        panic!("native ordinary template cast")
    };
    assert_eq!(component.name, "my-thing");
    assert!(component.attributes.is_empty());
    for source in [
        "<template v-if=\"ok\" is=\"vue:my-thing\">x</template>",
        "<template is=\"vue:my-thing\" v-if=\"ok\">x</template>",
    ] {
        with_lowered(source, |lowered, _| {
            let [Op::If(branches)] = lowered.root.ops.as_slice() else {
                panic!("template branch")
            };
            let [Op::Text(text)] = branches.branches[0].region.ops.as_slice() else {
                panic!("fragment text")
            };
            assert_eq!(text.content, "x");
        });
    }
    with_lowered("<slot is=\"vue:my-thing\">x</slot>", |lowered, _| {
        assert!(matches!(lowered.root.ops.as_slice(), [Op::Slot(_)]));
    });
    for source in [
        "<template>x</template>",
        "<template v-if=\"ok\" is=\"vue:my-thing\">x</template>",
        "<template is=\"vue:my-thing\" v-if=\"ok\">x</template>",
        "<template v-for=\"item in items\" is=\"vue:my-thing\">x</template>",
        "<template is=\"vue:my-thing\" v-for=\"item in items\">x</template>",
        "<template #default is=\"vue:my-thing\">x</template>",
        "<template is=\"vue:my-thing\" #default>x</template>",
        "<slot is=\"vue:my-thing\">x</slot>",
        "<script is=\"vue:my-thing\">&amp;</script>",
        "<style is=\"vue:my-thing\">&amp;</style>",
    ] {
        let lowered = lower_component_native(&allocator, source, Lang::Js).unwrap();
        assert!(
            !lowered.is_supported(),
            "existing bounded native contract: {source}"
        );
        assert!(
            lowered
                .holes
                .iter()
                .any(|hole| hole.kind == vize_l1_to_l2::native::NativeHoleKind::UnsupportedTag)
        );
    }
}
