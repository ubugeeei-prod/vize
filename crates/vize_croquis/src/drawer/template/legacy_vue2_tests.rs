use super::super::{Drawer, DrawerOptions};
use crate::ScopeData;
use vize_armature::parse;
use vize_carton::{Allocator, CompactString, cstr};

fn undefined_refs(template: &str, legacy_vue2: bool) -> Vec<CompactString> {
    let allocator = Allocator::new();
    let (root, errors) = parse(&allocator, template);
    assert!(errors.is_empty(), "template should parse: {errors:?}");
    let mut drawer = Drawer::with_options(DrawerOptions::full());
    if legacy_vue2 {
        drawer = drawer.with_legacy_vue2();
    }
    drawer.draw_script_plain("export default {}");
    drawer.draw_template(&root);
    drawer
        .finish()
        .undefined_refs
        .iter()
        .map(|reference| reference.name.clone())
        .collect()
}

#[test]
fn slot_scope_attributes_resolve_only_inside_their_subtree() {
    for attribute in ["slot-scope", "scope"] {
        let template = cstr!(
            r#"<Child><template slot="row" {attribute}="scope"><span>{{{{ scope.row.id }}}}</span></template><p>{{{{ scope }}}}</p></Child>"#,
        );
        let undefined = undefined_refs(template.as_str(), true);
        assert_eq!(
            undefined
                .iter()
                .filter(|name| name.as_str() == "scope")
                .count(),
            1,
            "{attribute} must not leak to siblings: {undefined:?}",
        );
    }
}

#[test]
fn destructured_slot_scope_records_name_pattern_and_component() {
    let allocator = Allocator::new();
    let template = r#"<Child><template slot="item" slot-scope="{ row, index }">{{ row.id }} {{ index }}</template></Child>"#;
    let (root, errors) = parse(&allocator, template);
    assert!(errors.is_empty(), "template should parse: {errors:?}");
    let mut drawer = Drawer::with_options(DrawerOptions::full()).with_legacy_vue2();
    drawer.draw_template(&root);
    let summary = drawer.finish();
    let data = summary
        .scopes
        .iter()
        .find_map(|scope| match scope.data() {
            ScopeData::VSlot(data) => Some(data),
            _ => None,
        })
        .expect("slot-scope must create a v-slot semantic scope");

    assert_eq!(data.name, "item");
    assert_eq!(data.props_pattern.as_deref(), Some("{ row, index }"));
    assert_eq!(data.prop_names.as_slice(), ["row", "index"]);
    assert_eq!(data.component.as_deref(), Some("Child"));
}

#[test]
fn slot_scope_attributes_are_inert_without_legacy_mode() {
    let undefined = undefined_refs(
        r#"<Child><template slot-scope="scope">{{ scope.row.id }}</template></Child>"#,
        false,
    );
    assert!(
        undefined.iter().any(|name| name == "scope"),
        "Vue 3 analysis must not activate Vue 2 attribute sugar: {undefined:?}",
    );
}
