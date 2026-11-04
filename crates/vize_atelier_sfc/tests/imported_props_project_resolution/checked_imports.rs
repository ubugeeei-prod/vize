//! Qualified imported macro references must fail rather than lose props.

use super::{compile_source, project};

#[test]
fn unresolved_parents_preserve_relative_imports_and_resolved_aliases() {
    use super::{compile, expected, write};
    use vize_atelier_sfc::script::SfcTypeProject;
    let project = project();
    write(
        project.path(),
        "tsconfig.json",
        r#"{"extends":["./missing.json","./base.json"]}"#,
    );
    write(
        project.path(),
        "base.json",
        r#"{"compilerOptions":{"paths":{"@lib/*":["./src/*"]}}}"#,
    );
    let captured = SfcTypeProject::from_tsconfig(&project.path().join("tsconfig.json")).unwrap();
    assert_eq!(
        captured.configuration_diagnostics(),
        [vize_carton::cstr!(
            "cannot resolve tsconfig extends \"./missing.json\" from {}",
            project.path().join("tsconfig.json").display()
        )]
    );
    for import in ["./models/FooProps", "@lib/models/FooProps"] {
        assert_eq!(
            compile(project.path(), "src/Partial.vue", import, false),
            expected()
        );
    }
}

#[test]
fn a_generated_parent_invalidates_the_partial_project_capture() {
    use super::{compile, expected, write};
    use vize_atelier_sfc::script::SfcTypeProject;
    let project = project();
    let config = project.path().join("tsconfig.json");
    write(
        project.path(),
        "tsconfig.json",
        r#"{"extends":"./.nuxt/tsconfig.json"}"#,
    );
    let before = SfcTypeProject::from_tsconfig(&config).unwrap();
    let diagnostics = [vize_carton::cstr!(
        "cannot resolve tsconfig extends \"./.nuxt/tsconfig.json\" from {}",
        config.display()
    )];
    assert_eq!(before.configuration_diagnostics(), diagnostics);
    assert_eq!(
        compile(
            project.path(),
            "src/Partial.vue",
            "./models/FooProps",
            false
        ),
        expected()
    );
    write(
        project.path(),
        ".nuxt/tsconfig.json",
        r#"{"compilerOptions":{"paths":{"@generated/*":["../src/*"]}}}"#,
    );
    let after = SfcTypeProject::from_tsconfig(&config).unwrap();
    assert!(!std::sync::Arc::ptr_eq(&before, &after));
    assert!(after.configuration_diagnostics().is_empty());
    assert_eq!(before.configuration_diagnostics(), diagnostics);
    assert_eq!(
        compile(
            project.path(),
            "app/Generated.vue",
            "@generated/models/FooProps",
            false
        ),
        expected()
    );
}

#[test]
fn unresolved_namespace_import_in_a_macro_is_not_silently_erased() {
    let project = project();
    let error = compile_source(project.path(), "<script setup lang=\"ts\">import type * as Types from '@missing/Props';defineProps<Types.Foo>();</script>").expect_err("unresolved qualified type");
    assert_eq!(error.code.as_deref(), Some("V_SFC_TYPE_RESOLUTION"));
    assert_eq!(
        error.message,
        "Cannot resolve imported macro type `Types` from `@missing/Props`."
    );
}

#[test]
fn property_value_types_do_not_need_to_resolve_the_props_shape() {
    use super::{RuntimeProp, RuntimeProps};
    use oxc_ast_visit::Visit;
    let project = project();
    for (declaration, shape, required) in [
        ("", "{ value?: Leaf }", false),
        ("type Props = { value?: Leaf };", "Props", false),
        ("", "Record<'value', Leaf>", true),
    ] {
        let source = vize_carton::cstr!(
            "<script setup lang=\"ts\">import type {{ Leaf }} from '@missing/Leaf'; {declaration} defineProps<{shape}>();</script>"
        );
        let result =
            compile_source(project.path(), &source).expect("value type is not the property shape");
        assert_eq!(result.errors.len(), 0);
        let allocator = oxc_allocator::Allocator::default();
        let parsed =
            oxc_parser::Parser::new(&allocator, &result.code, oxc_span::SourceType::mjs()).parse();
        assert_eq!(parsed.diagnostics.len(), 0);
        let mut props = RuntimeProps::default();
        props.visit_program(&parsed.program);
        assert_eq!(
            props.0,
            vec![RuntimeProp {
                name: "value".into(),
                constructor: None,
                required: Some(required)
            }]
        );
    }
}

#[test]
fn new_source_windows_detect_same_length_edits_with_unchanged_mtime() {
    use super::{RuntimeProp, RuntimeProps, write};
    use oxc_ast_visit::Visit;
    use vize_atelier_sfc::{begin_type_resolution_batch, script::TypeSourceSnapshot};
    let project = project();
    let types = project.path().join("src/models/FooProps.ts");
    let original = "export interface FooProps { bar?: string }";
    let changed = "export interface FooProps { bar?: number }";
    write(project.path(), "src/models/FooProps.ts", original);
    let metadata = std::fs::metadata(&types).unwrap();
    let stamp = (metadata.modified().unwrap(), metadata.len());
    let saved = TypeSourceSnapshot::default();
    assert_eq!(saved.read(&types).unwrap().as_ref(), original);
    let props = || {
        let result = compile_source(project.path(), "<script setup lang=\"ts\">import type { FooProps } from '@lib/models/FooProps'; defineProps<FooProps>();</script>").unwrap();
        assert_eq!(result.errors.len(), 0);
        let allocator = oxc_allocator::Allocator::default();
        let parsed =
            oxc_parser::Parser::new(&allocator, &result.code, oxc_span::SourceType::mjs()).parse();
        assert_eq!(parsed.diagnostics.len(), 0);
        let mut props = RuntimeProps::default();
        props.visit_program(&parsed.program);
        props.0
    };
    let expected = |constructor: &str| {
        vec![RuntimeProp {
            name: "bar".into(),
            constructor: Some(constructor.into()),
            required: Some(false),
        }]
    };
    assert_eq!(props(), expected("String"));
    std::fs::write(&types, changed).unwrap();
    std::fs::File::options()
        .write(true)
        .open(&types)
        .unwrap()
        .set_times(std::fs::FileTimes::new().set_modified(stamp.0))
        .unwrap();
    let edited = std::fs::metadata(&types).unwrap();
    assert_eq!((edited.modified().unwrap(), edited.len()), stamp);
    assert_eq!(props(), expected("Number"));
    assert_eq!(saved.read(&types).unwrap().as_ref(), original);
    std::fs::write(&types, original).unwrap();
    std::fs::File::options()
        .write(true)
        .open(&types)
        .unwrap()
        .set_times(std::fs::FileTimes::new().set_modified(stamp.0))
        .unwrap();
    let _batch = begin_type_resolution_batch();
    assert_eq!(props(), expected("String"));
    assert_eq!(props(), expected("String"));
}
