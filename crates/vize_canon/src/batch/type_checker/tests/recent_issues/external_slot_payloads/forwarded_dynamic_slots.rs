//! A wrapper that forwards every slot must not hand consumers a union payload.
use super::super::super::{resolve_test_tsgo_binary, snapshot_project_diagnostics};
use vize_carton::String;

macro_rules! fixture {
    ($name:literal) => {
        include_str!(concat!(
            "../../../../../../../../tests/_fixtures/differential/canon/forwarded-dynamic-slots/",
            $name,
            ".vue.txt"
        ))
    };
}

#[test]
fn forwarded_broad_slot_names_stay_unchecked_while_named_payloads_stay_typed() {
    let Some(_native) = resolve_test_tsgo_binary() else {
        return;
    };
    let project = super::super::real_vue::create_project_case(
        "forwarded-dynamic-slots",
        &[
            ("src/Inner.vue", fixture!("Inner")),
            ("src/Wrapper.vue", fixture!("Wrapper")),
            ("src/App.vue", fixture!("App")),
            ("src/Direct.vue", fixture!("Direct")),
            ("src/Union.vue", fixture!("Union")),
            ("src/Indexed.vue", fixture!("Indexed")),
            ("src/IndexedUse.vue", fixture!("IndexedUse")),
        ],
    );
    std::fs::write(
        project.join("tsconfig.json"),
        r#"{"compilerOptions":{"strict":true,"target":"ES2022","module":"ESNext","moduleResolution":"bundler","skipLibCheck":true,"noEmit":true},"include":["src/**/*.ts","src/**/*.vue"]}"#,
    )
    .expect("write tsconfig");

    let mut actual = snapshot_project_diagnostics(&project).expect("native diagnostics");
    actual.sort();
    let expected: Vec<(String, Option<u32>, String)> = [
        (
            "src/Direct.vue",
            "7:70:error Property 'missing' does not exist on type '{ option: { label: string; }; }'.",
        ),
        (
            "src/IndexedUse.vue",
            "8:71:error Property 'missing' does not exist on type '{ id: number; }'.",
        ),
        (
            "src/Union.vue",
            "8:44:error Property 'missing' does not exist on type '{ option: { label: string; }; } | { value: number; }'.\nProperty 'missing' does not exist on type '{ option: { label: string; }; }'.",
        ),
    ]
    .into_iter()
    .map(|(file, message)| (String::from(file), Some(2339), String::from(message)))
    .collect();
    assert_eq!(
        actual, expected,
        "vue-tsc reports the three typed controls and nothing for the forwarding consumer"
    );
    let _ = std::fs::remove_dir_all(project);
}
