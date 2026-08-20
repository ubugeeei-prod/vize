use super::super::{Drawer, DrawerOptions};
use crate::scope::ScopeData;
use vize_armature::parse;
use vize_carton::{Bump, cstr};

#[test]
fn v_slot_object_pattern_declaration_offsets_use_local_bindings() {
    let pattern = "  { name: name, label: local }";
    let template =
        cstr!(r#"<p>前置き</p><Child v-slot="{pattern}">{{{{ name }}}}{{{{ local }}}}</Child>"#,);
    let allocator = Bump::new();
    let (root, errors) = parse(&allocator, template.as_str());
    assert!(errors.is_empty(), "template should parse: {errors:?}");
    let mut drawer = Drawer::with_options(DrawerOptions::full());
    drawer.draw_template(&root);
    let summary = drawer.finish();
    let scope = summary
        .scopes
        .iter()
        .find(|scope| matches!(scope.data(), ScopeData::VSlot(_)))
        .expect("v-slot scope should be recorded");
    let pattern_offset = template.find(pattern).unwrap() as u32;

    assert_eq!(
        scope.get_binding("name").unwrap().declaration_offset,
        pattern_offset + pattern.rfind("name").unwrap() as u32
    );
    assert_eq!(
        scope.get_binding("local").unwrap().declaration_offset,
        pattern_offset + pattern.find("local").unwrap() as u32
    );
}
