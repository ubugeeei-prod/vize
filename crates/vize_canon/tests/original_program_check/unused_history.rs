//! The authored Batch control gains neutral custody, without a native oracle.

use super::*;
use vize_l2::file::{DeclarationKind, Namespace};
use vize_l4::targets::ts::project_program;

#[test]
fn historical_unused_typed_parameter_keeps_neutral_custody_and_changed_source_refusal() {
    let source = include_str!(
        "../../../../tests/_fixtures/differential/typechecker/authored-unused-symbols/helper.ts.txt"
    );
    let arena = Allocator::default();
    let original = file(&arena, source, Lang::Ts);
    assert!(original.is_complete(), "{:?}", original.issues());
    assert!(original.issues().is_empty());
    assert!(core::ptr::eq(original.artifact().source(), source));
    assert_eq!(original.scopes().len(), 2);
    let function = original
        .lookup(original.units()[0].scope, "echo", Namespace::Value)
        .unwrap();
    let parameter = original
        .lookup(original.scopes()[1].id, "unused", Namespace::Value)
        .unwrap();
    assert_eq!(
        parameter.declaration().unwrap().kind,
        DeclarationKind::Parameter
    );
    assert_eq!(
        parameter.declaration().unwrap().span.slice(source),
        "unused"
    );
    assert_eq!(original.exports().len(), 1);
    assert_eq!(original.exports()[0].local, Some(function.id()));
    assert!(original.references().is_empty());
    let projection = project_program(&original).unwrap();
    assert!(core::ptr::eq(projection.file(), &original));
    assert_eq!(
        projection.document().as_str(),
        vize_l0::cstr!("{source}\n;\nexport {{}};\n")
    );
    let root = tempfile::TempDir::new().unwrap();
    let path = root.path().join("helper.ts");
    std::fs::write(&path, source).unwrap();
    let bridge = project(root.path(), true);
    std::fs::write(
        &path,
        vize_l0::cstr!("{source}// changed after File completion\n"),
    )
    .unwrap();
    assert!(matches!(
        block_on(bridge.check_original_program(&original, &path)),
        Err(OriginalProgramError::SourceChanged)
    ));
    std::fs::write(&path, source).unwrap();
    assert_eq!(std::fs::read(&path).unwrap(), source.as_bytes());
}
