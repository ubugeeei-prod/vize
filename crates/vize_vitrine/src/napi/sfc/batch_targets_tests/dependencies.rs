//! Which dependency lists a later target leaves out.

use super::super::batch_targets::leave_out_shared_dependencies;
use super::super::BatchFileResultNapi;

fn result_with(dependencies: Option<&[&str]>) -> BatchFileResultNapi {
    BatchFileResultNapi {
        type_project: None,
        path: "/virtual/Shared.vue".into(),
        code: String::new(),
        map: None,
        css: None,
        scope_id: String::new(),
        has_scoped: false,
        errors: vec![],
        warnings: vec![],
        template_hash: None,
        style_hash: None,
        script_hash: None,
        styles: vec![],
        custom_blocks: vec![],
        macro_artifacts: vec![],
        module_shape: None,
        dependencies: dependencies.map(|list| list.iter().map(|path| (*path).into()).collect()),
    }
}

#[test]
fn only_an_equal_dependency_list_is_left_out() {
    let list: &[&str] = &["/project/types.ts"];
    let mut equal = [
        (result_with(Some(list)), true),
        (result_with(Some(list)), true),
    ];
    assert!(leave_out_shared_dependencies(&mut equal));
    let [(first, _), (later, _)] = &equal;
    assert_eq!(
        first.dependencies.as_deref(),
        Some(&["/project/types.ts".into()][..])
    );
    assert_eq!(later.dependencies, None);

    // A later target that failed has no list: it shares nothing.
    let mut failed = [(result_with(Some(list)), true), (result_with(None), false)];
    assert!(!leave_out_shared_dependencies(&mut failed));
    let mut different = [
        (result_with(Some(list)), true),
        (result_with(Some(&["/project/other.ts"])), true),
    ];
    assert!(!leave_out_shared_dependencies(&mut different));
    let [_, (later, _)] = &different;
    assert_eq!(later.dependencies.as_ref().map(Vec::len), Some(1));
    let mut none = [(result_with(None), true), (result_with(None), true)];
    assert!(!leave_out_shared_dependencies(&mut none));
}
