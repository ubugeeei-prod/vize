use vize_glyph::{FormatOptions, format_template};

#[test]
fn dotted_component_tags_are_preserved() {
    let source = r#"<routeLeaveConfirm.ModalAlert
  :is-opened="routeLeaveConfirm.alertCtx.isOpened.value"
  @click:cancel="routeLeaveConfirm.alertCtx.close"
>
  <p>Unsaved changes</p>
</routeLeaveConfirm.ModalAlert>"#;

    let options = FormatOptions::default();
    let first = format_template(source, &options).unwrap();
    let second = format_template(&first, &options).unwrap();

    assert_eq!(
        first.as_str(),
        r#"<routeLeaveConfirm.ModalAlert
  :is-opened="routeLeaveConfirm.alertCtx.isOpened.value"
  @click:cancel="routeLeaveConfirm.alertCtx.close"
>
  <p>
    Unsaved changes
  </p>
</routeLeaveConfirm.ModalAlert>"#
    );
    assert_eq!(first, second);
}
