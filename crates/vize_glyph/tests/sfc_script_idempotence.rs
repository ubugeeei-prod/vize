use vize_glyph::{FormatOptions, format_sfc};

#[test]
fn sfc_script_function_signature_return_type_is_idempotent() {
    let source = r#"<script setup lang="ts">
function getSlotPosition(slotName: string): { style: { left: string, top: string, width: string, height: string }, inPortal: boolean } | null {
  return null
}
</script>
"#;
    let options = FormatOptions::default();
    let first = format_sfc(source, &options).unwrap();
    let second = format_sfc(&first.code, &options).unwrap();
    let third = format_sfc(&second.code, &options).unwrap();

    assert_eq!(first.code, second.code, "fmt; fmt must be a no-op");
    assert_eq!(second.code, third.code, "fmt must stay at its fixed point");
}

#[test]
fn sfc_script_chained_zod_regex_call_is_idempotent() {
    let source = r#"<script setup lang="ts">
import { z } from "zod"

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
</script>
"#;
    let options = FormatOptions::default();
    let first = format_sfc(source, &options).unwrap();
    let second = format_sfc(&first.code, &options).unwrap();
    let third = format_sfc(&second.code, &options).unwrap();

    assert_eq!(
        first.code, second.code,
        "first fmt pass must reach the script fixed point"
    );
    assert_eq!(second.code, third.code, "fmt must stay at its fixed point");
    assert!(
        first
            .code
            .contains("confirmCode: z.string().regex(/^\\d{6}$/, {"),
        "fixture must exercise the chained-call layout reported in #2025"
    );
}
