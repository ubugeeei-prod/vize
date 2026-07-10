use vize_glyph::{FormatOptions, format_script};

#[test]
fn script_function_signature_return_type_is_idempotent() {
    let source = r#"function getSlotPosition(slotName: string): { style: { left: string, top: string, width: string, height: string }, inPortal: boolean } | null {
  return null
}
"#;
    let options = FormatOptions::default();
    let first = format_script(source, &options).unwrap();
    let second = format_script(&first, &options).unwrap();
    let third = format_script(&second, &options).unwrap();

    assert_eq!(first, second, "fmt; fmt must be a no-op");
    assert_eq!(second, third, "fmt must stay at its fixed point");
}

#[test]
fn script_check_mode_still_detects_non_fixed_point_output() {
    let source = r#"function getSlotPosition(slotName: string): { style: { left: string, top: string, width: string, height: string }, inPortal: boolean } | null {
  return null
}
"#;
    let check = FormatOptions {
        skip_script_stabilization: true,
        ..FormatOptions::default()
    };
    let write = FormatOptions::default();

    let once = format_script(source, &check).unwrap();
    let canonical = format_script(source, &write).unwrap();
    assert_ne!(
        once, canonical,
        "fixture must actually exercise the non-idempotent script path"
    );

    assert_ne!(
        format_script(&once, &check).unwrap(),
        once,
        "check-mode formatting must still detect a once-formatted intermediate"
    );
    assert_eq!(
        format_script(&canonical, &check).unwrap(),
        canonical,
        "check-mode formatting must treat the fixed point as already formatted"
    );
}

#[test]
fn script_chained_zod_regex_call_is_idempotent() {
    let source = r#"import { z } from "zod"

const schema = z.object({
  confirmCode: z
    .string()
    .regex(/^\d{6}$/, {
      message: t("form.validation.exactDigits", {
        target: t("form.field.confirmCode.label"),
        digits: 6,
      }),
    }),
})
"#;
    let options = FormatOptions::default();
    let first = format_script(source, &options).unwrap();
    let second = format_script(&first, &options).unwrap();
    let third = format_script(&second, &options).unwrap();

    assert_eq!(
        first, second,
        "first fmt pass must reach the script fixed point"
    );
    assert_eq!(second, third, "fmt must stay at its fixed point");
    assert!(
        first.contains("confirmCode: z.string().regex(/^\\d{6}$/, {"),
        "fixture must exercise the chained-call layout reported in #2025"
    );
}
