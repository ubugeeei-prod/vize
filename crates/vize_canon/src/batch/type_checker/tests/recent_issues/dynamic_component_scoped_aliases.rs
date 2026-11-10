//! `<component :is>` keeps its component target when its key or guard reads a
//! name bound by an enclosing slot or loop (#11491).

use super::super::{resolve_test_tsgo_binary, snapshot_project_diagnostics};
use vize_carton::{String, cstr};

// Pinned as `typechecker/regression/dynamic-component-scoped-aliases-11491`.
// `Clean.vue` covers slot-prop keys and guards, own and parent `v-for` keys,
// nested slots and loops, scope-bound identifier and member targets, and the
// loosely typed targets `<RouterView v-slot="{ Component }">` hands out.
// `Errors.vue` holds the negative controls.
const INPUTS: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/typechecker/dynamic-component-scoped-aliases-11491/"
);

#[test]
fn dynamic_component_targets_follow_the_scope_that_binds_their_names() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let files = ["Picker", "Panel", "Loose", "Clean", "Errors"].map(|name| {
        let source = std::fs::read_to_string(cstr!("{INPUTS}{name}.vue.txt").as_str()).unwrap();
        (cstr!("src/{name}.vue"), source)
    });
    let files = files
        .iter()
        .map(|(path, source)| (path.as_str(), source.as_str()))
        .collect::<Vec<_>>();
    let project_root =
        super::real_vue::create_project_case("dynamic-component-scoped-aliases", &files);
    std::fs::write(
        project_root.join("tsconfig.json"),
        r#"{
  "compilerOptions": {
    "strict": true,
    "module": "ESNext",
    "moduleResolution": "bundler"
  },
  "include": ["src/**/*.vue"]
}"#,
    )
    .unwrap();

    let snapshot = snapshot_project_diagnostics(&project_root);
    let _ = std::fs::remove_dir_all(&project_root);

    // The same codes and positions as vue-tsc 3.3.11; Clean.vue reports none.
    let wrong_listener = "Type '(v: number) => number' is not assignable to type '(value: string) => void'.\nTypes of parameters 'v' and 'value' are incompatible.\nType 'string' is not assignable to type 'number'.";
    let expected = [
        (
            2322,
            cstr!("20:75:error {wrong_listener}"),
        ),
        (
            2322,
            cstr!("22:33:error Type 'string' is not assignable to type 'number'."),
        ),
        (
            2345,
            cstr!("17:52:error Argument of type 'string' is not assignable to parameter of type 'number'."),
        ),
        (
            2345,
            cstr!("25:47:error Argument of type 'string' is not assignable to parameter of type 'number'."),
        ),
        (
            2551,
            cstr!("18:59:error Property 'toFixed' does not exist on type 'string'. Did you mean 'fixed'?"),
        ),
        (
            2551,
            cstr!("27:85:error Property 'toFixed' does not exist on type 'string'. Did you mean 'fixed'?"),
        ),
    ]
    .map(|(code, message)| (String::from("src/Errors.vue"), Some(code), message));
    assert_eq!(snapshot, Some(expected.to_vec()));
}
