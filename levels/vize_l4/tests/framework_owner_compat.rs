use core::any::type_name;

#[test]
fn framework_owned_projection_keeps_its_existing_public_type_identity() {
    let expected = "vize_l4::targets::ts::vue::VueProjection<'_, '_>";
    assert_eq!(
        type_name::<vize_l4::targets::ts::vue::VueProjection<'static, 'static>>(),
        expected
    );
    assert_eq!(
        type_name::<vize_l4::framework::VueProjection<'static, 'static>>(),
        expected
    );
}
