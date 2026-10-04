use super::*;
use vize_l2::file::NativeFileInterpolationState;

const SOURCE: &str = "<script setup>let count=2</script><template><i v-for='count in count'>{{count}}</i></template>";

#[test]
fn shadowed_original_parameter_is_the_body_read_and_foreign_equal_ids_never_join() {
    let arena = Allocator::default();
    let a = compile_native_selected_setup_sfc_dom(
        &arena,
        SOURCE,
        NativeSelectedSfcDomOptions::default(),
    );
    let b = compile_native_selected_setup_sfc_dom(
        &arena,
        SOURCE,
        NativeSelectedSfcDomOptions::default(),
    );
    assert!(a.result().is_ok() && b.result().is_ok());
    let av = a.observation().admitted().unwrap();
    let bv = b.observation().admitted().unwrap();
    let af = av.setup().file();
    let bf = bv.setup().file();
    let [Op::OriginalFor(ao)] = af.artifact().root().ops.as_slice() else {
        panic!("original");
    };
    let [Op::OriginalFor(bo)] = bf.artifact().root().ops.as_slice() else {
        panic!("foreign");
    };
    let head = af.for_head_for(ao).unwrap();
    let alias = head.value().unwrap();
    let [record] = af.native_interpolations() else {
        panic!("body");
    };
    let NativeFileInterpolationState::Admitted(node) = record.state() else {
        panic!("attached");
    };
    let analysis = build_native_selected_setup_dom_decisions(av.setup()).unwrap();
    let row = analysis.expression(node).unwrap();
    let [read] = row.reads() else {
        panic!("value");
    };
    assert_eq!(read.kind(), VueReadKind::ForValue);
    assert_eq!(read.occurrence().name, "count");
    assert_eq!(read.binding().id(), alias.id());
    assert_eq!(row.resolution().scope(), head.scope());
    assert_ne!(head.resolution().unwrap().collection().binding, alias.id());
    assert!(av.setup().binding(alias).is_err());
    assert!(alias.declaration().is_none());
    let foreign_alias = bf.for_head_for(bo).unwrap().value().unwrap();
    assert_eq!(alias.id(), foreign_alias.id());
    assert!(!row.resolution().accepts(foreign_alias));
    assert!(!alias.same_owner(foreign_alias));
    let origin = alias.template_declaration().unwrap();
    let foreign = foreign_alias.template_declaration().unwrap();
    assert!(!origin.same_owner(&foreign));
    assert_eq!(origin.declaration().origin(), ao.id());
    let params = head.resolution().unwrap().value_declaration();
    assert!(core::ptr::eq(
        origin.declaration().original().unwrap().parameter(),
        params.parameter()
    ));
    assert!(af.for_head_for(bo).is_none());
}

#[test]
fn movement_and_caught_consumer_unwind_leave_normal_program_params_and_body_ast_owned() {
    let arena = Allocator::default();
    let compiled = compile_native_selected_setup_sfc_dom(
        &arena,
        SOURCE,
        NativeSelectedSfcDomOptions {
            source_map: true,
            filename: "OriginalForAlias雪🌸.vue",
            ..NativeSelectedSfcDomOptions::default()
        },
    );
    let (program, parameter, body) = {
        let view = compiled.observation().admitted().unwrap();
        let file = view.setup().file();
        let [Op::OriginalFor(op)] = file.artifact().root().ops.as_slice() else {
            panic!("original");
        };
        (
            view.setup().program().program().body.as_ptr(),
            file.for_head_for(op)
                .unwrap()
                .resolution()
                .unwrap()
                .value_declaration()
                .parameter() as *const _,
            file.native_interpolations()[0]
                .input()
                .operand()
                .syntax()
                .expression()
                .unwrap() as *const _,
        )
    };
    let moved = Box::new(compiled);
    let view = moved.observation().admitted().unwrap();
    let file = view.setup().file();
    let caught = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let analysis = build_native_selected_setup_dom_decisions(view.setup()).unwrap();
        assert!(analysis.dom().unwrap().unsupported().is_empty());
        panic!("stop after genuine original alias read projection");
    }));
    assert!(caught.is_err());
    assert!(file.is_complete());
    assert_eq!(program, view.setup().program().program().body.as_ptr());
    let [Op::OriginalFor(op)] = file.artifact().root().ops.as_slice() else {
        panic!("original");
    };
    assert_eq!(
        parameter,
        file.for_head_for(op)
            .unwrap()
            .resolution()
            .unwrap()
            .value_declaration()
            .parameter() as *const _
    );
    assert_eq!(
        body,
        file.native_interpolations()[0]
            .input()
            .operand()
            .syntax()
            .expression()
            .unwrap() as *const _
    );
    let pack: Value = serde_json::from_str(include_str!(
        "../../../../tests/fixtures/native_original_for_value_sfc_vue_3_5_35.json"
    ))
    .unwrap();
    assert_eq!(
        moved.result().unwrap().code(),
        pack["fixtures"][2]["expectedCode"].as_str().unwrap()
    );
    assert_eq!(
        moved.result().unwrap().source_map().unwrap(),
        pack["fixtures"][2]["nativeMapRaw"].as_str().unwrap()
    );
    let retained_body = file.native_interpolations()[0]
        .input()
        .operand()
        .syntax()
        .expression()
        .unwrap();
    drop(moved);
    // The surviving stock AST grants no dropped File/For/read-row authority.
    assert_eq!(
        retained_body
            .get_identifier_reference()
            .unwrap()
            .name
            .as_str(),
        "count"
    );
}
