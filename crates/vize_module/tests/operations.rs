use vize_module::{
    ModuleExpressionKind, ModuleImportBindingKind, ModuleLanguage, ModuleOperationKind,
    ModulePattern, snapshot_module,
};

#[test]
fn snapshots_import_aliases_and_nested_operation_facts_from_one_program() {
    let source = r#"
import { ref as makeRef, type Ref } from 'vue'
import pinia from 'pinia'
const count = makeRef(0)
const { value: current = 1 } = count
async function refresh(input) {
  await load(input)
  count.value = normalize(input)
  return count.value
}
"#;
    let module = snapshot_module("fixture.ts", source, ModuleLanguage::TypeScript, 100, None);

    let vue = module
        .imports
        .iter()
        .find(|import| import.specifier.as_ref() == "vue")
        .unwrap();
    assert_eq!(vue.bindings.len(), 2);
    assert_eq!(vue.bindings[0].imported.as_deref(), Some("ref"));
    assert_eq!(vue.bindings[0].local.as_ref(), "makeRef");
    assert_eq!(vue.bindings[0].kind, ModuleImportBindingKind::Named);
    assert!(!vue.bindings[0].type_only);
    assert!(vue.bindings[1].type_only);

    assert!(module.operations.operations.iter().any(|operation| {
        matches!(
            &operation.kind,
            ModuleOperationKind::Binding {
                pattern: ModulePattern::Identifier(name),
                initializer: Some(initializer),
                ..
            } if name.as_ref() == "count"
                && matches!(&initializer.kind, ModuleExpressionKind::Call { callee, .. }
                    if matches!(&callee.kind, ModuleExpressionKind::Identifier(name)
                        if name.as_ref() == "makeRef"))
        )
    }));

    let refresh = module
        .operations
        .functions
        .iter()
        .find(|function| function.name.as_deref() == Some("refresh"))
        .unwrap();
    assert!(refresh.async_);
    assert!(
        refresh
            .local_bindings
            .iter()
            .any(|name| name.as_ref() == "input")
    );
    assert!(
        refresh
            .references
            .iter()
            .any(|name| name.as_ref() == "normalize")
    );
    assert!(module.operations.operations.iter().any(|operation| {
        operation.function == Some(refresh.id)
            && operation.after_await
            && matches!(&operation.kind, ModuleOperationKind::Assignment { target, .. }
                if matches!(target, ModulePattern::Path(path)
                    if path.iter().map(AsRef::as_ref).eq(["count", "value"])))
    }));
    assert!(
        module
            .operations
            .operations
            .iter()
            .all(|operation| operation.span.start >= 100)
    );
}
