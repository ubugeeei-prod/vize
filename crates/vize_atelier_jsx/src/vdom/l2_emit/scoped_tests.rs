#![expect(clippy::disallowed_macros, reason = "insta and fixtures use format!")]
use vize_croquis::Croquis;
use vize_l0::Allocator;

use crate::{JsxLang, lower_source};

use super::super::{VdomCompatOptions, VdomCompileOptions, compile_root_to_vdom};

#[test]
fn scoped_style_scope_id_emits_from_l2_vdom() {
    let allocator = Allocator::new();
    let source = r#"const A = () => <><section class="box" /><style scoped>{`.box { color: red; }`}</style></>;"#;
    let mut lowered = lower_source(&allocator, allocator.as_oxc(), source, JsxLang::Jsx);
    let analysis: &Croquis = allocator.alloc_owned(lowered.analysis);
    let mut root = lowered.roots.pop().expect("one JSX root");
    let l2 = root.l2.as_ref().expect("scoped style root projects to L2");

    assert!(super::root_is_supported(l2));

    root.root.children.clear();
    let mut diagnostics = Vec::new();
    let component = compile_root_to_vdom(
        &allocator,
        root,
        analysis,
        false,
        &VdomCompileOptions::default(),
        VdomCompatOptions::default(),
        &mut diagnostics,
        source,
    );

    assert!(diagnostics.is_empty(), "{diagnostics:?}");
    assert!(
        component.map.is_none(),
        "L2 VDOM emission is source-map-free"
    );

    let scoped_style = component
        .scoped_style
        .as_ref()
        .expect("scoped style metadata");
    let scope_id = scoped_style.scope_id.as_str();
    let hash = scope_id.strip_prefix("data-v-").expect("scope id prefix");
    assert_eq!(hash.len(), 8);
    assert!(hash.chars().all(|ch| ch.is_ascii_hexdigit()));

    assert!(component.code.contains("_createElementBlock(\"section\""));
    assert!(
        component.code.contains(&format!("\"{scope_id}\": \"\"")),
        "{}",
        component.code
    );
    assert!(
        scoped_style.css.contains(&format!(".box[{scope_id}]")),
        "{}",
        scoped_style.css
    );
}
