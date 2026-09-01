use super::*;

const UNSUPPORTED_BATTERY: &[(&str, &str, support::ExpectedRefusal)] = &[(
    "mixed_component_root_and_named_template",
    r#"<Foo v-slot><template #header>x</template></Foo>"#,
    support::ExpectedRefusal::Diagnostics,
)];

#[test]
fn s2_slot_forms_that_stay_unsupported_are_pinned_negative_cases() {
    support::assert_s2_refuses(UNSUPPORTED_BATTERY);
}
